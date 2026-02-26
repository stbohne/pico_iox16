use std::iter::chain;

use anyhow::Result;
use crossterm::{
    cursor::{RestorePosition, SavePosition},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use pico_iox16_protocol::{CheckReq, CheckRes};
use pico_iox16_tool::Protocol;

pub(crate) async fn scan(device: &mut Protocol, max_address: Option<u16>) -> Result<()> {
    let mut stdout = std::io::stdout();
    let baudrate = device.baudrate();
    execute!(stdout, SavePosition)?;
    let addresses = chain(
        0..=max_address.unwrap_or(0xFFFF),
        if matches!(max_address, Some(0xFFFF) | None) {
            None.into_iter()
        } else {
            Some(0xFFFF).into_iter()
        },
    );
    let mut scanned = 0;
    let mut found = 0;
    for address in addresses {
        execute!(
            stdout,
            RestorePosition,
            Clear(ClearType::FromCursorDown),
            Print(format!("Scanning address {address} at {baudrate} Hz...")),
        )?;
        scanned += 1;
        if device
            .send_request(address, CheckReq, |CheckRes| Ok(()))
            .await
            .is_ok()
        {
            found += 1;
            execute!(
                stdout,
                RestorePosition,
                Clear(ClearType::FromCursorDown),
                Print(format!("{address}\n")),
                SavePosition
            )?;
        }
    }
    execute!(
        stdout,
        RestorePosition,
        Clear(ClearType::FromCursorDown),
        Print(format!("Scan complete. Found {found} out of {scanned} devices.\n")),
    )?;
    Ok(())
}
