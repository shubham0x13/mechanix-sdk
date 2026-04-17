///   cargo run -p audio --example volume

use mechanix_audio::AudioClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AudioClient::new()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    print!("Enter device ID or name: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let query = input.trim().to_string();

    // Show current volume.
    match client.volume(&query) {
        Ok(vol) => println!("Current volume of '{}': {:.0}%", query, vol * 100.0),
        Err(e)  => { println!("Error: {e}"); return Ok(()); }
    }

    print!("New volume (0–100, or press Enter to keep): ");
    io::stdout().flush()?;
    let mut vol_input = String::new();
    io::stdin().read_line(&mut vol_input)?;
    let trimmed = vol_input.trim();

    if trimmed.is_empty() {
        println!("Volume unchanged.");
        return Ok(());
    }

    match trimmed.parse::<f32>() {
        Ok(percent) => {
            match client.set_volume(&query, percent / 100.0).await {
                Ok(()) => println!("Volume of '{}' set to {:.0}%.", query, percent.clamp(0.0, 100.0)),
                Err(e) => println!("Error: {e}"),
            }
        }
        Err(_) => println!("Invalid number '{trimmed}', volume unchanged."),
    }

    Ok(())
}
