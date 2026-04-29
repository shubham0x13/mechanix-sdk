use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let devices = nm.list_wifi_devices().await?;

    if devices.is_empty() {
        println!("No Wi-Fi devices found.");
        return Ok(());
    }

    println!("Found {} Wi-Fi device(s):", devices.len());
    for d in devices {
        println!("- interface: {}", d.interface);
        println!("  path: {}", d.path);
        println!("  driver: {}", d.driver);
        println!("  hw_address: {}", d.hw_address);
        println!("  state: {:?}", d.state);
        println!("  managed: {}", d.managed);
        println!("  wireless_capabilities: {:?}", d.wireless_capabilities);
    }

    Ok(())
}
