///   cargo run -p audio --example set_default_device

use audio::AudioClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AudioClient::new()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Show current defaults.
    match client.default_output_device() {
        Ok(d)  => println!("Current default output: [{}] {}", d.id, d.name),
        Err(e) => println!("Current default output: {e}"),
    }
    match client.default_input_device() {
        Ok(d)  => println!("Current default input:  [{}] {}", d.id, d.name),
        Err(e) => println!("Current default input:  {e}"),
    }
    println!();

    // Ask which direction.
    print!("Set default for (o = output, i = input): ");
    io::stdout().flush()?;
    let mut dir_input = String::new();
    io::stdin().read_line(&mut dir_input)?;

    let is_output = match dir_input.trim().to_lowercase().as_str() {
        "o" | "out" | "output" => true,
        "i" | "in"  | "input"  => false,
        other => {
            println!("Unrecognised choice '{other}', expected o or i.");
            return Ok(());
        }
    };

    // List available devices for the chosen direction.
    let devices = if is_output {
        client.list_output_devices()
    } else {
        client.list_input_devices()
    };

    if devices.is_empty() {
        println!("No {} devices found.", if is_output { "output" } else { "input" });
        return Ok(());
    }

    println!("\nAvailable {} devices:", if is_output { "output" } else { "input" });
    for d in &devices {
        println!("  [{id}] {name}  —  {desc}",
            id   = d.id,
            name = d.name,
            desc = d.description.as_deref().unwrap_or("-"),
        );
    }
    println!();

    print!("Enter device ID or name: ");
    io::stdout().flush()?;
    let mut dev_input = String::new();
    io::stdin().read_line(&mut dev_input)?;
    let query = dev_input.trim();

    let result = if is_output {
        client.set_default_output_device(query).await
    } else {
        client.set_default_input_device(query).await
    };

    match result {
        Ok(()) => println!("Default {} device set to '{query}'.", if is_output { "output" } else { "input" }),
        Err(e) => println!("Error: {e}"),
    }

    Ok(())
}
