use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use pico_iox16_protocol::{OutputGetReq, OutputGetRes, OutputGroup, OutputSetReq, OutputSetRes};
use rounded_div::RoundedDiv as _;

use crate::HandleMessage;

/// PWM abstraction
pub trait Pwm<Board: ?Sized> {
    type Error;
    type ChannelA: PwmChannel<Board, Error = Self::Error>;
    type ChannelB: PwmChannel<Board, Error = Self::Error>;
    fn get_frequency(&self) -> Result<u16, Self::Error>;
    fn channel_a(&self) -> &Self::ChannelA;
    fn channel_b(&self) -> &Self::ChannelB;
    fn set_frequency(&mut self, frequency: u16) -> Result<(), Self::Error>;
    fn channel_a_mut(&mut self) -> &mut Self::ChannelA;
    fn channel_b_mut(&mut self) -> &mut Self::ChannelB;
}

/// PWM channel abstraction
pub trait PwmChannel<Board: ?Sized> {
    type Error;
    fn max_duty_cycle(&self) -> Result<u16, Self::Error>;
    fn get_duty_cycle(&self) -> Result<u16, Self::Error>;
    fn set_duty_cycle(&mut self, duty_cycle: u16) -> Result<(), Self::Error>;
}

/// Abstraction for obtaining the PWMs for the outputs
pub trait Output<Board: ?Sized> {
    type Error;
    type Pwm0: Pwm<Board, Error = Self::Error>;
    fn pwm0(&self) -> &Self::Pwm0;
    fn pwm0_mut(&mut self) -> &mut Self::Pwm0;
    type Pwm1: Pwm<Board, Error = Self::Error>;
    fn pwm1(&self) -> &Self::Pwm1;
    fn pwm1_mut(&mut self) -> &mut Self::Pwm1;
    type Pwm2: Pwm<Board, Error = Self::Error>;
    fn pwm2(&self) -> &Self::Pwm2;
    fn pwm2_mut(&mut self) -> &mut Self::Pwm2;
    type Pwm3: Pwm<Board, Error = Self::Error>;
    fn pwm3(&self) -> &Self::Pwm3;
    fn pwm3_mut(&mut self) -> &mut Self::Pwm3;
    type Pwm4: Pwm<Board, Error = Self::Error>;
    fn pwm4(&self) -> &Self::Pwm4;
    fn pwm4_mut(&mut self) -> &mut Self::Pwm4;
    type Pwm5: Pwm<Board, Error = Self::Error>;
    fn pwm5(&self) -> &Self::Pwm5;
    fn pwm5_mut(&mut self) -> &mut Self::Pwm5;
    type Pwm6: Pwm<Board, Error = Self::Error>;
    fn pwm6(&self) -> &Self::Pwm6;
    fn pwm6_mut(&mut self) -> &mut Self::Pwm6;
    type Pwm7: Pwm<Board, Error = Self::Error>;
    fn pwm7(&self) -> &Self::Pwm7;
    fn pwm7_mut(&mut self) -> &mut Self::Pwm7;
}

impl<O: DerefMut<Target: Output<Board>>, Board: ?Sized> HandleMessage
    for (&OutputSetReq, O, PhantomData<Board>)
{
    type Response = OutputSetRes;
    type Error = <O::Target as Output<Board>>::Error;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        fn handle_group<P: Pwm<Board>, Board: ?Sized>(
            pwm: &mut P,
            group: &OutputGroup,
        ) -> Result<(), P::Error> {
            let frequency = group.frequency.get().clamp(10, 50_000);
            pwm.set_frequency(frequency)?;
            let duty_cycle_a = group.duty_cycle[0].get().clamp(0, 0x8000);
            let duty_cycle_a = (u32::from(duty_cycle_a) * 0x8000)
                .rounded_div(pwm.channel_a().max_duty_cycle()? as u32)
                as u16;
            let duty_cycle_b = group.duty_cycle[1].get().clamp(0, 0x8000);
            let duty_cycle_b = (u32::from(duty_cycle_b) * 0x8000)
                .rounded_div(pwm.channel_b().max_duty_cycle()? as u32)
                as u16;
            pwm.channel_a_mut().set_duty_cycle(duty_cycle_a)?;
            pwm.channel_b_mut().set_duty_cycle(duty_cycle_b)?;
            Ok(())
        }

        let (req, mut output, _) = self;
        handle_group(output.pwm0_mut(), &req.0[0])?;
        handle_group(output.pwm1_mut(), &req.0[1])?;
        handle_group(output.pwm2_mut(), &req.0[2])?;
        handle_group(output.pwm3_mut(), &req.0[3])?;
        handle_group(output.pwm4_mut(), &req.0[4])?;
        handle_group(output.pwm5_mut(), &req.0[5])?;
        handle_group(output.pwm6_mut(), &req.0[6])?;
        handle_group(output.pwm7_mut(), &req.0[7])?;
        Ok(OutputSetRes)
    }
}

impl<O: Deref<Target: Output<Board>>, Board: ?Sized> HandleMessage
    for (&OutputGetReq, O, PhantomData<Board>)
{
    type Response = OutputGetRes;
    type Error = <O::Target as Output<Board>>::Error;
    async fn handle(self) -> Result<Self::Response, Self::Error> {
        fn handle_group<P: Pwm<Board>, Board: ?Sized>(pwm: &P) -> Result<OutputGroup, P::Error> {
            let frequency = pwm.get_frequency()?;
            let duty_cycle_a = pwm.channel_a().get_duty_cycle()?;
            let duty_cycle_b = pwm.channel_b().get_duty_cycle()?;
            Ok(OutputGroup {
                duty_cycle: [duty_cycle_a.into(), duty_cycle_b.into()],
                frequency: frequency.into(),
            })
        }

        let (OutputGetReq, output, _) = self;
        Ok(OutputGetRes([
            handle_group(output.pwm0())?,
            handle_group(output.pwm1())?,
            handle_group(output.pwm2())?,
            handle_group(output.pwm3())?,
            handle_group(output.pwm4())?,
            handle_group(output.pwm5())?,
            handle_group(output.pwm6())?,
            handle_group(output.pwm7())?,
        ]))
    }
}
