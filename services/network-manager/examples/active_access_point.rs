use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let adapter = nm.default_wifi_device().await?;

    match adapter.active_access_point().await? {
        Some(ap) => {
            println!("Connected AP:");
            println!("- ssid: '{}'", ap.ssid);
            println!("- bssid: {}", ap.bssid);
            println!("- strength: {}", ap.strength);
            println!("- frequency: {} MHz", ap.frequency);
            println!("- secure: {}", ap.is_secure());
            println!("- path: {}", ap.path);
        }
        None => {
            println!("No active access point.");
        }
    }

    Ok(())
}
