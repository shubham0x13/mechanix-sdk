use bluetooth::{BluetoothEvent, BluetoothManager};
use futures::StreamExt;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::pin;
use zbus::zvariant::OwnedObjectPath;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hci0".to_string());
    let seconds = std::env::args()
        .nth(2)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);

    let bt = BluetoothManager::new().await?;
    let adapter = bt.adapter(&adapter_name).await?;
    let events = bt.stream_events();

    pin!(events);

    if !adapter.is_powered().await? {
        println!("Adapter {} is off. Powering it on...", adapter_name);
        adapter.set_powered(true).await?;
    }

    adapter.start_discovery().await?;
    println!(
        "Scanning on {} for {}s. Press Ctrl+C to stop early.",
        adapter_name, seconds
    );

    let start = Instant::now();
    let mut discovered: HashMap<OwnedObjectPath, String> = HashMap::new();

    while start.elapsed() < Duration::from_secs(seconds) {
        let Some(wait_time) = Duration::from_secs(seconds).checked_sub(start.elapsed()) else {
            break;
        };

        let recv = tokio::time::timeout(wait_time, events.next()).await;
        let Ok(Some(event)) = recv else {
            break;
        };

        if let BluetoothEvent::DeviceDiscovered(info) = event {
            discovered.insert(info.path.clone(), info.display_name().to_string());
            println!(
                "Discovered: {} ({:?}) -> {}",
                info.display_name(),
                info.properties.address,
                info.path
            );
        }
    }

    adapter.stop_discovery().await?;

    println!(
        "\nScan complete. {} unique device(s) seen.",
        discovered.len()
    );
    for (path, name) in discovered {
        println!("- {} [{}]", name, path);
    }

    Ok(())
}
