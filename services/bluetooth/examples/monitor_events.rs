use bluetooth::{BluetoothEvent, BluetoothManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt = BluetoothManager::new().await?;
    let mut events = bt.subscribe();

    println!("Listening for Bluetooth events. Press Ctrl+C to exit.");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Stopping event monitor...");
                break;
            }
            evt = events.recv() => {
                match evt {
                    Ok(event) => print_event(event),
                    Err(err) => eprintln!("Event stream error: {}", err),
                }
            }
        }
    }

    Ok(())
}

fn print_event(event: BluetoothEvent) {
    println!("[Event] -> {:#?}", event);
}
