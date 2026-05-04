use std::collections::HashMap;

use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, OwnedValue},
};

use crate::{
    dbus::{Battery1Proxy, Device1Proxy},
    error::BluetoothError,
};

/// A controller used to issue commands to a specific remote Bluetooth device.
pub struct Device {
    pub path: OwnedObjectPath,
    proxy: Device1Proxy<'static>,
    battery_proxy: Option<Battery1Proxy<'static>>,
}

impl Device {
    /// Creates a device handle for the given BlueZ device object path.
    pub async fn new<P>(connection: Connection, path: P) -> Result<Self, BluetoothError>
    where
        P: TryInto<OwnedObjectPath>,
        P::Error: std::fmt::Display,
    {
        let path: OwnedObjectPath = path
            .try_into()
            .map_err(|e| BluetoothError::InvalidObjectPath(e.to_string()))?;

        let proxy = Device1Proxy::new(&connection, path.clone()).await?;
        let battery_proxy = Battery1Proxy::new(&connection, path.clone()).await.ok();

        Ok(Self {
            path,
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
        Ok(self.proxy.set_alias(alias).await?)
    }

    /// Returns the device MAC address.
    pub async fn address(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.address().await?)
    }

    /// Returns the device address type (e.g. `"public"` or `"random"`).
    pub async fn address_type(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.address_type().await?)
    }

    /// Returns the Bluetooth class of the device.
    pub async fn class(&self) -> Result<u32, BluetoothError> {
        Ok(self.proxy.class().await?)
    }

    /// Returns the GAP appearance value of the device.
    pub async fn appearance(&self) -> Result<u16, BluetoothError> {
        Ok(self.proxy.appearance().await?)
    }

    /// Returns the device icon name.
    pub async fn icon(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.icon().await?)
    }

    /// Returns the device modalias string.
    pub async fn modalias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy.modalias().await?)
    }

    /// Returns the object path of the adapter this device belongs to.
    pub async fn adapter(&self) -> Result<OwnedObjectPath, BluetoothError> {
        Ok(self.proxy.adapter().await?)
    }

    // ---------- Battery ----------

    /// Returns the battery percentage when supported, otherwise `None`.
    pub async fn battery_percentage(&self) -> Result<Option<u8>, BluetoothError> {
        let Some(proxy) = self.battery_proxy.as_ref() else {
            return Ok(None);
        };
        Ok(Some(proxy.percentage().await?))
    }

    // ---------- Signal & Power ----------

    /// Returns the received signal strength indicator (RSSI).
    pub async fn rssi(&self) -> Result<i16, BluetoothError> {
        Ok(self.proxy.rssi().await?)
    }

    /// Returns the TX power level.
    pub async fn tx_power(&self) -> Result<i16, BluetoothError> {
        Ok(self.proxy.tx_power().await?)
    }

    // ---------- Advertisement Data ----------

    /// Returns the advertised service UUIDs.
    pub async fn uuids(&self) -> Result<Vec<String>, BluetoothError> {
        Ok(self.proxy.uuids().await?)
    }

    /// Returns manufacturer-specific advertisement data.
    pub async fn manufacturer_data(&self) -> Result<HashMap<u16, OwnedValue>, BluetoothError> {
        Ok(self.proxy.manufacturer_data().await?)
    }

    /// Returns service-specific advertisement data.
    pub async fn service_data(&self) -> Result<HashMap<String, OwnedValue>, BluetoothError> {
        Ok(self.proxy.service_data().await?)
    }

    // ---------- Core Status ----------

    /// Returns whether the device is bonded.
    pub async fn is_bonded(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.bonded().await?)
    }

    /// Returns whether the device uses legacy pairing.
    pub async fn legacy_pairing(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.legacy_pairing().await?)
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
        Ok(self.proxy.set_trusted(trusted).await?)
    }

    /// Returns whether the device is blocked.
    pub async fn is_blocked(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.blocked().await?)
    }

    /// Blocks or unblocks the device.
    pub async fn set_blocked(&self, blocked: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_blocked(blocked).await?)
    }

    // ---------- Connection ----------

    /// Returns whether the device is currently connected.
    pub async fn is_connected(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.connected().await?)
    }

    /// Connects to the device.
    ///
    /// Returns [`BluetoothError::DeviceAlreadyConnected`] if already connected.
    pub async fn connect(&self) -> Result<(), BluetoothError> {
        if self.is_connected().await? {
            return Err(BluetoothError::DeviceAlreadyConnected(
                self.path.to_string(),
            ));
        }
        Ok(self.proxy.connect().await?)
    }

    /// Disconnects from the device.
    ///
    /// Returns [`BluetoothError::DeviceNotConnected`] if not currently connected.
    pub async fn disconnect(&self) -> Result<(), BluetoothError> {
        if !self.is_connected().await? {
            return Err(BluetoothError::DeviceNotConnected(self.path.to_string()));
        }
        Ok(self.proxy.disconnect().await?)
    }

    // ---------- Pairing ----------

    /// Returns whether the device is paired.
    pub async fn is_paired(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy.paired().await?)
    }

    /// Pairs with the device and optionally marks it as trusted.
    ///
    /// Returns [`BluetoothError::DeviceAlreadyPaired`] if already paired.
    pub async fn pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        if self.is_paired().await? {
            return Err(BluetoothError::DeviceAlreadyPaired(self.path.to_string()));
        }
        self.proxy.pair().await?;
        self.proxy.set_trusted(trusted).await?;
        Ok(())
    }

    /// Cancels an ongoing pairing operation.
    pub async fn cancel_pairing(&self) -> Result<(), BluetoothError> {
        Ok(self.proxy.cancel_pairing().await?)
    }

    /// Connects to the device, pairing first if not already paired.
    ///
    /// Returns immediately if the device is already connected.
    pub async fn connect_or_pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        if self.is_connected().await? {
            return Ok(());
        }

        if !self.is_paired().await? {
            self.pair(trusted).await?;
        }

        self.proxy.connect().await?;
        Ok(())
    }

    // ---------- Hardware & Power ----------

    /// Enables or disables wake-from-device behavior.
    pub async fn set_wake_allowed(&self, allowed: bool) -> Result<(), BluetoothError> {
        Ok(self.proxy.set_wake_allowed(allowed).await?)
    }
}
