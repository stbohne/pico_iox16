//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(try_blocks)]
#![feature(never_type)]

use core::pin::pin;

use futures::future::{Either, select};

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::OutputPin;
use rp235x_hal::adc::AdcPin;
use rp235x_hal::clocks::init_clocks_and_plls;
use rp235x_hal::gpio::{Pins, PullNone};
use rp235x_hal::{Adc, entry};
use rp235x_hal::{Clock, pac};
// use panic_probe as _;
use rp235x_hal::fugit::{ExtU32 as _, RateExtU32 as _};
use rp235x_hal::uart::{DataBits, StopBits, UartConfig};

use crate::nvm::Nvm;
use crate::output::OutputPins;
use crate::runtime::{Timer0, Uart};
use pico_iox16_firmware::{
    runtime::Timer,
    runtime::{WaitUntil as _, block_on},
};

mod input;
mod nvm;
mod output;
mod panic;
mod runtime;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: rp235x_hal::block::ImageDef = rp235x_hal::block::ImageDef::secure_exe();

#[entry]
#[allow(clippy::never_loop)]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = rp235x_hal::Watchdog::new(pac.WATCHDOG);
    let sio = rp235x_hal::Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer0(rp235x_hal::Timer::new_timer0(
        pac.TIMER0,
        &mut pac.RESETS,
        &clocks,
    ));

    let Pins {
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
        gpio16,
        gpio17,
        gpio19,
        gpio20,
        gpio21,
        gpio22,
        gpio25,
        gpio26,
        gpio27,
        ..
    } = rp235x_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut uart = Uart::new(
        rp235x_hal::uart::UartPeripheral::new(
            pac.UART0,
            (gpio16.into_function(), gpio17.into_function()),
            &mut pac.RESETS,
        )
        .enable(
            UartConfig::new(115_200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap(),
    );
    let mut uart_send = gpio19.into_push_pull_output_in_state(rp235x_hal::gpio::PinState::Low);

    let mut led_pin = gpio25.into_push_pull_output().into_pull_type::<PullNone>();

    let mut main_loop = pico_iox16_firmware::MainLoop::new(&timer);
    let mut output = output::Output::new(
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
        },
        rp235x_hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS),
    );
    let mut input = input::Input::new(
        gpio22,
        gpio21,
        gpio20,
        Adc::new(pac.ADC, &mut pac.RESETS),
        AdcPin::new(gpio26).unwrap(),
        AdcPin::new(gpio27).unwrap(),
    );

    let nvm = Nvm::take().unwrap();
    let main = pin!(main_loop.main_loop(
        &mut uart,
        &mut uart_send,
        &timer,
        &mut output,
        &mut input,
        nvm,
    ));
    let blink = pin!(blink(&mut led_pin, &timer));
    let Either::Left((Err(err), _)) = block_on(select(main, blink));
    match err {}
}

async fn blink<LedPin: OutputPin>(led_pin: &mut LedPin, timer: &Timer0) -> ! {
    let mut until = timer.now();
    loop {
        led_pin.set_high().unwrap();
        until += 500.millis();
        timer.wait_until(until).await;
        led_pin.set_low().unwrap();
        until += 500.millis();
        timer.wait_until(until).await;
    }
}
/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [rp235x_hal::binary_info::EntryAddr; 5] = [
    rp235x_hal::binary_info::rp_cargo_bin_name!(),
    rp235x_hal::binary_info::rp_cargo_version!(),
    rp235x_hal::binary_info::rp_program_description!(c"RP2350 Template"),
    rp235x_hal::binary_info::rp_cargo_homepage_url!(),
    rp235x_hal::binary_info::rp_program_build_attribute!(),
];

// End of file
