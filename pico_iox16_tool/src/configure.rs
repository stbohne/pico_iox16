use anyhow::Result;
use pico_iox16_protocol::{Config, ConfigGetReq, ConfigGetRes, ConfigSetReq, ConfigSetRes, RebootReq, RebootRes};
use pico_iox16_tool::Protocol;

pub(crate) async fn configure(
    device: &mut Protocol,
    address: u16,
    new_address: Option<u16>,
    new_baudrate: Option<u32>,
) -> Result<()> {
    println!("Retrieving current configuration...");
    let old_config = device
        .send_request(address, ConfigGetReq, |ConfigGetRes(config)| Ok(*config))
        .await?;
    println!(
        "Current configuration: address={}, baudrate={} Hz",
        old_config.address, old_config.baudrate
    );
    let config = Config {
        address: new_address.unwrap_or(old_config.address.into()).into(),
        baudrate: new_baudrate.unwrap_or(old_config.baudrate.into()).into(),
        _reserved: [0; 2],
    };
    println!(
        "New configuration: address={}, baudrate={} Hz",
        config.address, config.baudrate
    );
    println!("Sending new configuration...");
    device
        .send_request(
            address,
            ConfigSetReq(config),
            |ConfigSetRes| {
                Ok(())
            },
        )
        .await?;
    println!("Rebooting device...");
    device.send_request(address, RebootReq, |RebootRes| Ok(())).await?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    println!("Check after rebooting...");
    let new_config = device.send_request(config.address.into(), ConfigGetReq, |ConfigGetRes(config)| Ok(*config)).await?;
    if new_config == config {
        println!("Configuration successful!");
    } else {
        println!("Configuration failed! Current configuration: address={}, baudrate={} Hz", new_config.address, new_config.baudrate);
    }
    Ok(())
}
