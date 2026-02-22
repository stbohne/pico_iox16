use core::panic::PanicInfo;
use cortex_m::{delay::Delay, interrupt};
use embedded_hal::digital::OutputPin;
use rp235x_hal::{Sio, gpio::Pins, pac::Peripherals};

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Use a critical section to ensure the setup is atomic.
    interrupt::free(|_| {
        // Unsafely take ownership of the peripherals.
        // SAFETY: This is the panic handler. We are halting the system and
        // providing a debug signal. We can risk taking the peripherals again.
        let mut pac = unsafe { Peripherals::steal() };
        let core = unsafe { cortex_m::Peripherals::steal() };
        let sio = Sio::new(pac.SIO);

        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        let mut delay = Delay::new(core.SYST, 125_000_000);

        // Configure GPIO25, the onboard LED pin, as a push-pull output.
        let mut led = pins.gpio25.into_push_pull_output();
        loop {
            let _ = led.set_high();
            delay.delay_ms(100);
            let _ = led.set_low();
            delay.delay_ms(100);
        }
    });

    // Loop forever to halt the processor.
    loop {
        cortex_m::asm::wfi(); // Wait for interrupt, effectively sleeping
    }
}
