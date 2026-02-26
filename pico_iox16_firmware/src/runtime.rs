use core::{
    ops::{Add, Sub},
    pin::pin,
    task::{Context, Waker},
};

use fugit::{Duration, Instant};

/// Timer counter abstraction
pub trait Timer<Board: ?Sized, T, const NOM: u32, const DENOM: u32> {
    /// Returns the current counter
    fn now(&self) -> Instant<T, NOM, DENOM>;
}
/// Convenience trait for calculating elapsed time since an instant
pub trait Elapsed<Board: ?Sized, T, const NOM: u32, const DENOM: u32>:
    Timer<Board, T, NOM, DENOM>
{
    /// Returns the elapsed time since the given instant
    fn elapsed(&self, since: Instant<T, NOM, DENOM>) -> Duration<T, NOM, DENOM>;
}
impl<Board: ?Sized, T, const NOM: u32, const DENOM: u32, U> Elapsed<Board, T, NOM, DENOM> for U
where
    U: Timer<Board, T, NOM, DENOM>,
    Instant<T, NOM, DENOM>: Sub<Output = Duration<T, NOM, DENOM>>,
{
    fn elapsed(&self, since: Instant<T, NOM, DENOM>) -> Duration<T, NOM, DENOM> {
        self.now() - since
    }
}

/// Error reading from an ADC.
pub enum ReadError<T> {
    /// ADC conversion failed, but can be retried.
    RecoverableError,
    /// ADC conversion failed in some unexpected way that may need special action.
    UnrecoverableError(T),
}

/// IO read abstraction
pub trait Read<Board: ?Sized> {
    type Error;
    /// Reads bytes into `buf`, returning the number of bytes read. If no data is available, returns `nb::Error::WouldBlock`. 
    /// If an error occurs, returns `nb::Error::Other`.
    fn read(&mut self, buf: &mut [u8]) -> nb::Result<usize, ReadError<Self::Error>>;
}

// IO write abstraction
pub trait Write<Board: ?Sized> {
    type Error;
    /// Writes bytes from `buf`, returning the number of bytes written. If the write would block, returns `nb::Error::WouldBlock`.
    /// If an error occurs, returns `nb::Error::Other`.
    fn write(&mut self, buf: &[u8]) -> nb::Result<usize, Self::Error>;
    /// Flushes any buffered data. If the flush would block, returns `nb::Error::WouldBlock`.
    /// If an error occurs, returns `nb::Error::Other`.
    fn flush(&mut self) -> nb::Result<(), Self::Error>;
}

/// Yield to the executor, allowing other tasks to run.
/// 
/// Since we are using [`nb`] for async IO, we have to make sure to call this function
/// at least once in every iteration of long-running (usually infinite) loops.
pub fn yield_now() -> impl core::future::Future<Output = ()> + Send + Sync {
    struct YieldNow {
        yielded: bool,
    }
    impl core::future::Future for YieldNow {
        type Output = ();
        fn poll(
            mut self: core::pin::Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> core::task::Poll<Self::Output> {
            if self.yielded {
                core::task::Poll::Ready(())
            } else {
                self.yielded = true;
                core::task::Poll::Pending
            }
        }
    }
    YieldNow { yielded: false }
}

/// Extremely simple single-threaded executor that runs a single future to completion.
/// This is used to run the main loop of the firmware.
pub fn block_on<F: core::future::Future>(f: F) -> F::Output {
    let waker = Waker::noop();
    let mut ctx = Context::from_waker(waker);
    let mut f = pin!(f);
    loop {
        match f.as_mut().poll(&mut ctx) {
            core::task::Poll::Ready(v) => return v,
            core::task::Poll::Pending => {}
        }
    }
}

/// Convenience trait for waiting until a certain instant.
pub trait WaitUntil<Board: ?Sized, T, const NOM: u32, const DENOM: u32>:
    Timer<Board, T, NOM, DENOM>
{
    /// Waits until the given instant is reached.
    fn wait_until(&self, until: Instant<T, NOM, DENOM>) -> impl core::future::Future<Output = ()>;
}
impl<Board: ?Sized, T, const NOM: u32, const DENOM: u32, U> WaitUntil<Board, T, NOM, DENOM> for U
where
    U: Timer<Board, T, NOM, DENOM>,
    Instant<T, NOM, DENOM>: PartialOrd,
{
    async fn wait_until(&self, until: Instant<T, NOM, DENOM>) {
        loop {
            if self.now() >= until {
                break;
            }
            yield_now().await;
        }
    }
}

/// Convenience trait for waiting for a certain duration.
pub trait WaitFor<Board: ?Sized, T, const NOM: u32, const DENOM: u32>:
    WaitUntil<Board, T, NOM, DENOM>
{
    /// Waits for the given duration.
    fn wait_for(&self, delay: Duration<T, NOM, DENOM>) -> impl core::future::Future<Output = ()>;
}
impl<Board: ?Sized, T, const NOM: u32, const DENOM: u32, U> WaitFor<Board, T, NOM, DENOM> for U
where
    U: WaitUntil<Board, T, NOM, DENOM>,
    Instant<T, NOM, DENOM>: Add<Duration<T, NOM, DENOM>, Output = Instant<T, NOM, DENOM>>,
{
    fn wait_for(&self, delay: Duration<T, NOM, DENOM>) -> impl core::future::Future<Output = ()> {
        self.wait_until(self.now() + delay)
    }
}

/// Turns an expression that returns `nb::Result` into an async expression that waits until it 
/// returns `Ok` or `Err(nb::Error::Other)`, yielding to the executor in the meantime.
#[macro_export]
macro_rules! nb_await {
    ($e:expr) => {
        loop {
            match $e {
                Ok(v) => break Ok(v),
                Err(nb::Error::Other(err)) => break Err(err),
                Err(nb::Error::WouldBlock) => {
                    $crate::runtime::yield_now().await;
                }
            }
        }
    };
}
pub use nb_await;

pub trait System<Board: ?Sized>: Sized {
    fn reboot(&self) -> !;
}