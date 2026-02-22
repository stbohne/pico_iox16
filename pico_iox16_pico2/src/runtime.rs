use core::convert::Infallible;

use cortex_m::prelude::_embedded_hal_PwmPin as _;
use defmt::info;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_0_2::PwmPin;
use fugit::Instant;
use pico_iox16_firmware::runtime::{Read, ReadError, Write};
use rounded_div::RoundedDiv as _;
use rp235x_hal::{
    Timer,
    pwm::{AnySlice, Channel, ChannelId, FreeRunning, Slice, SliceId},
    timer::CopyableTimer0,
    uart::{Enabled, UartDevice, UartPeripheral, ValidUartPinout},
};

pub enum Board {}

pub struct Timer0(pub Timer<CopyableTimer0>);
impl pico_iox16_firmware::runtime::Timer<Board, u64, 1, 1_000_000> for Timer0 {
    fn now(&self) -> Instant<u64, 1, 1_000_000> {
        self.0.get_counter()
    }
}

pub struct Uart<D: UartDevice, P: ValidUartPinout<D>>(
    pub UartPeripheral<Enabled, D, P>,
    Option<rp235x_hal::uart::ReadErrorType>,
);
impl<D: UartDevice, P: ValidUartPinout<D>> Uart<D, P> {
    pub fn new(peripheral: UartPeripheral<Enabled, D, P>) -> Self {
        Self(peripheral, None)
    }
}
impl<D: UartDevice, P: ValidUartPinout<D>> Read<Board> for Uart<D, P> {
    type Error = Infallible;

    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ReadError<Self::Error>> {
        if let Some(e) = self.1.take() {
            info!("UART read error: {:?}", e);
            return Err(nb::Error::Other(ReadError::RecoverableError));
        }
        match self.0.read_raw(buf) {
            Ok(n) => Ok(n),
            Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(e)) => {
                if e.discarded.is_empty() {
                    info!("UART read error: {:?}", e.err_type);
                    Err(nb::Error::Other(ReadError::RecoverableError))
                } else {
                    self.1 = Some(e.err_type);
                    Ok(e.discarded.len())
                }
            }
        }
    }
}
impl<D: UartDevice, P: ValidUartPinout<D>> Write<Board> for Uart<D, P> {
    type Error = Infallible;

    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, Self::Error> {
        let len = buf.len();
        self.0
            .write_raw(buf)
            .map(|remaining| len - remaining.len())
            .map_err(|nb::Error::WouldBlock| nb::Error::WouldBlock)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.0.uart_is_busy() {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl<S: SliceId> pico_iox16_firmware::output::Pwm<Board> for Slice<S, FreeRunning> {
    type Error = Infallible;
    type ChannelA = Channel<Self, rp235x_hal::pwm::A>;
    type ChannelB = Channel<Self, rp235x_hal::pwm::B>;
    fn get_frequency(&self) -> Result<u16, Self::Error> {
        let sys_clk_hz = 150_000_000;
        let int_frac = (u32::from(self.get_div_int()) << 4) | u32::from(self.get_div_frac());
        Ok((sys_clk_hz * 0x10).rounded_div(int_frac * 0x8000) as u16)
    }
    fn channel_a(&self) -> &Self::ChannelA {
        &self.channel_a
    }
    fn channel_b(&self) -> &Self::ChannelB {
        &self.channel_b
    }
    fn set_frequency(&mut self, frequency: u16) -> Result<(), Self::Error> {
        let sys_clk_hz = 150_000_000;
        let frequency = frequency.clamp(10, 50_000);
        let int_frac = (sys_clk_hz * 0x10).rounded_div(u32::from(frequency) * 0x8000);
        let top = (sys_clk_hz * 0x10).rounded_div(u32::from(frequency) * int_frac);
        self.set_div_int((int_frac >> 4).try_into().unwrap());
        self.set_div_frac((int_frac & 0x0F).try_into().unwrap());
        self.set_top(top.try_into().unwrap());
        Ok(())
    }
    fn channel_a_mut(&mut self) -> &mut Self::ChannelA {
        &mut self.channel_a
    }
    fn channel_b_mut(&mut self) -> &mut Self::ChannelB {
        &mut self.channel_b
    }
}
impl<S: AnySlice, C: ChannelId> pico_iox16_firmware::output::PwmChannel<Board> for Channel<S, C> {
    type Error = Infallible;
    fn max_duty_cycle(&self) -> Result<u16, Self::Error> {
        Ok(SetDutyCycle::max_duty_cycle(self))
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        SetDutyCycle::set_duty_cycle(
            self, duty
        )
    }
    fn get_duty_cycle(&self) -> Result<u16, Self::Error> {
        Ok(self.get_duty())
    }
}
