use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let adapter = nm.default_wifi_device().await?;
    adapter.disconnect().await?;
    println!("Disconnect requested for default Wi-Fi device.");
    Ok(())
}
