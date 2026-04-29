use network_manager::NetworkManager;

fn print_usage(bin: &str) {
    println!("Usage: {bin} <ssid> <passphrase>");
    println!("Example: {bin} HomeWiFi super-secret-password");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let bin = args.next().unwrap_or_else(|| "connect_psk".to_string());
    let Some(ssid) = args.next() else {
        print_usage(&bin);
        return Ok(());
    };
    let Some(passphrase) = args.next() else {
        print_usage(&bin);
        return Ok(());
    };

    let nm = NetworkManager::new().await?;
    let adapter = nm.default_wifi_device().await?;
    let active_path = adapter.connect_psk(&ssid, &passphrase).await?;

    println!("Connected to '{}'", ssid);
    println!("Active connection path: {}", active_path);
    Ok(())
}
