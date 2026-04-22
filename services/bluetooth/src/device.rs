use zbus::Connection;

use crate::{
    dbus::{Battery1Proxy, Device1Proxy},
    error::BluetoothError,
};

/// A controller used to issue commands to a specific remote Bluetooth device.
pub struct Device {
    pub path: String,
    proxy: Device1Proxy<'static>,
    battery_proxy: Option<Battery1Proxy<'static>>,
}

impl Device {
    /// Creates a device handle for the given BlueZ device object path.
    pub async fn new(connection: Connection, path: &str) -> Result<Self, BluetoothError> {
        let path_string = path.to_string();
        let proxy = Device1Proxy::new(&connection, path_string.clone()).await?;
        let battery_proxy = Battery1Proxy::new(&connection, path_string.clone())
            .await
            .ok();

        Ok(Self {
            path: path_string,
            proxy,
            battery_proxy,
        })
    }

    // ---------- Identity & Info ----------

    /// Returns the device name.
    pub async fn name(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.name().await?)
    }

    /// Returns the device alias.
    pub async fn alias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.alias().await?)
    }

    /// Sets the device alias.
    pub async fn set_alias(&self, alias: &str) -> Result<(), BluetoothError> {
        self.proxy.set_alias(alias).await?;
        Ok(())
    }

    /// Returns battery percentage when supported, otherwise `None`.
    pub async fn battery_percentage(&self) -> Result<Option<u8>, BluetoothError> {
        let Some(proxy) = self.battery_proxy.as_ref() else {
            return Ok(None);
        };
        Ok(Some(proxy.percentage().await?))
    }

    // ---------- Core Status ----------

    /// Returns whether the device is bonded.
    pub async fn is_bonded(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.bonded().await?)
    }

    /// Returns whether the device services have been resolved.
    pub async fn are_services_resolved(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.services_resolved().await?)
    }

    // ---------- Trust & Blocking ----------

    /// Returns whether the device is trusted.
    pub async fn is_trusted(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.trusted().await?)
    }

    /// Marks the device as trusted or untrusted.
    pub async fn set_trusted(&self, trusted: bool) -> Result<(), BluetoothError> {
        self.proxy.set_trusted(trusted).await?;
        Ok(())
    }

    /// Returns whether the device is blocked.
    pub async fn is_blocked(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.blocked().await?)
    }

    /// Blocks or unblocks the device.
    pub async fn set_blocked(&self, blocked: bool) -> Result<(), BluetoothError> {
        self.proxy.set_blocked(blocked).await?;
        Ok(())
    }

    // ---------- Connection ----------

    /// Returns whether the device is currently connected.
    pub async fn is_connected(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.connected().await?)
    }

    /// Connects to the device.
    pub async fn connect(&self) -> Result<(), BluetoothError> {
        self.proxy.connect().await?;
        Ok(())
    }

    /// Disconnects from the device if currently connected.
    pub async fn disconnect(&self) -> Result<(), BluetoothError> {
        if self.proxy.connected().await? {
            self.proxy.disconnect().await?;
        }
        Ok(())
    }

    // ---------- Pairing ----------

    /// Returns whether the device is paired.
    pub async fn is_paired(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.paired().await?)
    }

    /// Pairs with the device and sets its trusted state.
    pub async fn pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        self.proxy.pair().await?;
        self.proxy.set_trusted(trusted).await?;
        Ok(())
    }

    /// Cancels an ongoing pairing operation.
    pub async fn cancel_pairing(&self) -> Result<(), BluetoothError> {
        self.proxy.cancel_pairing().await?;
        Ok(())
    }

    /// Connects if possible, pairing first when required.
    pub async fn connect_or_pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        if self.proxy.connected().await? {
            return Ok(());
        }

        if !self.proxy.paired().await? {
            self.proxy.pair().await?;
            self.proxy.set_trusted(trusted).await?;
        }

        self.proxy.connect().await?;
        Ok(())
    }

    // ---------- Hardware & Power ----------

    /// Enables or disables wake-from-device behavior.
    pub async fn set_wake_allowed(&self, allowed: bool) -> Result<(), BluetoothError> {
        self.proxy.set_wake_allowed(allowed).await?;
        Ok(())
    }
}
