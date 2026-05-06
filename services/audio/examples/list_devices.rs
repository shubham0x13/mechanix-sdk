///   cargo run -p audio --example list_devices
use audio::AudioClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("audio=debug")
        .init();

    let client = AudioClient::new()?;

    // Give PipeWire a moment to push param events for every node.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    println!("=== Output devices ===");
    for d in client.list_output_devices() {
        println!(
            "  [{id}] {name}{default}\n       desc: {desc}\n       ch: {ch}  vol: {vol:.0}%  mute: {mute}",
            id = d.id,
            name = d.name,
            default = if d.is_default { " (default)" } else { "" },
            desc = d.description.as_deref().unwrap_or("-"),
            ch = d.channels,
            vol = d.volume * 100.0,
            mute = d.muted,
        );
    }

    println!("\n=== Input devices ===");
    for d in client.list_input_devices() {
        println!(
            "  [{id}] {name}{default}\n       desc: {desc}\n       ch: {ch}  vol: {vol:.0}%  mute: {mute}",
            id = d.id,
            name = d.name,
            default = if d.is_default { " (default)" } else { "" },
            desc = d.description.as_deref().unwrap_or("-"),
            ch = d.channels,
            vol = d.volume * 100.0,
            mute = d.muted,
        );
    }

    if let Ok(dev) = client.default_output_device() {
        println!(
            "\nDefault output: {} ({} ch, vol {:.0}%)",
            dev.name,
            dev.channels,
            dev.volume * 100.0
        );
    }

    if let Ok(dev) = client.default_input_device() {
        println!(
            "Default input:  {} ({} ch, vol {:.0}%)",
            dev.name,
            dev.channels,
            dev.volume * 100.0
        );
    }

    Ok(())
}
