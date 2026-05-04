use zbus::{
    Connection,
    zvariant::{ObjectPath, OwnedObjectPath},
};

use crate::{dbus::Adapter1Proxy, error::BluetoothError};

/// Represents a local Bluetooth adapter (e.g., `hci0`) and controls radio functions.
pub struct Adapter {
    pub path: OwnedObjectPath,
    proxy: Adapter1Proxy<'static>,
}

impl Adapter {
    /// Creates an adapter handle for the given BlueZ adapter object path.
    pub async fn new<P>(connection: Connection, path: P) -> Result<Self, BluetoothError>
    where
        P: TryInto<OwnedObjectPath>,
        P::Error: std::fmt::Display,
    {
        let path: OwnedObjectPath = path
            .try_into()
            .map_err(|e| BluetoothError::InvalidObjectPath(e.to_string()))?;

        let proxy = Adapter1Proxy::new(&connection, path.clone()).await?;

        Ok(Self { path, proxy })
    }

    // ---------- Identity ----------

    /// Returns the human-readable adapter name from BlueZ.
    pub async fn name(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.name().await?)
    }

    /// Returns the adapter alias.
    pub async fn alias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.alias().await?)
    }

    /// Sets the adapter alias.
    pub async fn set_alias(&self, alias: &str) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_alias(alias).await?)
    }

    /// Returns the adapter Bluetooth MAC address.
    pub async fn address(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.address().await?)
    }

    /// Returns the adapter address type.
    pub async fn address_type(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.address_type().await?)
    }

    /// Returns the Bluetooth class of the adapter.
    pub async fn class(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.class().await?)
    }

    /// Returns the manufacturer ID reported by the adapter.
    pub async fn manufacturer(&self) -> Result<u16, BluetoothError> {
        Ok(self.proxy.manufacturer().await?)
    }

    /// Returns the adapter firmware/hardware version.
    pub async fn version(&self) -> Result<u8, BluetoothError> {
        Ok(self.proxy.version().await?)
    }

    /// Returns the kernel modalias for the adapter.
    pub async fn modalias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.modalias().await?)
    }

    // ---------- UUIDs & Roles ----------

    /// Returns the advertised service UUIDs.
    pub async fn uuids(&self) -> Result<Vec<String>, BluetoothError> {
        Ok(self.proxy.uuids().await?)
    }

    /// Returns the supported roles (e.g. `"central"`, `"peripheral"`).
    pub async fn roles(&self) -> Result<Vec<String>, BluetoothError> {
        Ok(self.proxy.roles().await?)
    }

    /// Returns the experimental features enabled on the adapter.
    pub async fn experimental_features(&self) -> Result<Vec<String>, BluetoothError> {
        Ok(self.proxy.experimental_features().await?)
    }

    // ---------- Power ----------

    /// Returns whether the adapter is currently powered on.
    pub async fn is_powered(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.powered().await?)
    }

    /// Sets the adapter power state.
    pub async fn set_powered(&self, powered: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_powered(powered).await?)
    }

    /// Toggles adapter power and returns the new state.
    pub async fn toggle_power(&self) -> Result<bool, BluetoothError> {
        let new_state = !self.is_powered().await?;
        self.proxy.set_powered(new_state).await?;
        Ok(new_state)
    }

    /// Returns the current power state string (e.g. `"on"`, `"off"`, `"off-enabling"`).
    pub async fn power_state(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.power_state().await?)
    }

    // ---------- Discovery ----------

    /// Returns whether active device discovery is running.
    pub async fn is_discovering(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.discovering().await?)
    }

    /// Starts discovery for nearby Bluetooth devices.
    ///
    /// Returns [`BluetoothError::AdapterPoweredOff`] if the adapter is off.
    pub async fn start_discovery(&self) -> Result<(), BluetoothError> {
        if !self.is_powered().await? {
            return Err(BluetoothError::AdapterPoweredOff);
        }
        Ok(self.proxy.start_discovery().await?)
    }

    /// Stops ongoing device discovery.
    pub async fn stop_discovery(&self) -> Result<(), BluetoothError> {
        Ok(self.proxy.stop_discovery().await?)
    }

    // ---------- Visibility ----------

    /// Returns whether the adapter is currently discoverable.
    pub async fn is_discoverable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.discoverable().await?)
    }

    /// Enables or disables adapter discoverability.
    pub async fn set_discoverable(&self, discoverable: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_discoverable(discoverable).await?)
    }

    /// Returns the discoverable timeout in seconds.
    pub async fn discoverable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.discoverable_timeout().await?)
    }

    /// Sets the discoverable timeout in seconds.
    pub async fn set_discoverable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_discoverable_timeout(timeout_secs).await?)
    }

    // ---------- Pairing ----------

    /// Returns whether the adapter is currently pairable.
    pub async fn is_pairable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.pairable().await?)
    }

    /// Enables or disables pairing mode on the adapter.
    pub async fn set_pairable(&self, pairable: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_pairable(pairable).await?)
    }

    /// Returns the pairable timeout in seconds.
    pub async fn pairable_timeout(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.pairable_timeout().await?)
    }

    /// Sets the pairable timeout in seconds.
    pub async fn set_pairable_timeout(&self, timeout_secs: u32) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_pairable_timeout(timeout_secs).await?)
    }

    // ---------- Connectability ----------

    /// Returns whether incoming connections are allowed.
    pub async fn is_connectable(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.connectable().await?)
    }

    /// Enables or disables adapter connectability.
    pub async fn set_connectable(&self, connectable: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_connectable(connectable).await?)
    }

    // ---------- Device Management ----------

    /// Removes a previously paired device by object path.
    ///
    /// Returns [`BluetoothError::DeviceNotFound`] if the path cannot be resolved.
    pub async fn forget_device<'a, T>(&self, device_path: T) -> Result<(), BluetoothError>
    where
        T: TryInto<ObjectPath<'a>>,
        T::Error: std::fmt::Display,
    {
        let path = device_path
            .try_into()
            .map_err(|e| BluetoothError::DeviceNotFound(e.to_string()))?;

        Ok(self.proxy.remove_device(&path).await?)
    }
}
