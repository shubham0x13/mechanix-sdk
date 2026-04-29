use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nm = NetworkManager::new().await?;
    let active = nm.list_active_connections().await?;

    if active.is_empty() {
        println!("No active connections.");
        return Ok(());
    }

    println!("Found {} active connection(s):", active.len());
    for c in active {
        println!("- id: {}", c.id);
        println!("  uuid: {}", c.uuid);
        println!("  type: {}", c.connection_type);
        println!("  state: {:?}", c.state);
        println!("  default_ipv4: {}", c.is_default_ipv4);
        println!("  default_ipv6: {}", c.is_default_ipv6);
        println!("  vpn: {}", c.is_vpn);
        println!("  path: {}", c.path);
    }

    Ok(())
}
