///   cargo run -p audio --example default_devices
use audio::AudioClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AudioClient::new()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    match client.default_output_device() {
        Ok(d) => println!(
            "Default output:\n  [{id}] {name}\n  {desc}\n  volume: {vol:.0}%  muted: {mute}",
            id = d.id,
            name = d.name,
            desc = d.description.as_deref().unwrap_or("(no description)"),
            vol = d.volume * 100.0,
            mute = d.muted,
        ),
        Err(e) => println!("Default output: {e}"),
    }

    println!();

    match client.default_input_device() {
        Ok(d) => println!(
            "Default input:\n  [{id}] {name}\n  {desc}\n  volume: {vol:.0}%  muted: {mute}",
            id = d.id,
            name = d.name,
            desc = d.description.as_deref().unwrap_or("(no description)"),
            vol = d.volume * 100.0,
            mute = d.muted,
        ),
        Err(e) => println!("Default input: {e}"),
    }

    Ok(())
}
