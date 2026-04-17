use zbus::Connection;

use crate::{
    dbus::{Battery1Proxy, Device1Proxy},
    error::BluetoothError,
};

/// A controller used to issue commands to a specific remote Bluetooth device.
pub struct Device {
    connection: Connection,
    pub path: String,
}

impl Device {
    pub(crate) fn new(connection: Connection, path: &str) -> Self {
        Self {
            connection,
            path: path.to_string(),
        }
    }

    /// Internal helper to construct the device proxy on demand.
    async fn proxy(&self) -> Result<Device1Proxy<'_>, BluetoothError> {
        Ok(Device1Proxy::new(&self.connection, self.path.clone()).await?)
    }

    // ---------- State ----------

    pub async fn is_connected(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.connected().await?)
    }

    pub async fn is_paired(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.paired().await?)
    }

    pub async fn is_bonded(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.bonded().await?)
    }

    pub async fn is_trusted(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.trusted().await?)
    }

    pub async fn is_blocked(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.blocked().await?)
    }

    pub async fn are_services_resolved(&self) -> Result<bool, BluetoothError> {
        Ok(self.proxy().await?.services_resolved().await?)
    }

    // ---------- Connection & Pairing ----------

    pub async fn connect(&self) -> Result<(), BluetoothError> {
        self.proxy().await?.connect().await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), BluetoothError> {
        let proxy = self.proxy().await?;
        if proxy.connected().await? {
            proxy.disconnect().await?;
        }
        Ok(())
    }

    pub async fn connect_or_pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        let proxy = self.proxy().await?;

        if proxy.connected().await? {
            return Ok(());
        }

        if !proxy.paired().await? {
            proxy.pair().await?;
            proxy.set_trusted(trusted).await?;
        }

        proxy.connect().await?;
        Ok(())
    }

    pub async fn pair(&self, trusted: bool) -> Result<(), BluetoothError> {
        let proxy = self.proxy().await?;
        proxy.pair().await?;
        proxy.set_trusted(trusted).await?;
        Ok(())
    }

    pub async fn cancel_pairing(&self) -> Result<(), BluetoothError> {
        self.proxy().await?.cancel_pairing().await?;
        Ok(())
    }

    // ---------- Settings ----------

    pub async fn set_alias(&self, alias: &str) -> Result<(), BluetoothError> {
        self.proxy().await?.set_alias(alias).await?;
        Ok(())
    }

    pub async fn set_trusted(&self, trusted: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_trusted(trusted).await?;
        Ok(())
    }

    pub async fn set_blocked(&self, blocked: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_blocked(blocked).await?;
        Ok(())
    }

    pub async fn set_wake_allowed(&self, allowed: bool) -> Result<(), BluetoothError> {
        self.proxy().await?.set_wake_allowed(allowed).await?;
        Ok(())
    }

    // --------- Info ----------

    pub async fn name(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.name().await?)
    }

    pub async fn alias(&self) -> Result<String, BluetoothError> {
        Ok(self.proxy().await?.alias().await?)
    }

    pub async fn battery_percentage(&self) -> Result<Option<u8>, BluetoothError> {
        let proxy = Battery1Proxy::new(&self.connection, self.path.clone()).await?;

        match proxy.percentage().await {
            Ok(pct) => Ok(Some(pct)),
            Err(_) => Ok(None),
        }
    }
}
