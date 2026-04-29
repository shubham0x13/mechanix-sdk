use std::collections::HashMap;

use uuid::Uuid;
use zbus::{
    Connection,
    zvariant::{ObjectPath, Value},
};

use crate::{
    WifiAccessPointInfo,
    dbus::{AccessPointProxy, DeviceProxy, NetworkManagerProxy, WirelessProxy},
    error::NetworkManagerError,
    types::{NM80211ApFlags, NM80211ApSecurityFlags, NMDeviceState},
};

/// Represents a Wi-Fi capable NetworkManager device (for example `wlan0`).
pub struct WifiDevice {
    connection: Connection,
    pub path: String,
}

impl WifiDevice {
    pub(crate) fn new(connection: Connection, path: &str) -> Self {
        Self {
            connection,
            path: path.to_string(),
        }
    }

    /// Internal helper to construct the device proxy on demand.
    async fn proxy(&self) -> Result<DeviceProxy<'_>, NetworkManagerError> {
        Ok(DeviceProxy::new(&self.connection, self.path.as_str()).await?)
    }

    async fn wireless_proxy(&self) -> Result<WirelessProxy<'_>, NetworkManagerError> {
        Ok(WirelessProxy::new(&self.connection, self.path.as_str()).await?)
    }

    async fn nm_proxy(&self) -> Result<NetworkManagerProxy<'_>, NetworkManagerError> {
        Ok(NetworkManagerProxy::new(&self.connection).await?)
    }

    pub async fn interface(&self) -> Result<String, NetworkManagerError> {
        Ok(self.proxy().await?.interface().await?)
    }

    pub async fn state(&self) -> Result<NMDeviceState, NetworkManagerError> {
        let raw = self.proxy().await?.state().await?;
        Ok(NMDeviceState::try_from(raw).unwrap_or(NMDeviceState::Unknown))
    }

    pub async fn is_managed(&self) -> Result<bool, NetworkManagerError> {
        Ok(self.proxy().await?.managed().await?)
    }

    pub async fn request_scan(&self) -> Result<(), NetworkManagerError> {
        let options: HashMap<&str, &Value<'_>> = HashMap::new();
        self.wireless_proxy().await?.request_scan(options).await?;
        Ok(())
    }

    pub async fn list_access_points(
        &self,
    ) -> Result<Vec<WifiAccessPointInfo>, NetworkManagerError> {
        let ap_paths = self.wireless_proxy().await?.get_all_access_points().await?;
        let mut access_points = Vec::with_capacity(ap_paths.len());

        for ap_path in ap_paths {
            access_points.push(self.access_point_info(ap_path.as_str()).await?);
        }

        access_points.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(access_points)
    }

    pub async fn active_access_point(
        &self,
    ) -> Result<Option<WifiAccessPointInfo>, NetworkManagerError> {
        let ap_path = self.wireless_proxy().await?.active_access_point().await?;
        if ap_path.as_str() == "/" {
            return Ok(None);
        }

        let info = self.access_point_info(ap_path.as_str()).await?;
        Ok(Some(info))
    }

    pub async fn connect_open(&self, ssid: &str) -> Result<String, NetworkManagerError> {
        self.connect_internal(ssid, None, None).await
    }

    pub async fn connect_psk(
        &self,
        ssid: &str,
        passphrase: &str,
    ) -> Result<String, NetworkManagerError> {
        self.connect_internal(ssid, Some(passphrase), None).await
    }

    pub async fn connect_to_access_point_open(
        &self,
        ssid: &str,
        access_point_path: &str,
    ) -> Result<String, NetworkManagerError> {
        self.connect_internal(ssid, None, Some(access_point_path))
            .await
    }

    pub async fn connect_to_access_point_psk(
        &self,
        ssid: &str,
        passphrase: &str,
        access_point_path: &str,
    ) -> Result<String, NetworkManagerError> {
        self.connect_internal(ssid, Some(passphrase), Some(access_point_path))
            .await
    }

    pub async fn disconnect(&self) -> Result<(), NetworkManagerError> {
        let active_connection = self.proxy().await?.active_connection().await?;
        if active_connection.as_str() == "/" {
            return Ok(());
        }

        let active_path = ObjectPath::try_from(active_connection.as_str())
            .map_err(|_| NetworkManagerError::InvalidObjectPath(active_connection.to_string()))?;

        self.nm_proxy()
            .await?
            .deactivate_connection(&active_path)
            .await?;
        Ok(())
    }

    pub async fn active_connection_path(&self) -> Result<Option<String>, NetworkManagerError> {
        let active_connection = self.proxy().await?.active_connection().await?;
        if active_connection.as_str() == "/" {
            Ok(None)
        } else {
            Ok(Some(active_connection.to_string()))
        }
    }

    fn ssid_from_raw(raw: &[u8]) -> String {
        String::from_utf8_lossy(raw).into_owned()
    }

    async fn connect_internal(
        &self,
        ssid: &str,
        passphrase: Option<&str>,
        explicit_ap_path: Option<&str>,
    ) -> Result<String, NetworkManagerError> {
        let nm = self.nm_proxy().await?;
        let device_path = ObjectPath::try_from(self.path.as_str())
            .map_err(|_| NetworkManagerError::InvalidObjectPath(self.path.clone()))?;

        let discovered_ap = if explicit_ap_path.is_none() {
            self.find_access_point_by_ssid(ssid).await?
        } else {
            None
        };

        let specific_object_path = explicit_ap_path
            .map(str::to_string)
            .or(discovered_ap)
            .unwrap_or_else(|| "/".to_string());

        let specific_object =
            ObjectPath::try_from(specific_object_path.as_str()).map_err(|_| {
                NetworkManagerError::InvalidObjectPath(specific_object_path.to_string())
            })?;

        let profile_name = format!("WiFi {ssid}");
        let profile_uuid = Uuid::new_v4().to_string();

        let v_conn_id = Value::from(profile_name.as_str());
        let v_conn_uuid = Value::from(profile_uuid.as_str());
        let v_conn_type = Value::from("802-11-wireless");
        let v_autoconnect = Value::from(true);
        let v_wifi_mode = Value::from("infrastructure");
        let v_wifi_ssid = Value::from(ssid.as_bytes().to_vec());

        let mut connection: HashMap<&str, &Value<'_>> = HashMap::new();
        connection.insert("id", &v_conn_id);
        connection.insert("uuid", &v_conn_uuid);
        connection.insert("type", &v_conn_type);
        connection.insert("autoconnect", &v_autoconnect);

        let mut wifi: HashMap<&str, &Value<'_>> = HashMap::new();
        wifi.insert("mode", &v_wifi_mode);
        wifi.insert("ssid", &v_wifi_ssid);

        let mut profile: HashMap<&str, HashMap<&str, &Value<'_>>> = HashMap::new();
        profile.insert("connection", connection);
        profile.insert("802-11-wireless", wifi);

        let v_key_mgmt = Value::from("wpa-psk");
        let v_psk = passphrase.map(Value::from);
        if let Some(psk) = &v_psk {
            let mut security: HashMap<&str, &Value<'_>> = HashMap::new();
            security.insert("key-mgmt", &v_key_mgmt);
            security.insert("psk", psk);
            profile.insert("802-11-wireless-security", security);
        }

        let (_, active_path) = nm
            .add_and_activate_connection(profile, &device_path, &specific_object)
            .await?;
        Ok(active_path.to_string())
    }

    async fn find_access_point_by_ssid(
        &self,
        expected_ssid: &str,
    ) -> Result<Option<String>, NetworkManagerError> {
        let expected = Self::normalize_ssid(expected_ssid);
        let ap_paths = self.wireless_proxy().await?.get_all_access_points().await?;

        for ap_path in ap_paths {
            let ap_proxy = AccessPointProxy::new(&self.connection, ap_path.as_str()).await?;

            let ssid = Self::ssid_from_raw(&ap_proxy.ssid().await?);
            if Self::normalize_ssid(&ssid) == expected {
                return Ok(Some(ap_path.to_string()));
            }
        }

        Ok(None)
    }

    async fn access_point_info(
        &self,
        access_point_path: &str,
    ) -> Result<WifiAccessPointInfo, NetworkManagerError> {
        let proxy = AccessPointProxy::new(&self.connection, access_point_path).await?;

        Ok(WifiAccessPointInfo {
            path: access_point_path.to_string(),
            ssid: Self::ssid_from_raw(&proxy.ssid().await?),
            bssid: proxy.hw_address().await?,
            strength: proxy.strength().await?,
            frequency: proxy.frequency().await?,
            max_bitrate: proxy.max_bitrate().await?,
            flags: NM80211ApFlags::from_bits_truncate(proxy.flags().await?),
            wpa_flags: NM80211ApSecurityFlags::from_bits_truncate(proxy.wpa_flags().await?),
            rsn_flags: NM80211ApSecurityFlags::from_bits_truncate(proxy.rsn_flags().await?),
            last_seen: proxy.last_seen().await?,
        })
    }

    fn normalize_ssid(ssid: &str) -> String {
        ssid.trim().to_ascii_lowercase()
    }
}
