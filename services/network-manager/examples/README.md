# NetworkManager Examples

Run any example with:

```bash
cargo run --example <name> -- [args]
```

Examples included:

- `list_wifi_devices`
  - Shows detected Wi-Fi interfaces and basic device state.
- `toggle_wifi [true|false]`
  - Toggles Wi-Fi when no argument is provided, or forces a specific state.
- `scan_access_points [wait-seconds]`
  - Triggers a scan, waits (default `3s`), then prints APs sorted by strength.
- `active_access_point`
  - Shows currently connected AP on the default Wi-Fi device.
- `connect_open <ssid>`
  - Connects to an open network.
- `connect_psk <ssid> <passphrase>`
  - Connects to a WPA-PSK network.
- `disconnect_wifi`
  - Disconnects the active Wi-Fi connection on the default device.
- `list_saved_connections`
  - Prints stored NetworkManager connection profiles.
- `forget_saved_connection <uuid>`
  - Deletes a stored profile by UUID.
- `list_active_connections`
  - Prints currently active connections.

Typical self-test flow:

1. `cargo run --example list_wifi_devices`
2. `cargo run --example toggle_wifi true`
3. `cargo run --example scan_access_points -- 4`
4. `cargo run --example connect_psk -- "MyWiFi" "my-passphrase"`
5. `cargo run --example active_access_point`
6. `cargo run --example list_active_connections`
7. `cargo run --example list_saved_connections`
