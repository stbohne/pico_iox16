use core::convert::Infallible;

use embedded_hal::digital::OutputPin;
use rp235x_hal::{
    Adc,
    adc::{AdcPin, Error},
    gpio::{
        AnyPin, FunctionNull, FunctionSio, Pin, PinId, PullNone, PullType, SioOutput, ValidFunction,
    },
};

use pico_iox16_firmware::input::InputError;

use crate::runtime::Board;

pub struct Input<Sel0: PinId, Sel1: PinId, Sel2: PinId, Pin0: AnyPin, Pin1: AnyPin> {
    sel0: Pin<Sel0, FunctionSio<SioOutput>, PullNone>,
    sel1: Pin<Sel1, FunctionSio<SioOutput>, PullNone>,
    sel2: Pin<Sel2, FunctionSio<SioOutput>, PullNone>,
    adc: Adc,
    pin0: AdcPin<Pin0>,
    pin1: AdcPin<Pin1>,
}
impl<
    Sel0: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Sel1: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Sel2: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Pin0: AnyPin,
    Pin1: AnyPin,
> pico_iox16_firmware::input::Input<Board> for Input<Sel0, Sel1, Sel2, Pin0, Pin1>
{
    type Error = Infallible;
    fn select0(&mut self, value: bool) -> nb::Result<(), Self::Error> {
        self.sel0.set_state(value.into()).map_err(nb::Error::Other)
    }
    fn select1(&mut self, value: bool) -> nb::Result<(), Self::Error> {
        self.sel1.set_state(value.into()).map_err(nb::Error::Other)
    }
    fn select2(&mut self, value: bool) -> nb::Result<(), Self::Error> {
        self.sel2.set_state(value.into()).map_err(nb::Error::Other)
    }
    fn start_read0(&mut self) -> nb::Result<(), Self::Error> {
        self.adc.start_oneshot(&mut self.pin0)
    }

    fn start_read1(&mut self) -> nb::Result<(), Self::Error> {
        self.adc.start_oneshot(&mut self.pin1)
    }

    fn read_last(&mut self) -> nb::Result<u16, InputError<Self::Error>> {
        if self.adc.is_ready() {
            match self.adc.read_single() {
                Ok(v) => Ok(v),
                Err(Error::ConversionFailed) => Err(nb::Error::Other(InputError::RecoverableError)),
            }
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
impl<
    Sel0: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Sel1: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Sel2: PinId + ValidFunction<FunctionSio<SioOutput>>,
    Pin0: AnyPin,
    Pin1: AnyPin,
> Input<Sel0, Sel1, Sel2, Pin0, Pin1>
{
    pub fn new<Pull0: PullType, Pull1: PullType, Pull2: PullType>(
        sel0: Pin<Sel0, FunctionNull, Pull0>,
        sel1: Pin<Sel1, FunctionNull, Pull1>,
        sel2: Pin<Sel2, FunctionNull, Pull2>,
        adc: Adc,
        pin0: AdcPin<Pin0>,
        pin1: AdcPin<Pin1>,
    ) -> Self {
        let sel0 = sel0
            .into_push_pull_output_in_state(false.into())
            .into_pull_type::<PullNone>();
        let sel1 = sel1
            .into_push_pull_output_in_state(false.into())
            .into_pull_type::<PullNone>();
        let sel2 = sel2
            .into_push_pull_output_in_state(false.into())
            .into_pull_type::<PullNone>();
        Self {
            sel0,
            sel1,
            sel2,
            adc,
            pin0,
            pin1,
        }
    }
}
