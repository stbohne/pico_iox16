use core::convert::Infallible;

use rp235x_hal::{
    gpio::{
        FunctionNull, Pin, PullDown,
        bank0::{
            Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9, Gpio10, Gpio11,
            Gpio12, Gpio13, Gpio14, Gpio15,
        },
    },
    pwm::{FreeRunning, Slice, Slices},
};

use crate::runtime::Board;

pub struct Output {
    slices: Slices,
}
impl Output {
    pub fn new(
        OutputPins {
            gpio0,
            gpio1,
            gpio2,
            gpio3,
            gpio4,
            gpio5,
            gpio6,
            gpio7,
            gpio8,
            gpio9,
            gpio10,
            gpio11,
            gpio12,
            gpio13,
            gpio14,
            gpio15,
        }: OutputPins,
        mut slices: Slices,
    ) -> Self {
        slices.pwm0.channel_a.output_to(gpio0);
        slices.pwm0.channel_b.output_to(gpio1);
        slices.pwm1.channel_a.output_to(gpio2);
        slices.pwm1.channel_b.output_to(gpio3);
        slices.pwm2.channel_a.output_to(gpio4);
        slices.pwm2.channel_b.output_to(gpio5);
        slices.pwm3.channel_a.output_to(gpio6);
        slices.pwm3.channel_b.output_to(gpio7);
        slices.pwm4.channel_a.output_to(gpio8);
        slices.pwm4.channel_b.output_to(gpio9);
        slices.pwm5.channel_a.output_to(gpio10);
        slices.pwm5.channel_b.output_to(gpio11);
        slices.pwm6.channel_a.output_to(gpio12);
        slices.pwm6.channel_b.output_to(gpio13);
        slices.pwm7.channel_a.output_to(gpio14);
        slices.pwm7.channel_b.output_to(gpio15);
        slices.enable_simultaneous(0xFF);
        Self { slices }
    }
}
impl pico_iox16_firmware::output::Output<Board> for Output {
    type Error = Infallible;
    type Pwm0 = Slice<rp235x_hal::pwm::Pwm0, FreeRunning>;
    fn pwm0(&self) -> &Self::Pwm0 {
        &self.slices.pwm0
    }
    fn pwm0_mut(&mut self) -> &mut Self::Pwm0 {
        &mut self.slices.pwm0
    }
    type Pwm1 = Slice<rp235x_hal::pwm::Pwm1, FreeRunning>;
    fn pwm1(&self) -> &Self::Pwm1 {
        &self.slices.pwm1
    }
    fn pwm1_mut(&mut self) -> &mut Self::Pwm1 {
        &mut self.slices.pwm1
    }
    type Pwm2 = Slice<rp235x_hal::pwm::Pwm2, FreeRunning>;
    fn pwm2(&self) -> &Self::Pwm2 {
        &self.slices.pwm2
    }
    fn pwm2_mut(&mut self) -> &mut Self::Pwm2 {
        &mut self.slices.pwm2
    }
    type Pwm3 = Slice<rp235x_hal::pwm::Pwm3, FreeRunning>;
    fn pwm3(&self) -> &Self::Pwm3 {
        &self.slices.pwm3
    }
    fn pwm3_mut(&mut self) -> &mut Self::Pwm3 {
        &mut self.slices.pwm3
    }
    type Pwm4 = Slice<rp235x_hal::pwm::Pwm4, FreeRunning>;
    fn pwm4(&self) -> &Self::Pwm4 {
        &self.slices.pwm4
    }
    fn pwm4_mut(&mut self) -> &mut Self::Pwm4 {
        &mut self.slices.pwm4
    }
    type Pwm5 = Slice<rp235x_hal::pwm::Pwm5, FreeRunning>;
    fn pwm5(&self) -> &Self::Pwm5 {
        &self.slices.pwm5
    }
    fn pwm5_mut(&mut self) -> &mut Self::Pwm5 {
        &mut self.slices.pwm5
    }
    type Pwm6 = Slice<rp235x_hal::pwm::Pwm6, FreeRunning>;
    fn pwm6(&self) -> &Self::Pwm6 {
        &self.slices.pwm6
    }
    fn pwm6_mut(&mut self) -> &mut Self::Pwm6 {
        &mut self.slices.pwm6
    }
    type Pwm7 = Slice<rp235x_hal::pwm::Pwm7, FreeRunning>;
    fn pwm7(&self) -> &Self::Pwm7 {
        &self.slices.pwm7
    }
    fn pwm7_mut(&mut self) -> &mut Self::Pwm7 {
        &mut self.slices.pwm7
    }
}

pub struct OutputPins {
    pub gpio0: Pin<Gpio0, FunctionNull, PullDown>,
    pub gpio1: Pin<Gpio1, FunctionNull, PullDown>,
    pub gpio2: Pin<Gpio2, FunctionNull, PullDown>,
    pub gpio3: Pin<Gpio3, FunctionNull, PullDown>,
    pub gpio4: Pin<Gpio4, FunctionNull, PullDown>,
    pub gpio5: Pin<Gpio5, FunctionNull, PullDown>,
    pub gpio6: Pin<Gpio6, FunctionNull, PullDown>,
    pub gpio7: Pin<Gpio7, FunctionNull, PullDown>,
    pub gpio8: Pin<Gpio8, FunctionNull, PullDown>,
    pub gpio9: Pin<Gpio9, FunctionNull, PullDown>,
    pub gpio10: Pin<Gpio10, FunctionNull, PullDown>,
    pub gpio11: Pin<Gpio11, FunctionNull, PullDown>,
    pub gpio12: Pin<Gpio12, FunctionNull, PullDown>,
    pub gpio13: Pin<Gpio13, FunctionNull, PullDown>,
    pub gpio14: Pin<Gpio14, FunctionNull, PullDown>,
    pub gpio15: Pin<Gpio15, FunctionNull, PullDown>,
}
