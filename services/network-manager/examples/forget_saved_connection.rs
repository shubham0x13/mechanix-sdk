use network_manager::NetworkManager;

fn print_usage(bin: &str) {
    println!("Usage: {bin} <connection-uuid>");
    println!("Example: {bin} 6b4fd6f9-3064-4d14-9e33-df4ab3b1e092");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let bin = args
        .next()
        .unwrap_or_else(|| "forget_saved_connection".to_string());
    let Some(uuid) = args.next() else {
        print_usage(&bin);
        return Ok(());
    };

    let nm = NetworkManager::new().await?;
    nm.forget_saved_connection(&uuid).await?;
    println!("Removed saved connection UUID: {}", uuid);

    Ok(())
}
