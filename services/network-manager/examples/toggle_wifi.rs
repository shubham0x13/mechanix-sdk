use network_manager::NetworkManager;

fn parse_bool_arg(value: &str) -> Result<bool, String> {
    value
        .parse::<bool>()
        .map_err(|_| format!("Invalid bool '{}', expected true|false", value))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maybe_arg = std::env::args().nth(1);

    let nm = NetworkManager::new().await?;

    match maybe_arg {
        Some(value) => {
            let enabled = parse_bool_arg(&value)?;
            nm.set_wifi_enabled(enabled).await?;
            println!("Wi-Fi set to: {}", enabled);
        }
        None => {
            let enabled = nm.toggle_wifi().await?;
            println!("Wi-Fi toggled. New state: {}", enabled);
        }
    }

    println!("Wi-Fi currently enabled: {}", nm.is_wifi_enabled().await?);
    Ok(())
}
