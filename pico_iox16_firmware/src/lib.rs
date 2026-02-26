#![no_std]
#![feature(never_type)]

pub mod input;
pub mod nvm;
pub mod output;
pub mod runtime;

use core::{marker::PhantomData, ops::Sub, pin::pin};
use defmt::info;
use embedded_hal::digital::OutputPin;
use fugit::{Duration, Instant};
use futures::future::{Either, select};
use pico_iox16_protocol::{
    CheckReq, CheckRes, Command, ConfigGetReq, InfoGetReq, InfoGetRes, InputGetReq, Message, OutputGetReq, RebootReq, Request, slave_next
};
use runtime::{Read, Timer, Write};
use zerocopy::{Immutable, IntoBytes};

use crate::{
    input::InputLoop,
    runtime::{Elapsed as _, ReadError, System, WaitFor as _, yield_now},
};

trait HandleMessage {
    type Response;
    type Error;
    fn handle(self) -> impl Future<Output = Result<Self::Response, Self::Error>>;
}

#[derive(Debug, thiserror::Error, defmt::Format)]
pub enum MainLoopError<ReadError, WriteError, IoSendError, OutputError, InputError, NvmError> {
    // IO read error
    Read(ReadError),
    // IO write error
    Write(WriteError),
    // Error setting TX gpio pin
    IoSend(IoSendError),
    // Error setting PWM
    Output(OutputError),
    // ADC read error
    Input(InputError),
    // Flash read or write error
    Nvm(NvmError),
}
impl<A, B, C, D, E, F> MainLoopError<A, B, C, D, E, F> {
    fn convert<G: From<A>, H: From<B>, I: From<C>, J: From<D>, K: From<E>, L: From<F>>(
        self,
    ) -> MainLoopError<G, H, I, J, K, L> {
        match self {
            MainLoopError::Read(a) => MainLoopError::Read(a.into()),
            MainLoopError::Write(b) => MainLoopError::Write(b.into()),
            MainLoopError::IoSend(c) => MainLoopError::IoSend(c.into()),
            MainLoopError::Output(d) => MainLoopError::Output(d.into()),
            MainLoopError::Input(e) => MainLoopError::Input(e.into()),
            MainLoopError::Nvm(f) => MainLoopError::Nvm(f.into()),
        }
    }
}
macro_rules! error_coerce {
    ($err:expr) => {
        match $err {
            #[allow(unreachable_code)]
            MainLoopError::Read(a) => MainLoopError::Read(a),
            #[allow(unreachable_code)]
            MainLoopError::Write(b) => MainLoopError::Write(b),
            #[allow(unreachable_code)]
            MainLoopError::IoSend(c) => MainLoopError::IoSend(c),
            #[allow(unreachable_code)]
            MainLoopError::Output(d) => MainLoopError::Output(d),
            #[allow(unreachable_code)]
            MainLoopError::Input(e) => MainLoopError::Input(e),
            #[allow(unreachable_code)]
            MainLoopError::Nvm(f) => MainLoopError::Nvm(f),
        }
    };
}

pub struct MainLoop<const NOM: u32, const DENOM: u32> {
    started: Instant<u64, NOM, DENOM>,
    input_loop: InputLoop<NOM, DENOM>,
}
impl<const NOM: u32, const DENOM: u32> MainLoop<NOM, DENOM> {
    pub fn new<Board: ?Sized>(timer: &impl Timer<Board, u64, NOM, DENOM>) -> Self {
        let now = timer.now();
        Self {
            started: now,
            input_loop: InputLoop::new(now),
        }
    }

    async fn write_all_bytes<
        Board: ?Sized,
        IO: Write<Board>,
        IoSend: OutputPin,
        P: IntoBytes + Immutable + ?Sized,
    >(
        io: &mut IO,
        io_send: &mut IoSend,
        payload: &P,
    ) -> Result<(), MainLoopError<!, IO::Error, IoSend::Error, !, !, !>> {
        let mut bytes = payload.as_bytes();
        io_send.set_high().map_err(MainLoopError::IoSend)?;
        {
            let mut preamble = 2;
            while preamble > 0 {
                let written = nb_await!(io.write(&[0xFF])).map_err(MainLoopError::Write)?;
                preamble -= written;
            }
        }
        while !bytes.is_empty() {
            let written = nb_await!(io.write(bytes)).map_err(MainLoopError::Write)?;
            assert!(written > 0);
            bytes = &bytes[written..];
        }
        nb_await!(io.flush()).map_err(MainLoopError::Write)?;
        io_send.set_low().map_err(MainLoopError::IoSend)?;
        Ok(())
    }

