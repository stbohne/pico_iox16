use anyhow::Result;
use pico_iox16_tool::Protocol;

pub(crate) async fn calibrate(device: &mut Protocol, address: u16) -> Result<()> {
    Ok(())
}
