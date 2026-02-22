use core::{cell::Cell, convert::Infallible, marker::PhantomData, ops::Deref};

use pico_iox16_protocol::{
    ConfigGetReq, ConfigGetRes, ConfigSetReq, ConfigSetRes, InputGetCalibrationsReq,
    InputGetCalibrationsRes, InputGetThresholdsReq, InputGetThresholdsRes, InputSetCalibrationsReq,
    InputSetCalibrationsRes, InputSetThresholdsReq, InputSetThresholdsRes,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::{HandleMessage, nb_await};

#[derive(Debug, Clone, Copy, IntoBytes, TryFromBytes, Immutable)]
#[repr(C)]
pub(crate) struct Config {
    pub address: u16,
    pub _padding: [u8; 2],
}
impl From<pico_iox16_protocol::Config> for Config {
    fn from(value: pico_iox16_protocol::Config) -> Self {
        Self {
            address: value.address.into(),
            _padding: [0; 2],
        }
    }
}
impl From<Config> for pico_iox16_protocol::Config {
    fn from(value: Config) -> Self {
        Self {
            address: value.address.into(),
            reserved: [0; 2],
        }
    }
}
impl<I: Deref<Target = Nvm<NVM, Board>>, NVM: NonvolatileStorage<Board>, Board: ?Sized>
    HandleMessage for (&InputSetThresholdsReq, I, PhantomData<(NVM, Board)>)
{
    type Response = InputSetThresholdsRes;
    type Error = <NVM as NonvolatileStorage<Board>>::Error;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputSetThresholdsReq(trips), storage, PhantomData) = self;
        let new_data = NonvolatileData {
            thresholds: trips.each_ref().map(|trip| (*trip).into()),
            ..storage.get()
        };
        storage.set(&new_data).await?;
        Ok(InputSetThresholdsRes)
    }
}
impl<I: Deref<Target = Nvm<NVM, Board>>, NVM, Board: ?Sized> HandleMessage
    for (&InputGetThresholdsReq, I, PhantomData<(NVM, Board)>)
{
    type Response = InputGetThresholdsRes;
    type Error = Infallible;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetThresholdsReq, storage, PhantomData) = self;
        Ok(InputGetThresholdsRes(
            storage
                .get()
                .thresholds
                .each_ref()
                .map(|trip| (*trip).into()),
        ))
    }
}

#[derive(Debug, Clone, Copy, IntoBytes, TryFromBytes, Immutable)]
#[repr(C)]
pub(crate) struct Calibration {
    /// The value to multiply the raw input by.
    /// But stored XOR 0xFFFE so that the default is 1.
    pub multiply: i16,
    /// The value to divide by after multiplication.
    /// But stored XOR 0xFFFE so that the default is 1.
    pub divide: i16,
    /// The value to add to the raw input after division.
    /// But stored XOR 0xFFFF so that the default is 0.
    pub add: i16,
    /// The minimum value after addition.
    /// But stored XOR 0xFFFF so that the default is 0.
    pub min: i16,
    /// The maximum value after addition.
    /// Stored directly, so the default is 0xFFFF.
    pub max: i16,
}
impl Calibration {
    pub fn apply(&self, value: u16) -> i16 {
        let value =
            (value as i32 * (self.multiply) as i32) / (self.divide) as i32 + (self.add) as i32;
        value.clamp((self.min) as i32, self.max as i32) as i16
    }
}
impl From<pico_iox16_protocol::InputCalibration> for Calibration {
    fn from(value: pico_iox16_protocol::InputCalibration) -> Self {
        Self {
            multiply: i16::from(value.multiply),
            divide: i16::from(value.divide),
            add: i16::from(value.add),
            min: i16::from(value.min),
            max: i16::from(value.max),
        }
    }
}
impl From<Calibration> for pico_iox16_protocol::InputCalibration {
    fn from(value: Calibration) -> Self {
        Self {
            multiply: (value.multiply).into(),
            divide: (value.divide).into(),
            add: (value.add).into(),
            min: (value.min).into(),
            max: (value.max).into(),
        }
    }
}
impl<I: Deref<Target = Nvm<NVM, Board>>, NVM: NonvolatileStorage<Board>, Board: ?Sized>
    HandleMessage for (&InputSetCalibrationsReq, I, PhantomData<(NVM, Board)>)
{
    type Response = InputSetCalibrationsRes;
    type Error = <NVM as NonvolatileStorage<Board>>::Error;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputSetCalibrationsReq(calibrations), storage, PhantomData) = self;
        let new_data = NonvolatileData {
            calibrations: calibrations.each_ref().map(|cal| (*cal).into()),
            ..storage.get()
        };
        storage.set(&new_data).await?;
        Ok(InputSetCalibrationsRes)
    }
}
impl<I: Deref<Target = Nvm<NVM, Board>>, NVM, Board: ?Sized> HandleMessage
    for (&InputGetCalibrationsReq, I, PhantomData<(NVM, Board)>)
{
    type Response = InputGetCalibrationsRes;
    type Error = Infallible;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (InputGetCalibrationsReq, storage, PhantomData) = self;
        Ok(InputGetCalibrationsRes(
            storage
                .get()
                .calibrations
                .each_ref()
                .map(|cal| (*cal).into()),
        ))
    }
}

