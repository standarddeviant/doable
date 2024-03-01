// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use std::error::Error;
use std::io::Cursor;
use std::time::Duration;
use tokio::time;

use log::{debug, error, info, warn};

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use uuid::Uuid;

// use crate::items;
use prost::Message;

// Include the `items` module, which is generated from items.proto.
pub mod items {
    include!(concat!(env!("OUT_DIR"), "/doable.items.rs"));
}
pub fn create_large_shirt(color: String) -> items::Shirt {
    let mut shirt = items::Shirt::default();
    shirt.color = color;
    shirt.set_size(items::shirt::Size::Large);
    shirt
}
// pub fn deserialize_shirt(buf: &[u8]) -> Result<items::Shirt, prost::DecodeError> {
pub fn parse_shirt(buf: &Vec<u8>) -> Result<items::Shirt, prost::DecodeError> {
    items::Shirt::decode(&mut Cursor::new(buf))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let large = create_large_shirt("red".to_string());
    info!("PB: large shirt = {large:?}");

    let large_bytes = large.encode_to_vec();
    info!("PB: encoded bytes = {large_bytes:?}");

    if let Ok(shirt) = parse_shirt(&large_bytes) {
        info!("PB: parsed shirt = {shirt:?}");
    }
    // return Ok(());

    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        error!("BLE: No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        info!("BLE: Starting scan on {}...", adapter.adapter_info().await?);
        adapter
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            error!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                info!(
                    "Peripheral {:?} is connected: {:?}",
                    local_name, is_connected
                );
                if !is_connected {
                    debug!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        error!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                }
                let is_connected = peripheral.is_connected().await?;
                info!(
                    "Now connected ({:?}) to peripheral {:?}...",
                    is_connected, &local_name
                );
                peripheral.discover_services().await?;
                info!("Discover peripheral {:?} services...", &local_name);
                for service in peripheral.services() {
                    let check_uuid = Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0")?;
                    if check_uuid == service.uuid {
                        for _ix in 0..10 {
                            warn!("HEY!!!!!! we found it! {check_uuid:?}");
                        }
                    }

                    info!(
                        "Service UUID {}, primary: {}",
                        service.uuid, service.primary
                    );
                    for characteristic in service.characteristics {
                        info!("  {:?}", characteristic.uuid);
                        debug!("  {:?}", characteristic);
                    }
                }
                if is_connected {
                    debug!("Disconnecting from peripheral {:?}...", &local_name);
                    peripheral
                        .disconnect()
                        .await
                        .expect("Error disconnecting from BLE peripheral");
                }
            }
        }
    }
    Ok(())
}
