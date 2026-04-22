use zbus::{Connection, zvariant::ObjectPath};

use crate::{dbus::Adapter1Proxy, error::BluetoothError};

/// Represents a local Bluetooth adapter (e.g., "hci0") and controls radio functions.
pub struct Adapter {
    // connection: Connection,
    pub name: String,
    pub path: String,
    proxy: Adapter1Proxy<'static>,
}

impl Adapter {
    /// Creates an adapter handle for the given BlueZ adapter name (for example, `hci0`).
    pub async fn new(connection: Connection, name: &str) -> Result<Self, BluetoothError> {
        let path = format!("/org/bluez/{}", name);
        let proxy = Adapter1Proxy::new(&connection, path.clone()).await?;

        Ok(Self {
            // connection,
            name: name.to_string(),
            path,
            proxy,
        })
    }

    // ---------- Power ----------

    /// Returns whether the adapter is currently powered on.
    pub async fn is_powered(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.powered().await?)
    }

    /// Sets the adapter power state.
    pub async fn set_powered(&self, powered: bool) -> Result<(), BluetoothError> {
        self.proxy.set_powered(powered).await?;
        Ok(())
    }

    /// Toggles adapter power and returns the new state.
    pub async fn toggle_power(&self) -> Result<bool, BluetoothError> {
        let current_state = self.proxy.powered().await?;
        self.proxy.set_powered(!current_state).await?;
        Ok(!current_state)
    }

    // ---------- Identity ----------

    /// Returns the human-readable adapter name from BlueZ.
    pub async fn name(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.name().await?)
    }

    /// Returns the adapter Bluetooth MAC address.
    pub async fn address(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.address().await?)
    }

    /// Returns the manufacturer ID reported by the adapter.
    pub async fn manufacturer(&self) -> Result<u16, BluetoothError> {
        Ok(self.proxy.manufacturer().await?)
    }

    /// Returns the kernel modalias for the adapter.
    pub async fn modalias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.modalias().await?)
    }

    /// Returns the adapter alias.
    pub async fn alias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.alias().await?)
    }

    /// Sets the adapter alias.
    pub async fn set_alias(&self, alias: &str) -> Result<(), BluetoothError> {
        self.proxy.set_alias(alias).await?;
        Ok(())
    }

    // --------- Discovery ----------

    /// Returns whether active device discovery is running.
    pub async fn is_discovering(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.discovering().await?)
    }

    /// Starts discovery for nearby Bluetooth devices.
    pub async fn start_discovery(&self) -> Result<(), BluetoothError> {
        self.proxy.start_discovery().await?;
        Ok(())
    }

    /// Stops ongoing device discovery.
    pub async fn stop_discovery(&self) -> Result<(), BluetoothError> {
        self.proxy.stop_discovery().await?;
        Ok(())
    }

    // --------- Visibility & Pairing ----------

    /// Returns whether the adapter is currently discoverable.
    pub async fn is_discoverable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.discoverable().await?)
    }

    /// Returns the discoverable timeout in seconds.
    pub async fn discoverable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.discoverable_timeout().await?)
    }

    /// Enables or disables adapter discoverability.
    pub async fn set_discoverable(&self, discoverable: bool) -> Result<(), BluetoothError> {
        self.proxy.set_discoverable(discoverable).await?;
        Ok(())
    }

    /// Sets the discoverable timeout in seconds.
    pub async fn set_discoverable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        self.proxy.set_discoverable_timeout(timeout_secs).await?;
        Ok(())
    }

    /// Returns whether the adapter is currently pairable.
    pub async fn is_pairable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.pairable().await?)
    }

    /// Returns the pairable timeout in seconds.
    pub async fn pairable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.pairable_timeout().await?)
    }

    /// Enables or disables pairing mode on the adapter.
    pub async fn set_pairable(&self, pairable: bool) -> Result<(), BluetoothError> {
        self.proxy.set_pairable(pairable).await?;
        Ok(())
    }

    /// Sets the pairable timeout in seconds.
    pub async fn set_pairable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        self.proxy.set_pairable_timeout(timeout_secs).await?;
        Ok(())
    }

    // --------- Connectability  ----------

    /// Returns whether incoming connections are allowed.
    pub async fn is_connectable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.connectable().await?)
    }

    /// Enables or disables adapter connectability.
    pub async fn set_connectable(&self, connectable: bool) -> Result<(), BluetoothError> {
        self.proxy.set_connectable(connectable).await?;
        Ok(())
    }

    // --------- Device Management ----------

    /// Removes a previously paired device by object path.
    pub async fn forget_device(&self, device_path: &str) -> Result<(), BluetoothError> {
        let path = ObjectPath::try_from(device_path)
            .map_err(|_| BluetoothError::InvalidObjectPath(device_path.to_string()))?;

        self.proxy.remove_device(&path).await?;
        Ok(())
    }
}
