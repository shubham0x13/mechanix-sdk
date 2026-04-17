use bluetooth::BluetoothManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt = BluetoothManager::new().await?;
    let adapters = bt.get_adapters().await?;

    if adapters.is_empty() {
        println!("No Bluetooth adapters found.");
        return Ok(());
    }

    println!("Found {} adapter(s):", adapters.len());
    for adapter in adapters {
        println!("{:#?}", adapter);
    }

    Ok(())
}
