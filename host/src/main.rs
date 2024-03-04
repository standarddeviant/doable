// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use std::error::Error;
use std::io::Cursor;
// use std::io::Cursor;
use std::time::Duration;
use tokio::time;

use log::{debug, error, info, warn};

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use uuid::Uuid;

// use crate::ble_gatt;
use prost::Message;

// Include the `ble_gatt` module, which is generated from items.proto.
pub mod ble_gatt {
    include!(concat!(env!("OUT_DIR"), "/doable.ble_gatt.rs"));
}

pub fn parse_default_gatt(
    buf: &Vec<u8>,
) -> Result<ble_gatt::DefaultGattMessage, prost::DecodeError> {
    ble_gatt::DefaultGattMessage::decode(&mut Cursor::new(buf))
}

fn pb_test() {
    let nrfx = ble_gatt::SoftwareVersion {
        major: 3,
        minor: 2,
        patch: 1,
        url: "na".into(),
        hash: "na".into(),
    };
    let p = ble_gatt::TelemetryToPeripheral {
        nrfx: Some(nrfx.clone()),
    };
    info!("telem to periph = {p:?}");

    let q = ble_gatt::DefaultGattMessage {
        m: Some(ble_gatt::default_gatt_message::M::Swver(nrfx.clone())),
    };
    info!("default gatt message = {q:?}");

    // let r: Vec<u8> = q.into();
    let r: Vec<u8> = q.clone().encode_to_vec();
    info!("encoded = {r:?}");

    if let Ok(defgatt) = ble_gatt::DefaultGattMessage::decode(&mut Cursor::new(r)) {
        // parse_default_gatt(&r) {
        info!("PB: parsed message = {defgatt:?}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    pb_test();
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
