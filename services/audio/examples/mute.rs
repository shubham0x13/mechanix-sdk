///   cargo run -p audio --example mute
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
    let query = input.trim().to_string();

    // Show current mute state.
    match client.mute(&query) {
        Ok(muted) => println!(
            "'{}' is currently {}.",
            query,
            if muted { "muted" } else { "not muted" }
        ),
        Err(e) => {
            println!("Error: {e}");
            return Ok(());
        }
    }

    print!("Set mute? (y = mute, n = unmute, Enter = keep): ");
    io::stdout().flush()?;
    let mut yn = String::new();
    io::stdin().read_line(&mut yn)?;

    let muted = match yn.trim().to_lowercase().as_str() {
        "y" | "yes" | "1" | "true" => true,
        "n" | "no" | "0" | "false" => false,
        "" => {
            println!("Mute state unchanged.");
            return Ok(());
        }
        other => {
            println!("Unrecognised input '{other}', expected y or n.");
            return Ok(());
        }
    };

    match client.set_mute(&query, muted).await {
        Ok(()) => println!(
            "'{}' is now {}.",
            query,
            if muted { "muted" } else { "unmuted" }
        ),
        Err(e) => println!("Error: {e}"),
    }

    Ok(())
}
