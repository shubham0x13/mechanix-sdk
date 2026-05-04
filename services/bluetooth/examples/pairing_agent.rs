use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};

use bluetooth::{
    BluetoothManager,
    agent::{AgentCapability, PairingRequest},
};

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
        "Using adapter: {} ({:?})",
        adapter.display_name(),
        adapter.path
    );

    let adapter = bt.adapter(adapter.path).await?;
    adapter.set_discoverable(true).await?;
    adapter.set_pairable(true).await?;

    let (agent_guard, mut requests) = bt
        .register_agent(AgentCapability::KeyboardDisplay, true)
        .await?;

    println!("Pairing agent registered successfully.");
    println!("Keep this process running while pairing devices.");
    println!("Press Ctrl+C to stop.");

    loop {
        tokio::select! {
            request = requests.recv() => {
                let Some(req) = request else {
                    println!("Pairing request channel closed.");
                    break;
                };

                match req {
                    PairingRequest::RequestPinCode { device_address, responder } => {
                        print!("PIN requested for {}. Enter PIN: ", device_address);
                        std::io::stdout().flush().unwrap();

                        let mut stdin = BufReader::new(tokio::io::stdin());
                        let mut line = String::new();

                        // RACE: Keyboard Input vs. Cancellation Signal
                        tokio::select! {
                            _ = stdin.read_line(&mut line) => {
                                responder.reply(line.trim());
                            }
                            cancel_req = requests.recv() => {
                                if let Some(PairingRequest::Cancel) = cancel_req {
                                    println!("\n[!] Pairing canceled by remote device.");
                                }
                            }
                        }
                    }
                    PairingRequest::RequestPasskey { device_address, responder } => {
                        print!("Passkey requested for {}. Enter numeric passkey: ", device_address);
                        std::io::stdout().flush().unwrap();

                        let mut stdin = BufReader::new(tokio::io::stdin());
                        let mut line = String::new();

                        tokio::select! {
                            _ = stdin.read_line(&mut line) => {
                                if let Ok(passkey) = line.trim().parse::<u32>() {
                                    responder.reply(passkey);
                                } else {
                                    println!("Invalid passkey. Rejecting.");
                                    responder.reject();
                                }
                            }
                            cancel_req = requests.recv() => {
                                if let Some(PairingRequest::Cancel) = cancel_req {
                                    println!("\n[!] Pairing canceled by remote device.");
                                }
                            }
                        }
                    }
                    PairingRequest::RequestConfirmation { device_address, passkey, responder } => {
                        print!("Confirm passkey {:06} for {}? [y/N]: ", passkey, device_address);
                        std::io::stdout().flush().unwrap();

                        let mut stdin = BufReader::new(tokio::io::stdin());
                        let mut line = String::new();

                        tokio::select! {
                            _ = stdin.read_line(&mut line) => {
                                let accepted = matches!(line.trim().to_lowercase().as_str(), "y" | "yes");
                                println!("{} pairing request.", if accepted { "Accepting" } else { "Rejecting" });
                                if accepted {
                                    responder.confirm();
                                } else {
                                    responder.reject();
                                }
                            }
                            cancel_req = requests.recv() => {
                                if let Some(PairingRequest::Cancel) = cancel_req {
                                    println!("\n[!] Pairing canceled by remote device.");
                                }
                            }
                        }
                    }
                    PairingRequest::DisplayPinCode { device_address, pin_code } => {
                        println!("Display PIN for {}: {}", device_address, pin_code);
                    }
                    PairingRequest::DisplayPasskey { device_address, passkey, entered } => {
                        println!(
                            "Display passkey for {}: {:06} (entered digits: {})",
                            device_address, passkey, entered
                        );
                    }
                    PairingRequest::Cancel => {
                        // This catches any stray cancelations that happen outside of active prompts
                        println!("[!] Pairing canceled by BlueZ.");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nStopping pairing agent...");
                break;
            }
        }
    }

    if let Err(err) = agent_guard.unregister().await {
        eprintln!("Warning: failed to unregister agent: {}", err);
    }

    Ok(())
}
