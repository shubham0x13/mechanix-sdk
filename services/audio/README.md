# Mechanix Audio

This crate handles system-level audio for Mechanix, acting as a wrapper around PipeWire. It lets you list devices, change volumes, and toggle mute states without having to deal with the PipeWire C API directly.

## How it works

The architecture is built around a background event loop to keep the main application responsive.

### The Background Thread
When you create an `AudioClient`, it spawns a dedicated thread (`mechanix-pw`). This thread:
1. Connects to the PipeWire daemon.
2. Watches for registry events (new speakers or mics being plugged in/out).
3. Listens for volume and mute changes from the system.
4. Updates a shared state cache (`SharedState`) so reads are fast.

### The Client Handle
`AudioClient` is just a handle you can clone and pass around.
- **Reads**: Functions like `list_devices()` or `volume()` read directly from the cached state in RAM.
- **Writes**: Functions like `set_volume()` send a command to the background thread and wait for a reply. These are `async` because they might involve a tiny bit of round-trip time to the PipeWire daemon.

## Examples

### Listing Devices
```rust
use audio::AudioClient;

let client = AudioClient::new()?;

// Give the background thread a moment to sync with PipeWire
tokio::time::sleep(std::time::Duration::from_millis(300)).await;

let devices = client.list_output_devices();
for device in devices {
    println!("ID: {}, Name: {}, Volume: {}", device.id, device.name, device.volume);
}
```

### Changing Volume
```rust
// Set volume to 75% for a specific device. 
// Note: this is an async call.
client.set_volume("alsa_output.pci-0000_00_1f.3.analog-stereo", 0.75).await?;
```

### Setting the Default Output
```rust
// Make a specific device the system default
client.set_default_output_device("my-bluetooth-headphones").await?;
```

