///   cargo run -p audio --example get_device

use audio::AudioClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AudioClient::new()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    print!("Enter device ID or name: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let query = input.trim();

    match client.get_device(query) {
        Some(d) => {
            println!("\nFound device:");
            println!("  ID          : {}", d.id);
            println!("  Name        : {}", d.name);
            println!(
                "  Description : {}",
                d.description.as_deref().unwrap_or("-")
            );
            println!("  Type        : {}", if d.is_output() { "Output (Sink)" } else { "Input (Source)" });
            println!("  Volume      : {:.0}%", d.volume * 100.0);
            println!("  Muted       : {}", d.muted);
            println!("  Default     : {}", d.is_default);
        }
        None => println!("No device found for '{query}'."),
    }

    Ok(())
}
