# NetworkManager D-Bus Proxies

This directory contains Rust code for interacting with the NetworkManager daemon over D-Bus, utilizing the `zbus` crate.

These files are generated from static D-Bus XML introspection files to keep the module tree clean and separated.

## Architecture

To ensure reproducible builds regardless of the host machine's current network state, we use **Vendored XML Introspection**:

1. **The Source of Truth (`/xml`):** The raw D-Bus XML interface definitions are dumped from a live system and stored in the root `xml/` directory.
2. **Automatic Generation:** We use the `zbus-xmlgen` CLI tool to parse the XML and output the generated Rust traits into distinct `.rs` files.

## How to Update or Add Proxies

If the upstream NetworkManager API changes, or if you need to expose a new interface, follow this workflow:

### Step 1: Dump the Live XML
You must dump the updated XML from a live system using `busctl`.

**1. Root Interfaces (`Manager`, `Settings`)**
These live at static, unchanging paths.

    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager --xml > xml/nm-manager.xml
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Settings --xml > xml/nm-settings.xml

**2. Device & Dynamic Interfaces**
When you run `busctl tree org.freedesktop.NetworkManager`, you will see multiple IDs for devices, access points, and connections (e.g., `Devices/1`, `Devices/2`, `AccessPoint/42`). 

**It does not matter which ID you pick.** Every object under a specific category implements the exact same D-Bus interface blueprint. You can safely pick the first available ID for each category.

*⚠️ **NOTE:** To capture `AccessPoint` or `ActiveConnection` interfaces, you MUST be actively connected to a network (specifically Wi-Fi for Access Points) when running this command.*

    # 1. Base Device (e.g., picking ID 2)
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Devices/2 --xml > xml/nm-device.xml

    # 2. Access Point (e.g., picking ID 1)
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/AccessPoint/1 --xml > xml/nm-access-point.xml

    # 3. Active Connection (e.g., picking ID 1)
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/ActiveConnection/1 --xml > xml/nm-active-connection.xml

    # 4. Settings Connection (e.g., picking ID 1)
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Settings/1 --xml > xml/nm-settings-connection.xml

    # 5. IP4 / IP6 Configs (e.g., picking ID 1)
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/IP4Config/1 --xml > xml/nm-ip4-config.xml
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/IP6Config/1 --xml > xml/nm-ip6-config.xml

**3. Hardware-Specific Interfaces (e.g., `Device.Wireless`)**
In D-Bus, an object path only exposes interfaces for the hardware it actually represents. If you introspect a generic device ID and it is missing `org.freedesktop.NetworkManager.Device.Wireless`, that means the ID you picked is likely your Ethernet or Loopback interface.
To get Wi-Fi specific methods, you must find the exact ID of your Wi-Fi card:

    # 1. Check your device IDs until you find the one that implements 'Wireless'
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Devices/3 | grep Wireless

    # 2. Once found, dump that specific ID
    busctl introspect org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Devices/3 --xml > xml/nm-device-wireless.xml

### Step 2: Generate the Rust Code
Use `zbus-xmlgen` to convert the static XML files into Rust code. When run without redirecting the output, `zbus-xmlgen` automatically reads the D-Bus interface names inside the XML and generates perfectly named `snake_case.rs` files for every interface it finds.

    # 1. Move into the target directory
    cd src/dbus

    # 2. Generate the Rust files
    zbus-xmlgen file ../../xml/nm-manager.xml
    zbus-xmlgen file ../../xml/nm-settings.xml
    zbus-xmlgen file ../../xml/nm-device.xml
    zbus-xmlgen file ../../xml/nm-device-wireless.xml
    zbus-xmlgen file ../../xml/nm-access-point.xml
    zbus-xmlgen file ../../xml/nm-settings-connection.xml
    zbus-xmlgen file ../../xml/nm-active-connection.xml
    zbus-xmlgen file ../../xml/nm-ip4-config.xml
    zbus-xmlgen file ../../xml/nm-ip6-config.xml

*Note: Because `zbus-xmlgen` auto-generates the filenames, make sure your `src/dbus/mod.rs` is updated to expose the newly created modules.*


## ⚠️ The zbus Gotchas (Mandatory Manual Fixes)

Because D-Bus and Rust have different naming conventions and rules, `zbus-xmlgen` produces code that will fail to compile or crash at runtime without the following manual adjustments.

### Fix 1: `assume_defaults` (Runtime Crash)
By default, `zbus-xmlgen` adds `assume_defaults = true` to the generated proxy macros. If you leave this, your code will crash at runtime because `zbus` will assume the service name matches the interface (e.g., `org.freedesktop.NetworkManager.Device`) instead of the actual daemon service (`org.freedesktop.NetworkManager`).

**For dynamic objects (Devices, AccessPoints, Connections, IP Configs):**
Define the `default_service` only. Do *not* define a `default_path`.

❌ WRONG:
```rust
#[dbus_proxy(interface = "org.freedesktop.NetworkManager.Device", assume_defaults = true)]
```

✅ CORRECT:
```rust
#[dbus_proxy(interface = "org.freedesktop.NetworkManager.Device", default_service = "org.freedesktop.NetworkManager")]
```

**For root objects (`network_manager.rs` and `network_manager_settings.rs` or `settings.rs`):**
Define both `default_service` and `default_path`.

✅ CORRECT (Root Manager):
```rust
#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager", 
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
```

✅ CORRECT (Settings Manager):
```rust
#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager.Settings", 
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
```

### Fix 2: Naming Collisions (Compiler Error)
`zbus` auto-generates event stream methods. For a property named `state`, it generates `receive_state_changed()`. For a signal named `state_changed`, it also generates `receive_state_changed()`. NetworkManager interfaces (specifically `ActiveConnection`, `Device`, and the root `Manager`) have both, causing duplicate definition compiler panics. NetworkManager also has a `state()` method, further polluting the namespace.

You must rename the Rust functions and map them to the correct D-Bus names using the `name` attribute.

**Fixing the Signal:**

❌ WRONG:
```rust
/// StateChanged signal
#[dbus_proxy(signal)]
fn state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;
```

✅ CORRECT:
```rust
/// StateChanged signal
#[dbus_proxy(signal, name = "StateChanged")]
fn signal_state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;
```

**Fixing the Method (if applicable, e.g., in `network_manager.rs`):**
❌ WRONG:
```rust
/// state method
fn state(&self) -> zbus::Result<u32>;
```

✅ CORRECT:
```rust
/// state method
#[dbus_proxy(name = "state")]
fn call_state(&self) -> zbus::Result<u32>;
```

### Fix 3: HashMap Trait Bounds (Compiler Error)
In `network_manager.rs`, `zbus` struggles to map `std::collections::HashMap` types that contain references to `zvariant::Value`. The compiler will complain that the type does not implement `Into<zbus::zvariant::Value>`.

You must remove the `&` before `zbus::zvariant::Value<'_>` in method arguments like `set_global_dns_configuration`, `add_and_activate_connection`, and `add_and_activate_connection2`.

❌ WRONG:
```rust
#[dbus_proxy(property)]
fn set_global_dns_configuration(
    &self,
    value: std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
) -> zbus::Result<()>;
```

✅ CORRECT:
```rust
#[dbus_proxy(property)]
fn set_global_dns_configuration(
    &self,
    value: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>, // Removed the `&`
) -> zbus::Result<()>;
```

### Step 4: Format and Finish
Finally, format the updated code and return to the project root.

```bash
cargo fmt
cd ../../
```
