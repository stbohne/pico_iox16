use core::{array, cell::Cell, marker::PhantomData, ops::Deref};

use fugit::{Duration, Instant};
use futures::future::Either;
use pico_iox16_protocol::{
    InputGetFullReq, InputGetFullRes, InputGetReq, InputGetRes, InputGetThresholdStatesReq,
    InputGetThresholdStatesRes, InputGetThresholdTimesReq, InputGetThresholdTimesRes, InputStat,
    InputThresholdTimes,
};

use crate::{
    HandleMessage, nb_await,
    nvm::{self, NonvolatileStorage, Nvm},
    runtime::{Timer, WaitUntil as _, yield_now},
};

pub enum InputError<T> {
    UnrecoverableError(T),
    RecoverableError,
}

pub trait Input<Board: ?Sized> {
    type Error;
    /// Set the first output pin that selectes the input to read.
    fn select0(&mut self, value: bool) -> nb::Result<(), Self::Error>;
    /// Set the second output pin that selectes the input to read.
    fn select1(&mut self, value: bool) -> nb::Result<(), Self::Error>;
    /// Set the third output pin that selectes the input to read.
    fn select2(&mut self, value: bool) -> nb::Result<(), Self::Error>;
    /// Start reading the selected input on the left half of the board.
    /// The value can be read by [`read_last`](Self::read_last).
    fn start_read0(&mut self) -> nb::Result<(), Self::Error>;
    /// Start reading the selected input on the right half of the board.
    /// The value can be read by [`read_last`](Self::read_last).
    fn start_read1(&mut self) -> nb::Result<(), Self::Error>;
    /// Read the last value read by `start_read0` or `start_read1`. 
    /// Returns `Err(InputError::RecoverableError)` if the reading failed but can be started again, 
    /// or `Err(InputError::UnrecoverableError(e))` if there was an unrecoverable error.
    fn read_last(&mut self) -> nb::Result<u16, InputError<Self::Error>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputData {
    /// The average value input when it was last read. Returned when no new value has been read since then.
    pub previous_value: i16,
    /// The sum of all values read since the last time it was read, used for calculating the average value.
    pub sum: i32,
    /// The sum of squares of all values read since the last time it was read, used for calculating the standard deviation.
    pub sum_squares: u64,
    /// The minimum value read since the last time it was read.
    pub min: i16,
    /// The maximum value read since the last time it was read.
    pub max: i16,
    /// The number of values read since the last time it was read. If this overflows,
    /// the sum and sum of squares are halved to prevent overflow while still providing a reasonable average and
    /// standard deviation, and the count is reset to 0x8000 which also effectively halves the count.
    pub count: u16,
}
impl From<InputData> for InputStat {
    fn from(value: InputData) -> Self {
        Self {
            sum: value.sum.into(),
            sum_squares: value.sum_squares.into(),
            min: value.min.into(),
            max: value.max.into(),
            count: value.count.into(),
        }
    }
}
impl Default for InputData {
    fn default() -> Self {
        Self::new()
    }
}
impl InputData {
    pub const fn new() -> Self {
        Self {
            previous_value: 0,
            sum: 0,
            sum_squares: 0,
            min: i16::MAX,
            max: i16::MIN,
            count: 0,
        }
    }
    pub fn update(mut self, value: i16) -> Self {
        self.sum += value as i32;
        self.sum_squares += (value as i32 * value as i32) as u64;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.count = self.count.wrapping_add(1);
        if self.count == 0 {
            self.sum = (self.sum + 1 - (1 - self.sum % 2)) / 2;
            self.sum_squares = (self.sum_squares + 2 - (1 - self.sum_squares / 2 % 2)) / 4;
            self.count = 0x8000;
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdData<const NOM: u32, const DENOM: u32> {
    /// The last time the input went from below to above `threshold_high`
    pub last_above_threshold: Instant<u64, NOM, DENOM>,
    /// The last time the input went from above to below `threshold_low`
    pub last_below_threshold: Instant<u64, NOM, DENOM>,
    /// The number of consecutive readings above `threshold_high`
    pub above_count: u16,
    /// The number of consecutive readings below `threshold_low`
    pub below_count: u16,
    /// The last time the input went from below to above `threshold_high` after debouncing
    pub last_above_threshold_debounced: Instant<u64, NOM, DENOM>,
    /// The last time the input went from above to below `threshold_low` after debouncing
    pub last_below_threshold_debounced: Instant<u64, NOM, DENOM>,
}
impl<const NOM: u32, const DENOM: u32> ThresholdData<NOM, DENOM> {
    pub const fn new(now: Instant<u64, NOM, DENOM>) -> Self {
        Self {
            last_above_threshold: now,
            last_below_threshold: now,
            above_count: 0,
            below_count: 0,
            last_above_threshold_debounced: now,
            last_below_threshold_debounced: now,
        }
    }
    fn update(
        mut self,
        value: i16,
        now: Instant<u64, NOM, DENOM>,
        threshold: &nvm::Threshold,
    ) -> Self {
        let above_threshold = value > threshold.threshold_high;
        let below_threshold = value < threshold.threshold_low;
        if above_threshold {
            if self.above_count == 0 {
                self.last_above_threshold = now;
            }
            if self.above_count >= threshold.debounce_count
                && now - self.last_above_threshold
                    >= Duration::<u64, NOM, DENOM>::micros(threshold.debounce_time_us as u64)
            {
                self.last_above_threshold_debounced = self.last_above_threshold;
            }
            self.above_count = self.above_count.saturating_add(1);
        } else {
            self.above_count = 0;
        }
        if below_threshold {
            if self.below_count == 0 {
                self.last_below_threshold = now;
            }
            if self.below_count >= threshold.debounce_count
                && now - self.last_below_threshold
                    >= Duration::<u64, NOM, DENOM>::micros(threshold.debounce_time_us as u64)
            {
                self.last_below_threshold_debounced = self.last_below_threshold;
            }
            self.below_count = self.below_count.saturating_add(1);
        } else {
            self.below_count = 0;
        }
        self
    }
}

pub struct InputLoop<const NOM: u32, const DENOM: u32> {
    inputs: [Cell<InputData>; 16],
    thresholds: [Cell<ThresholdData<NOM, DENOM>>; 16],
}
impl<I: Deref<Target = InputLoop<NOM, DENOM>>, const NOM: u32, const DENOM: u32> HandleMessage
    for (&InputGetReq, I)
{
    type Response = InputGetRes;
    type Error = !;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetReq, input_loop) = self;
        Ok(InputGetRes {
            values: input_loop.inputs.each_ref().map(|v| {
                let data = v.get();
                let avg = if data.count == 0 {
                    data.previous_value
                } else {
                    (data.sum / i32::from(data.count)) as i16
                };
                v.set(InputData {
                    previous_value: avg,
                    ..InputData::default()
                });
                avg.into()
            }),
        })
    }
}
impl<I: Deref<Target = InputLoop<NOM, DENOM>>, const NOM: u32, const DENOM: u32> HandleMessage
    for (&InputGetFullReq, I)
{
    type Response = InputGetFullRes;
    type Error = !;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetFullReq, input_loop) = self;
        Ok(InputGetFullRes {
            stats: input_loop.inputs.each_ref().map(|v| {
                let data = v.get();
                let avg = if data.count == 0 {
                    data.previous_value
                } else {
                    (data.sum / i32::from(data.count)) as i16
                };
                v.set(InputData {
                    previous_value: avg,
                    ..InputData::default()
                });
                data.into()
            }),
        })
    }
}
impl<
    I: Deref<Target = InputLoop<NOM, DENOM>>,
    T: Timer<Board, u64, NOM, DENOM>,
    Board: ?Sized,
    const NOM: u32,
    const DENOM: u32,
