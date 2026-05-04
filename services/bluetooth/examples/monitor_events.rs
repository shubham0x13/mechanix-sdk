use bluetooth::{BluetoothEvent, BluetoothManager};
use futures::StreamExt;
use tokio::pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let bt = BluetoothManager::new().await?;
    let events = bt.stream_events();

    pin!(events);

    println!("Listening for Bluetooth events. Press Ctrl+C to exit.");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nStopping event monitor...");
                break;
            }
            Some(event) = events.next() => {
                print_event(event);
            }
        }
    }

    Ok(())
}

fn print_event(event: BluetoothEvent) {
    match event {
        // ADAPTER EVENTS (Power & Scanning)
        BluetoothEvent::AdapterPropertiesChanged { path, changes } => {
            if let Some(powered) = changes.powered {
                let state = if powered { "ON" } else { "OFF" };
                println!("🔌 Adapter [{path}] power changed to: {state}");
            }
            if let Some(discovering) = changes.discovering {
                let state = if discovering { "Started" } else { "Stopped" };
                println!("🔍 Adapter [{path}] scanning: {state}");
            }
        }

        // DISCOVERY EVENTS
        BluetoothEvent::DeviceDiscovered(device) => {
            let name = device
                .properties
                .name
                .unwrap_or_else(|| "Unknown".to_string());
            let addr = device
                .properties
                .address
                .unwrap_or_else(|| "No Address".to_string());
            println!("✨ Discovered Device: {name} ({addr})");
        }

        BluetoothEvent::DeviceLost { path } => {
            println!("👻 Device lost or went out of range: {path}");
        }

        // CONNECTION & TELEMETRY EVENTS
        BluetoothEvent::DevicePropertiesChanged {
            address, changes, ..
        } => {
            // Connection state flipped
            if let Some(connected) = changes.connected {
                let state = if connected {
                    "Connected"
                } else {
                    "Disconnected"
                };
                println!("🔗 Device [{address}] is now: {state}");
            }

            // Services resolved (Safe to start GATT operations)
            if let Some(true) = changes.services_resolved {
                println!("⚙️ Device [{address}] GATT services fully resolved and ready.");
            }

            // Signal strength updates
            if let Some(rssi) = changes.rssi {
                println!("📶 Device [{address}] RSSI changed to: {rssi} dBm");
            }
        }

        // BATTERY EVENTS
        BluetoothEvent::BatteryChanged {
            address, changes, ..
        } => {
            if let Some(percentage) = changes.percentage {
                println!("🔋 Device [{address}] battery level: {percentage}%");
            }
        }

        other => {
            println!("📡 Other System Event: {:?}", other);
        }
    }
}