    /// Continuously read requests from the IO, handle them and write the responses back to the IO.
    async fn run<
        Board: ?Sized,
        IO: Read<Board> + Write<Board>,
        S: OutputPin,
        T: Timer<Board, u64, NOM, DENOM>,
        O: output::Output<Board>,
        NVM: nvm::NonvolatileStorage<Board>,
    >(
        &self,
        io: &mut IO,
        io_send: &mut S,
        timer: &T,
        output: &mut O,
        nvm: &nvm::Nvm<NVM, Board>,
        input_loop: &InputLoop<NOM, DENOM>,
        system: &impl System<Board>,
    ) -> Result<
        !,
        MainLoopError<
            <IO as Read<Board>>::Error,
            <IO as Write<Board>>::Error,
            <S as embedded_hal::digital::ErrorType>::Error,
            <O as output::Output<Board>>::Error,
            !,
            <NVM as nvm::NonvolatileStorage<Board>>::Error,
        >,
    >
    where
        Instant<u64, NOM, DENOM>: Sub<Output = Duration<u64, NOM, DENOM>>,
    {
        let address = nvm.get().config.address;
        info!("Starting main loop with {:?}", nvm.get_config());
        let mut buf_len = 0;
        let mut buf = [0; 256];
        let mut last_receive = timer.now();
        loop {
            let received = match nb_await!(io.read(&mut buf[buf_len..])) {
                Ok(received) => received,
                Err(err) => {
                    buf_len = 0;
                    if let ReadError::UnrecoverableError(e) = err {
                        return Err(MainLoopError::Read(e));
                    } else {
                        continue;
                    }
                }
            };
            if timer.elapsed(last_receive).to_micros() > 1000 {
                buf_len = 0;
            }
            if received > 0 {
                last_receive = timer.now();
                buf_len += received;
            }

            loop {
                let (maybe_request, processed) = slave_next(&buf[..buf_len], address);
                if let Some(request) = maybe_request {
                    info!("Received request: {:?}", request.command());
                    match request {
                        Request::Check(CheckReq) => {
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::Check, CheckRes),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InfoGet(InfoGetReq) => {
                            let info = "Pico I∴O×16 v1.0".as_bytes();
                            let mut info_array = [0u8; 32];
                            for (a, b) in info_array.iter_mut().zip(info.iter().copied()) {
                                *a = b;
                            }
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InfoGet,
                                    InfoGetRes {
                                        info: info_array,
                                        firmware_version_major: 0,
                                        firmware_version_minor: 1,
                                        firmware_version_patch: 0.into(),
                                        uptime: ((timer.now() - self.started).to_secs() as u32)
                                            .into(),
                                    },
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::ConfigGet(ConfigGetReq) => {
                            let Ok(response) = (&ConfigGetReq, nvm, PhantomData).handle().await;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::ConfigGet, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::ConfigSet(request) => {
                            let response = (request, nvm, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Nvm)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::ConfigSet, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::OutputSet(request) => {
                            let response = (request, &mut *output, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Output)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::OutputSet, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::OutputGet(OutputGetReq) => {
                            let response = (&OutputGetReq, &mut *output, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Output)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::OutputGet, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGet(InputGetReq) => {
                            let response = (&InputGetReq, input_loop)
                                .handle()
                                .await
                                .map_err(MainLoopError::Input)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::InputGet, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGetFull(request) => {
                            let response = (request, input_loop)
                                .handle()
                                .await
                                .map_err(MainLoopError::Input)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::InputGetFull, response),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputSetCalibrations(request) => {
                            let response = (request, nvm, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Nvm)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputSetCalibrations,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGetCalibrations(request) => {
                            let Ok(response) = (request, nvm, PhantomData).handle().await;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputGetCalibrations,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputSetThresholds(request) => {
                            let response = (request, nvm, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Nvm)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputSetThresholds,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGetThresholds(request) => {
                            let Ok(response) = (request, nvm, PhantomData).handle().await;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputGetThresholds,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGetThresholdTimes(request) => {
                            let response = (request, timer, input_loop, PhantomData)
                                .handle()
                                .await
                                .map_err(MainLoopError::Input)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputGetThresholdTimes,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::InputGetThresholdStates(request) => {
                            let response = (request, input_loop)
                                .handle()
                                .await
                                .map_err(MainLoopError::Input)?;
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(
                                    address,
                                    Command::InputGetThresholdStates,
                                    response,
                                ),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                        }
                        Request::Reboot(RebootReq) => {
                            info!("Rebooting address {} @ {} Hz", nvm.get().config.address, nvm.get().config.baudrate);
                            Self::write_all_bytes(
                                io,
                                io_send,
                                &Message::new_response(address, Command::Reboot, ()),
                            )
                            .await
                            .map_err(|err| error_coerce!(err))?;
                            timer.wait_for(Duration::<u64, _, _>::millis(1)).await;
                            system.reboot();
                        }
                    }
                    info!("Handled request, response sent");
                }
                if processed == 0 {
                    break;
                }
                buf.copy_within(processed..buf_len, 0);
                buf_len -= processed;
            }
            // make sure to at least one guarantied yield per iteration of the loop to prevent starvation of other tasks
            yield_now().await;
        }
    }

    /// Run the main loop of the firmware.
    pub async fn main_loop<
        Board: ?Sized,
        Io: Read<Board> + Write<Board>,
        IoSend: OutputPin,
        T: Timer<Board, u64, NOM, DENOM>,
        O: output::Output<Board>,
        I: input::Input<Board, Error: From<!>>,
        NVM: nvm::NonvolatileStorage<Board>,
        S: System<Board>,
    >(
        &mut self,
        io: &mut Io,
        io_send: &mut IoSend,
        timer: &T,
        output: &mut O,
        input: &mut I,
        nvm: &nvm::Nvm<NVM, Board>,
        system: &S,
    ) -> Result<
        !,
        MainLoopError<
            <Io as Read<Board>>::Error,
            <Io as Write<Board>>::Error,
            <IoSend as embedded_hal::digital::ErrorType>::Error,
            <O as output::Output<Board>>::Error,
            <I as input::Input<Board>>::Error,
            <NVM as nvm::NonvolatileStorage<Board>>::Error,
        >,
    >
    where
        Instant<u64, NOM, DENOM>: Sub<Output = Duration<u64, NOM, DENOM>>,
    {
        let control = pin!(async {
            let r: Result<
                !,
                MainLoopError<
                    <Io as Read<Board>>::Error,
                    <Io as Write<Board>>::Error,
                    <IoSend as embedded_hal::digital::ErrorType>::Error,
                    <O as output::Output<Board>>::Error,
                    <I as input::Input<Board>>::Error,
                    <NVM as nvm::NonvolatileStorage<Board>>::Error,
                >,
            > = self
                .run(io, io_send, timer, output, nvm, &self.input_loop, system)
                .await
                .map_err(|err| err.convert());
            r
        });
        let input = pin!(async {
            let r: Result<
                !,
                MainLoopError<
                    <Io as Read<Board>>::Error,
                    <Io as Write<Board>>::Error,
                    <IoSend as embedded_hal::digital::ErrorType>::Error,
                    <O as output::Output<Board>>::Error,
                    <I as input::Input<Board>>::Error,
                    <NVM as nvm::NonvolatileStorage<Board>>::Error,
                >,
            > = self
                .input_loop
                .run(input, timer, nvm)
                .await
                .map_err(|err| match err {
                    Either::Left(err) => MainLoopError::Input(err),
                    Either::Right(err) => MainLoopError::Nvm(err),
                });
            r
        });
        select(control, input).await.factor_first().0
    }
}
