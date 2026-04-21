use zbus::{Connection, zvariant::ObjectPath};

use crate::{dbus::Adapter1Proxy, error::BluetoothError};

/// Represents a local Bluetooth adapter (e.g., "hci0") and controls radio functions.
pub struct Adapter {
    connection: Connection,
    pub name: String,
    pub path: String,
}

impl Adapter {
    /// Creates an adapter handle for the given BlueZ adapter name (for example, `hci0`).
    pub(crate) fn new(connection: Connection, name: &str) -> Self {
        Self {
            connection,
            name: name.to_string(),
            path: format!("/org/bluez/{}", name),
        }
    }

    /// Internal helper to construct the proxy on demand.
    async fn proxy(&self) -> Result<Adapter1Proxy<'_>, BluetoothError> {
        Ok(Adapter1Proxy::new(&self.connection, self.path.clone()).await?)
    }

    // ---------- Power ----------

    /// Returns whether the adapter is currently powered on.
    pub async fn is_powered(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.powered().await?)
    }

    /// Sets the adapter power state.
    pub async fn set_powered(&self, powered: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_powered(powered).await?;
        Ok(())
    }

    /// Toggles adapter power and returns the new state.
    pub async fn toggle_power(&self) -> Result<bool, BluetoothError> {
        let proxy = self.proxy().await?;
        let current_state = proxy.powered().await?;
        proxy.set_powered(!current_state).await?;
        Ok(!current_state)
    }

    // ---------- Identity ----------

    /// Returns the human-readable adapter name from BlueZ.
    pub async fn name(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.name().await?)
    }

    /// Returns the adapter Bluetooth MAC address.
    pub async fn address(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.address().await?)
    }

    /// Returns the manufacturer ID reported by the adapter.
    pub async fn manufacturer(&self) -> Result<u16, BluetoothError> {
        Ok(self.proxy().await?.manufacturer().await?)
    }

    /// Returns the kernel modalias for the adapter.
    pub async fn modalias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.modalias().await?)
    }

    /// Returns the adapter alias.
    pub async fn alias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.alias().await?)
    }

    /// Sets the adapter alias.
    pub async fn set_alias(&self, alias: &str) -> Result<(), BluetoothError> {
        self.proxy().await?.set_alias(alias).await?;
        Ok(())
    }

    // --------- Discovery ----------

    /// Returns whether active device discovery is running.
    pub async fn is_discovering(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.discovering().await?)
    }

    /// Starts discovery for nearby Bluetooth devices.
    pub async fn start_discovery(&self) -> Result<(), BluetoothError> {
        self.proxy().await?.start_discovery().await?;
        Ok(())
    }

    /// Stops ongoing device discovery.
    pub async fn stop_discovery(&self) -> Result<(), BluetoothError> {
        self.proxy().await?.stop_discovery().await?;
        Ok(())
    }

    // --------- Visibility & Pairing ----------

    /// Returns whether the adapter is currently discoverable.
    pub async fn is_discoverable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.discoverable().await?)
    }

    /// Returns the discoverable timeout in seconds.
    pub async fn discoverable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy().await?.discoverable_timeout().await?)
    }

    /// Enables or disables adapter discoverability.
    pub async fn set_discoverable(&self, discoverable: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_discoverable(discoverable).await?;
        Ok(())
    }

    /// Sets the discoverable timeout in seconds.
    pub async fn set_discoverable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        self.proxy()
            .await?
            .set_discoverable_timeout(timeout_secs)
            .await?;
        Ok(())
    }

    /// Returns whether the adapter is currently pairable.
    pub async fn is_pairable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.pairable().await?)
    }

    /// Returns the pairable timeout in seconds.
    pub async fn pairable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy().await?.pairable_timeout().await?)
    }

    /// Enables or disables pairing mode on the adapter.
    pub async fn set_pairable(&self, pairable: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_pairable(pairable).await?;
        Ok(())
    }

    /// Sets the pairable timeout in seconds.
    pub async fn set_pairable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        self.proxy()
            .await?
            .set_pairable_timeout(timeout_secs)
            .await?;
        Ok(())
    }

    // --------- Connectability  ----------

    /// Returns whether incoming connections are allowed.
    pub async fn is_connectable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.connectable().await?)
    }

    /// Enables or disables adapter connectability.
    pub async fn set_connectable(&self, connectable: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_connectable(connectable).await?;
        Ok(())
    }

    // --------- Device Management ----------

    /// Removes a previously paired device by object path.
    pub async fn forget_device(&self, device_path: &str) -> Result<(), BluetoothError> {
        let proxy = self.proxy().await?;
        let path = ObjectPath::try_from(device_path)
            .map_err(|_| BluetoothError::InvalidObjectPath(device_path.to_string()))?;

        proxy.remove_device(&path).await?;
        Ok(())
    }
}
