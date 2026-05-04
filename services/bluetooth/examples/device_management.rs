use bluetooth::BluetoothManager;

fn print_usage(binary: &str) {
    println!("Usage: {binary} <device-path> <action> [value]");
    println!("Actions:");
    println!("  info");
    println!("  trust <true|false>");
    println!("  block <true|false>");
    println!("  alias <new-alias>");
    println!("  wake <true|false>");
    println!("  disconnect");
    println!("  forget");
    println!("Examples:");
    println!("  {binary} /org/bluez/hci0/dev_12_34_56_78_9A_BC trust true",);
    println!("  {binary} /org/bluez/hci0/dev_12_34_56_78_9A_BC alias My Earbuds");
    println!("  {binary} /org/bluez/hci0/dev_12_34_56_78_9A_BC forget");
}

fn parse_bool(value: &str, field: &str) -> Result<bool, String> {
    value
        .parse::<bool>()
        .map_err(|_| format!("Invalid value for {field}: '{value}' (expected true|false)"))
}

fn extract_adapter_name(device_path: &str) -> Option<&str> {
    device_path
        .split('/')
        .find(|segment| segment.starts_with("hci"))
}

async fn print_device_info(
    bt: &BluetoothManager,
    device_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let device = bt.device(device_path).await?;

    let name = device.name().await?;
    let alias = device.alias().await?;
    let connected = device.is_connected().await?;
    let paired = device.is_paired().await?;
    let trusted = device.is_trusted().await?;
    let blocked = device.is_blocked().await?;
    let services_resolved = device.are_services_resolved().await?;
    let battery = device.battery_percentage().await?;

    println!("Device state:");
    println!("- path:              {}", device_path);
    println!("- name:              {}", name);
    println!("- alias:             {}", alias);
    println!("- connected:         {}", connected);
    println!("- paired:            {}", paired);
    println!("- trusted:           {}", trusted);
    println!("- blocked:           {}", blocked);
    println!("- services_resolved: {}", services_resolved);
    println!("- battery:           {:?}", battery);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let binary = args
        .next()
        .unwrap_or_else(|| "device_management".to_string());

    let Some(device_path) = args.next() else {
        print_usage(&binary);
        return Ok(());
    };

    let Some(action) = args.next() else {
        print_usage(&binary);
        return Ok(());
    };

    let bt = BluetoothManager::new().await?;
    let device = bt.device(device_path.clone()).await?;

    match action.as_str() {
        "info" => {
            print_device_info(&bt, &device_path).await?;
        }
        "trust" => {
            let Some(value) = args.next() else {
                return Err("Missing value for trust action".into());
            };
            let trust = parse_bool(&value, "trust")?;
            device.set_trusted(trust).await?;
            println!("Updated trusted -> {}", trust);
            print_device_info(&bt, &device_path).await?;
        }
        "block" => {
            let Some(value) = args.next() else {
                return Err("Missing value for block action".into());
            };
            let blocked = parse_bool(&value, "block")?;
            device.set_blocked(blocked).await?;
            println!("Updated blocked -> {}", blocked);
            print_device_info(&bt, &device_path).await?;
        }
        "alias" => {
            let alias_parts: Vec<String> = args.collect();
            if alias_parts.is_empty() {
                return Err("Missing alias value".into());
            }
            let alias = alias_parts.join(" ");
            device.set_alias(&alias).await?;
            println!("Updated alias -> {}", alias);
            print_device_info(&bt, &device_path).await?;
        }
        "wake" => {
            let Some(value) = args.next() else {
                return Err("Missing value for wake action".into());
            };
            let wake_allowed = parse_bool(&value, "wake")?;
            device.set_wake_allowed(wake_allowed).await?;
            println!("Updated wake_allowed -> {}", wake_allowed);
            print_device_info(&bt, &device_path).await?;
        }
        "disconnect" => {
            device.disconnect().await?;
            println!("Device disconnected (if it was connected).");
            print_device_info(&bt, &device_path).await?;
        }
        "forget" => {
            let Some(adapter_name) = extract_adapter_name(&device_path) else {
                return Err(
                    format!("Could not infer adapter name from path: {}", device_path).into(),
                );
            };
            bt.adapter(adapter_name)
                .await?
                .forget_device(device_path.as_str())
                .await?;
            println!("Device forgotten from adapter {}.", adapter_name);
        }
        _ => {
            print_usage(&binary);
            return Err(format!("Unknown action: {}", action).into());
        }
    }

    Ok(())
}
