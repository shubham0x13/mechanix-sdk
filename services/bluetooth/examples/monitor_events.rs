use bluetooth::{BluetoothEvent, BluetoothManager};
use futures::StreamExt;
use tokio::pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("[Event] -> {:#?}", event);
}
