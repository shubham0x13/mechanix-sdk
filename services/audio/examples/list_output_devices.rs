///   cargo run -p audio --example list_output_devices

use audio::AudioClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AudioClient::new()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let devices = client.list_output_devices();
    if devices.is_empty() {
        println!("No output devices found.");
        return Ok(());
    }

    println!("Output devices:\n");
    for d in &devices {
        println!(
            "  [{id}] {name}{default}\n       {desc}\n       volume: {vol:.0}%  muted: {mute}\n",
            id      = d.id,
            name    = d.name,
            default = if d.is_default { "  ← default" } else { "" },
            desc    = d.description.as_deref().unwrap_or("(no description)"),
            vol     = d.volume * 100.0,
            mute    = d.muted,
        );
    }

    Ok(())
}