> HandleMessage for (&InputGetThresholdTimesReq, &T, I, PhantomData<Board>)
{
    type Response = InputGetThresholdTimesRes;
    type Error = !;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetThresholdTimesReq, timer, input_loop, PhantomData) = self;
        let now = timer.now().ticks().into();
        let inputs = input_loop
            .thresholds
            .each_ref()
            .map(|threshold| InputThresholdTimes {
                last_high: threshold
                    .get()
                    .last_above_threshold_debounced
                    .ticks()
                    .into(),
                last_low: threshold
                    .get()
                    .last_below_threshold_debounced
                    .ticks()
                    .into(),
            });
        Ok(InputGetThresholdTimesRes { now, inputs })
    }
}
impl<I: Deref<Target = InputLoop<NOM, DENOM>>, const NOM: u32, const DENOM: u32> HandleMessage
    for (&InputGetThresholdStatesReq, I)
{
    type Response = InputGetThresholdStatesRes;
    type Error = !;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetThresholdStatesReq, input_loop) = self;
        let mut above = 0u16;
        let mut below = 0u16;
        for (i, threshold) in input_loop.thresholds.iter().enumerate() {
            let threshold = threshold.get();
            if threshold.last_above_threshold_debounced >= threshold.last_below_threshold_debounced
            {
                above |= 1 << i;
            }
            if threshold.last_below_threshold_debounced >= threshold.last_above_threshold_debounced
            {
                below |= 1 << i;
            }
        }
        Ok(InputGetThresholdStatesRes {
            above: above.into(),
            below: below.into(),
        })
    }
}
impl<const NOM: u32, const DENOM: u32> InputLoop<NOM, DENOM> {
    pub fn new(now: Instant<u64, NOM, DENOM>) -> Self {
        Self {
            inputs: [const { Cell::new(InputData::new()) }; 16],
            thresholds: array::from_fn(|_| Cell::new(ThresholdData::new(now))),
        }
    }
    async fn wait_read0<Board: ?Sized, I: Input<Board>>(input: &mut I) -> Result<u16, I::Error> {
        loop {
            match nb_await!(input.read_last()) {
                Ok(v) => return Ok(v),
                Err(InputError::RecoverableError) => {
                    nb_await!(input.start_read0())?;
                    continue;
                }
                Err(InputError::UnrecoverableError(e)) => return Err(e),
            }
        }
    }
    async fn wait_read1<Board: ?Sized, I: Input<Board>>(input: &mut I) -> Result<u16, I::Error> {
        loop {
            match nb_await!(input.read_last()) {
                Ok(v) => return Ok(v),
                Err(InputError::RecoverableError) => {
                    nb_await!(input.start_read1())?;
                    continue;
                }
                Err(InputError::UnrecoverableError(e)) => return Err(e),
            }
        }
    }