#[derive(Debug, Clone, Copy, IntoBytes, TryFromBytes, Immutable)]
#[repr(C)]
pub(crate) struct Threshold {
    /// The value above which the input is considered above threshold
    pub threshold_high: i16,
    /// The value below which the input is considered below threshold
    pub threshold_low: i16,
    /// The time duration for which the input must remain above or below the threshold to be considered debounced
    pub debounce_time_us: u32,
    /// The number of consecutive readings minus one above or below the threshold required for debouncing
    pub debounce_count: u16,
    #[doc(hidden)]
    pub _padding: [u8; 2],
}
impl From<pico_iox16_protocol::InputThreshold> for Threshold {
    fn from(value: pico_iox16_protocol::InputThreshold) -> Self {
        Self {
            threshold_high: value.threshold_high.into(),
            threshold_low: value.threshold_low.into(),
            debounce_time_us: value.debounce_time_us.into(),
            debounce_count: value.debounce_count.into(),
            _padding: [0xFF; 2],
        }
    }
}
impl From<Threshold> for pico_iox16_protocol::InputThreshold {
    fn from(value: Threshold) -> Self {
        Self {
            threshold_high: value.threshold_high.into(),
            threshold_low: value.threshold_low.into(),
            debounce_time_us: value.debounce_time_us.into(),
            debounce_count: value.debounce_count.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, IntoBytes, TryFromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub(crate) struct NonvolatileData {
    pub config: Config,
    pub calibrations: [Calibration; 16],
    pub thresholds: [Threshold; 16],
}

pub trait NonvolatileStorage<Board: ?Sized> {
    type Error;
    fn read(&self) -> nb::Result<[u8; 4096], Self::Error>;
    fn write(&self, data: &[u8; 4096]) -> nb::Result<(), Self::Error>;
}

pub const fn default_nonvolatile_data() -> [u8; 4096] {
    let default = NonvolatileData {
        config: Config {
            address: 0xFFFF,
            _padding: [0xFF; 2],
        },
        calibrations: [Calibration {
            multiply: 1,
            divide: 1,
            add: 0,
            min: i16::MIN,
            max: i16::MAX,
        }; 16],
        thresholds: [Threshold {
            threshold_high: i16::MAX,
            threshold_low: i16::MIN,
            debounce_time_us: 0,
            debounce_count: 0,
            _padding: [0xFF; 2],
        }; 16],
    };
    let mut data = [0xFF; 4096];
    let mut i = 0;
    while i < core::mem::size_of::<NonvolatileData>() {
        data[i] = unsafe { core::ptr::addr_of!(default).cast::<u8>().add(i).read() };
        i += 1;
    }
    data
}

pub struct Nvm<NVM, Board: ?Sized>(Cell<NonvolatileData>, NVM, PhantomData<Board>);
impl<NVM, Board: ?Sized> Nvm<NVM, Board> {
    pub(crate) fn get(&self) -> NonvolatileData {
        self.0.get()
    }
}
impl<NVM: NonvolatileStorage<Board>, Board: ?Sized> Nvm<NVM, Board> {
    pub(crate) async fn new(nvm: NVM) -> Result<Self, NVM::Error> {
        let data = nb_await!(nvm.read())?;
        let data = NonvolatileData::try_ref_from_prefix(&data).unwrap().0;
        Ok(Self(Cell::new(*data), nvm, PhantomData))
    }
    pub(crate) async fn set(&self, data: &NonvolatileData) -> Result<(), NVM::Error> {
        self.0.set(*data);
        let mut buf = [0xFF; 4096];
        data.write_to_prefix(&mut buf).unwrap();
        nb_await!(self.1.write(&buf))?;
        Ok(())
    }
}

impl<O: Deref<Target = Nvm<NVM, Board>>, NVM: NonvolatileStorage<Board>, Board: ?Sized>
    HandleMessage for (&ConfigSetReq, O, PhantomData<(NVM, Board)>)
{
    type Response = ConfigSetRes;
    type Error = <NVM as NonvolatileStorage<Board>>::Error;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (ConfigSetReq(config), storage, PhantomData) = self;
        let new_data = NonvolatileData {
            config: (*config).into(),
            ..storage.get()
        };
        storage.set(&new_data).await?;
        Ok(ConfigSetRes)
    }
}
impl<O: Deref<Target = Nvm<NVM, Board>>, NVM, Board: ?Sized> HandleMessage
    for (&ConfigGetReq, O, PhantomData<(NVM, Board)>)
{
    type Response = ConfigGetRes;
    type Error = Infallible;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        let (ConfigGetReq, storage, PhantomData) = self;
        Ok(ConfigGetRes(storage.get().config.into()))
    }
}
