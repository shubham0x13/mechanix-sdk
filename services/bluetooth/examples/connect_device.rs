use bluetooth::BluetoothManager;

fn print_usage(binary: &str) {
    println!("Usage: {binary} <device-path> [trust]");
    println!("Example:");
    println!("  {binary} /org/bluez/hci0/dev_12_34_56_78_9A_BC true",);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let binary = args.next().unwrap_or_else(|| "connect_device".to_string());
    let Some(device_path) = args.next() else {
        print_usage(&binary);
        return Ok(());
    };

    let trust = args
        .next()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);

    let bt = BluetoothManager::new().await?;
    let device = bt.device(device_path.clone()).await?;

    println!("Connecting to {} (trust={})...", device_path, trust);
    device.connect_or_pair(trust).await?;

    println!("Done.");
    let connected = device.is_connected().await?;
    let paired = device.is_paired().await?;
    let trusted = device.is_trusted().await?;
    let alias = device.alias().await?;
    let battery = device.battery_percentage().await?;

    println!("- alias:     {}", alias);
    println!("- connected: {}", connected);
    println!("- paired:    {}", paired);
    println!("- trusted:   {}", trusted);
    println!("- battery:   {:?}", battery);

    Ok(())
}