    /// Run the input loop, which continuously reads the inputs and updates the input data and threshold data.
    pub async fn run<Board: ?Sized, I: Input<Board>, NVM: NonvolatileStorage<Board>>(
        &self,
        input: &mut I,
        timer: &impl Timer<Board, u64, NOM, DENOM>,
        nvm: &Nvm<NVM, Board>,
    ) -> Result<!, Either<I::Error, NVM::Error>> {
        const GRAY_CODE_INCREMENT: [u8; 8] = [1, 3, 6, 2, 0, 4, 7, 5];
        let mut i = 0;
        loop {
            nb_await!(input.start_read0()).map_err(Either::Left)?;
            // make sure to at least one guarantied yield per iteration of the loop to prevent starvation of other tasks
            yield_now().await;
            let v0 = Self::wait_read0(input).await.map_err(Either::Left)?;
            let now0 = timer.now();
            // start next read as early as possible
            nb_await!(input.start_read1()).map_err(Either::Left)?;
            let calibration = nvm.get().calibrations[i];
            let v0 = calibration.apply(v0);
            self.inputs[i].update(|data| data.update(v0));
            let threshold = nvm.get().thresholds[i];
            self.thresholds[i].update(|t| t.update(v0, now0, &threshold));

            let v1 = Self::wait_read1(input).await.map_err(Either::Left)?;
            let now1 = timer.now();
            // select next input as early as possible
            let i_tmp = GRAY_CODE_INCREMENT[i] as usize;
            nb_await!(input.select0(i_tmp & 0x1 != 0)).map_err(Either::Left)?;
            nb_await!(input.select1(i_tmp & 0x2 != 0)).map_err(Either::Left)?;
            nb_await!(input.select2(i_tmp & 0x4 != 0)).map_err(Either::Left)?;
            let calibration = nvm.get().calibrations[i + 8];
            let v1 = calibration.apply(v1);
            self.inputs[i + 8].update(|data| data.update(v1));
            let threshold = nvm.get().thresholds[i + 8];
            self.thresholds[i + 8].update(|t| t.update(v1, now1, &threshold));

            i = i_tmp;
            // let inputs settle
            timer
                .wait_until(now1 + Duration::<u64, NOM, DENOM>::micros(3))
                .await;
        }
    }
}
