use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let saved = nm.list_saved_connections().await?;

    if saved.is_empty() {
        println!("No saved connections.");
        return Ok(());
    }

    println!("Found {} saved connection(s):", saved.len());
    for c in saved {
        println!("- id: {}", c.id);
        println!("  uuid: {}", c.uuid);
        println!("  type: {}", c.connection_type);
        println!("  autoconnect: {}", c.autoconnect);
        println!("  path: {}", c.path);
    }

    Ok(())
}
