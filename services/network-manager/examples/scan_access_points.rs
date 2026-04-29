use std::time::Duration;

use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wait_secs = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(3);

    let nm = NetworkManager::new().await?;
    let adapter = nm.default_wifi_device().await?;
    println!("Requesting Wi-Fi scan...");
    adapter.request_scan().await?;

    println!("Waiting {}s for scan results...", wait_secs);
    tokio::time::sleep(Duration::from_secs(wait_secs)).await;

    let aps = adapter.list_access_points().await?;
    if aps.is_empty() {
        println!("No access points found.");
        return Ok(());
    }

    println!("Found {} access point(s):", aps.len());
    for ap in aps {
        println!("- ssid: '{}'", ap.ssid);
        println!("  bssid: {}", ap.bssid);
        println!("  strength: {}", ap.strength);
        println!("  frequency: {} MHz", ap.frequency);
        println!("  secure: {}", ap.is_secure());
        println!("  path: {}", ap.path);
    }

    Ok(())
}
