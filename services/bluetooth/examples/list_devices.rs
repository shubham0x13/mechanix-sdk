use bluetooth::BluetoothManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt = BluetoothManager::new().await?;
    let devices = bt.get_devices().await?;

    if devices.is_empty() {
        println!("No cached Bluetooth devices found.");
        println!("Tip: run discovery first to populate BlueZ device cache.");
        return Ok(());
    }

    println!("Found {} device(s):", devices.len());
    for device in devices {
        println!("{:#?}", device);
    }

    Ok(())
}
