use std::io::{self, Write};

use bluetooth::{
    BluetoothManager,
    agent::{AgentCapability, PairingRequest},
};

fn ask_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn ask_yes_no(prompt: &str) -> io::Result<bool> {
    let answer = ask_line(prompt)?;
    Ok(matches!(answer.to_lowercase().as_str(), "y" | "yes"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bt = BluetoothManager::new().await?;

    let adapter = bt
        .get_adapters()
        .await?
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found")?;
    println!(
        "Using adapter: {} ({})",
        adapter.display_name(),
        adapter.address
    );

    let adapter = bt.adapter(&adapter.name).await?;
    adapter.set_discoverable(true).await?;
    adapter.set_pairable(true).await?;

    let (agent_path, mut requests) = bt
        .register_agent_with_options(AgentCapability::KeyboardDisplay, true)
        .await?;

    println!("Pairing agent registered at {agent_path}.");
    println!("Keep this process running while pairing devices from your UI or shell.");
    println!("Press Ctrl+C to stop.");

    loop {
        tokio::select! {
            request = requests.recv() => {
                let Some(req) = request else {
                    println!("Pairing request channel closed.");
                    break;
                };

                match req {
                    PairingRequest::RequestPinCode {
                        device_address,
                        reply,
                    } => {
                        let prompt = format!("PIN requested for {device_address}. Enter PIN: ");
                        let pin = ask_line(&prompt).unwrap_or_else(|_| "0000".to_string());
                        let _ = reply.send(pin);
                    }
                    PairingRequest::RequestPasskey {
                        device_address,
                        reply,
                    } => {
                        let prompt = format!("Passkey requested for {device_address}. Enter numeric passkey: ");
                        let passkey = ask_line(&prompt)
                            .ok()
                            .and_then(|v| v.parse::<u32>().ok())
                            .unwrap_or(0);
                        let _ = reply.send(passkey);
                    }
                    PairingRequest::RequestConfirmation {
                        device_address,
                        passkey,
                        reply,
                    } => {
                        let prompt = format!(
                            "Confirm passkey {:06} for {device_address}? [y/N]: ",
                            passkey
                        );
                        let accepted = ask_yes_no(&prompt).unwrap_or(false);
                        let _ = reply.send(accepted);
                    }
                    PairingRequest::DisplayPinCode {
                        device_address,
                        pin_code,
                    } => {
                        println!("Display PIN for {}: {}", device_address, pin_code);
                    }
                    PairingRequest::DisplayPasskey {
                        device_address,
                        passkey,
                        entered,
                    } => {
                        println!(
                            "Display passkey for {}: {:06} (entered digits: {})",
                            device_address, passkey, entered
                        );
                    }
                    PairingRequest::Cancel => {
                        println!("Pairing canceled by BlueZ.");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Stopping pairing agent...");
                break;
            }
        }
    }

    if let Err(err) = bt.unregister_agent(&agent_path).await {
        eprintln!(
            "Warning: failed to unregister agent {}: {}",
            agent_path, err
        );
    }

    Ok(())
}
