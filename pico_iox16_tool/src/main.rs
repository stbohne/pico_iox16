use std::time::Duration;

use clap::Parser;
use anyhow::{Context as _, Result};
use pico_iox16_tool::Protocol;
use tokio_serial::SerialPortBuilderExt;

mod scan;
mod configure;
mod calibrate;

#[derive(Debug, Parser)]
struct Args {
    /// The serial device to use, e.g. /dev/ttyUSB0
    device: String,
    /// The baud rate for the serial connection
    #[clap(short, long, default_value = "1000000")]
    baudrate: u32,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Scans all addresses for the presence of a device and prints the results.
    Scan{
        /// Highest address to scan. If not specified, scans all addresses up to 0xFFFF.
        /// Address 0xFFFF is always scanned, even if a lower max address is specified.
        max_address: Option<u16>,
    },
    /// Sets address and baudrate for a device and reboots it.
    Configure{
        /// The address of the device to configure.
        address: u16,
        /// The new address to set for the device.
        #[clap(short = 'a', long)]
        new_address: Option<u16>,
        /// The new baud rate to set for the device.
        #[clap(short = 'b', long)]
        new_baudrate: Option<u32>,
    },
    /// Interactive calibration of the inputs and outputs of the device at the given address
    Calibrate{
        /// The address of the device to calibrate.
        address: u16,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut device = Protocol::new(tokio_serial::new(&args.device, args.baudrate).timeout(Duration::from_micros(100))
        .open_native_async().context("Opening serial port")?);
    match args.command {
        Command::Scan { max_address } => scan::scan(&mut device, max_address).await,
        Command::Configure { address, new_address, new_baudrate } => configure::configure(&mut device, address, new_address, new_baudrate).await,
        Command::Calibrate { address } => calibrate::calibrate(&mut device, address).await,
    }
}
