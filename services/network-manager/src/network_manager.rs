use futures::Stream;
use std::collections::HashMap;
use zbus::{
    Connection, MatchRule,
    message::Type,
    zvariant::{OwnedObjectPath, OwnedValue},
};

use common::dbus::create_event_stream;

use crate::{
    ActiveConnectionInfo, NetworkManagerEvent, SavedConnectionInfo, WifiDeviceInfo,
    dbus::{
        AccessPointProxy, ActiveProxy, ConnectionProxy, DeviceProxy, NetworkManagerProxy,
        SettingsProxy, WirelessProxy,
    },
    error::NetworkManagerError,
    types::{
        NMActiveConnectionState, NMActiveConnectionStateReason, NMConnectivityState, NMDeviceState,
        NMDeviceStateReason, NMDeviceType, NMDeviceWifiCapabilities, NMState,
    },
    wifi_device::WifiDevice,
};

// ---------- D-Bus Constants ----------
const NM_DEST: &str = "org.freedesktop.NetworkManager";
const IFACE_NM: &str = "org.freedesktop.NetworkManager";
const IFACE_DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const IFACE_WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const IFACE_ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const IFACE_PROPERTIES: &str = "org.freedesktop.DBus.Properties";

#[derive(Clone)]
pub struct NetworkManager {
    connection: Connection,
}

impl NetworkManager {
    pub async fn new() -> Result<Self, NetworkManagerError> {
        let connection = Connection::system().await?;
        Ok(Self { connection })
    }

    async fn nm_proxy(&self) -> Result<NetworkManagerProxy<'_>, NetworkManagerError> {
        Ok(NetworkManagerProxy::new(&self.connection).await?)
    }

    async fn settings_proxy(&self) -> Result<SettingsProxy<'_>, NetworkManagerError> {
        Ok(SettingsProxy::new(&self.connection).await?)
    }

    /// Returns a controller for a specific Wi-Fi device path.
    pub fn wifi_device(&self, path: &str) -> WifiDevice {
        WifiDevice::new(self.connection.clone(), path)
    }

    pub async fn default_wifi_device(&self) -> Result<WifiDevice, NetworkManagerError> {
        let first = self
            .list_wifi_devices()
            .await?
            .into_iter()
            .next()
            .ok_or(NetworkManagerError::NoWifiDevice)?;
        Ok(self.wifi_device(&first.path))
    }

    // ---------- Radio & Global Network State ----------

    pub async fn is_networking_enabled(&self) -> Result<bool, NetworkManagerError> {
        Ok(self.nm_proxy().await?.networking_enabled().await?)
    }

    pub async fn set_networking_enabled(&self, enabled: bool) -> Result<(), NetworkManagerError> {
        self.nm_proxy().await?.enable(enabled).await?;
        Ok(())
    }

    pub async fn is_wifi_enabled(&self) -> Result<bool, NetworkManagerError> {
        Ok(self.nm_proxy().await?.wireless_enabled().await?)
    }

    pub async fn set_wifi_enabled(&self, enabled: bool) -> Result<(), NetworkManagerError> {
        self.nm_proxy().await?.set_wireless_enabled(enabled).await?;
        Ok(())
    }

    pub async fn toggle_wifi(&self) -> Result<bool, NetworkManagerError> {
        let proxy = self.nm_proxy().await?;
        let current = proxy.wireless_enabled().await?;
        proxy.set_wireless_enabled(!current).await?;
        Ok(!current)
    }

    pub async fn is_wifi_hardware_enabled(&self) -> Result<bool, NetworkManagerError> {
        Ok(self.nm_proxy().await?.wireless_hardware_enabled().await?)
    }

    pub async fn check_connectivity(&self) -> Result<NMConnectivityState, NetworkManagerError> {
        // Triggers Network Manager to actively check connectivity (HTTP ping to a captive portal server)
        let raw = self.nm_proxy().await?.check_connectivity().await?;
        Ok(NMConnectivityState::try_from(raw).unwrap_or(NMConnectivityState::Unknown))
    }

    // ---------- Device discovery ----------

    pub async fn list_wifi_devices(&self) -> Result<Vec<WifiDeviceInfo>, NetworkManagerError> {
        let device_paths = self.nm_proxy().await?.get_devices().await?;
        let mut wifi_devices = Vec::new();

        for path in device_paths {
            let device_proxy = DeviceProxy::new(&self.connection, path.as_str()).await?;
            let device_type = NMDeviceType::try_from(device_proxy.device_type().await?)
                .unwrap_or(NMDeviceType::Unknown);

            if device_type != NMDeviceType::Wifi {
                continue;
            }

            let wireless_caps = match WirelessProxy::new(&self.connection, path.as_str()).await {
                Ok(wireless_proxy) => NMDeviceWifiCapabilities::from_bits_truncate(
                    wireless_proxy.wireless_capabilities().await.unwrap_or(0),
                ),
                Err(_) => NMDeviceWifiCapabilities::empty(),
            };

            let state = NMDeviceState::try_from(device_proxy.state().await?)
                .unwrap_or(NMDeviceState::Unknown);

            wifi_devices.push(WifiDeviceInfo {
                path: path.to_string(),
                interface: device_proxy.interface().await?,
                driver: device_proxy.driver().await?,
                hw_address: device_proxy.hw_address().await?,
                state,
                managed: device_proxy.managed().await?,
                wireless_capabilities: wireless_caps,
            });
        }

        Ok(wifi_devices)
    }

    // ---------- Saved connection management ----------

    pub async fn list_saved_connections(
        &self,
    ) -> Result<Vec<SavedConnectionInfo>, NetworkManagerError> {
        let paths = self.settings_proxy().await?.list_connections().await?;
        let mut saved = Vec::with_capacity(paths.len());

        for path in paths {
            let connection_proxy = ConnectionProxy::new(&self.connection, path.as_str()).await?;
            let settings = connection_proxy.get_settings().await?;

            if let Some(info) = SavedConnectionInfo::from_settings(path.to_string(), &settings) {
                saved.push(info);
            }
        }

        Ok(saved)
    }

    pub async fn forget_saved_connection(&self, uuid: &str) -> Result<(), NetworkManagerError> {
        let paths = self.settings_proxy().await?.list_connections().await?;

        for path in paths {
            let proxy = ConnectionProxy::new(&self.connection, path.as_str()).await?;
            let settings = proxy.get_settings().await?;

            let found_uuid = settings
                .get("connection")
                .and_then(|conn| conn.get("uuid"))
                .and_then(|value| <&str>::try_from(value).ok())
                .map(String::from);

            if found_uuid.as_deref() == Some(uuid) {
                proxy.delete().await?;
                return Ok(());
            }
        }

        Err(NetworkManagerError::SavedConnectionNotFound(
            uuid.to_string(),
        ))
    }

    // ---------- Active connection state ----------

    pub async fn list_active_connections(
        &self,
    ) -> Result<Vec<ActiveConnectionInfo>, NetworkManagerError> {
        let active_paths = self.nm_proxy().await?.active_connections().await?;
        let mut active_connections = Vec::with_capacity(active_paths.len());

        for path in active_paths {
            let active_proxy = ActiveProxy::new(&self.connection, path.as_str()).await?;
            let state = NMActiveConnectionState::try_from(active_proxy.state().await?)
                .unwrap_or(NMActiveConnectionState::Unknown);

            active_connections.push(ActiveConnectionInfo {
                path: path.to_string(),
                id: active_proxy.id().await?,
                uuid: active_proxy.uuid().await?,
                connection_type: active_proxy.type_().await?,
                state,
                is_default_ipv4: active_proxy.default().await?,
                is_default_ipv6: active_proxy.default6().await?,
                is_vpn: active_proxy.vpn().await?,
            });
        }

        Ok(active_connections)
    }

    pub async fn version(&self) -> Result<String, NetworkManagerError> {
        Ok(self.nm_proxy().await?.version().await?)
    }

    pub async fn get_access_point_ssid(
        &self,
        ap_path: &str,
    ) -> Result<String, NetworkManagerError> {
        // Assuming AccessPointProxy is imported from your dbus module
        let proxy = AccessPointProxy::new(&self.connection, ap_path).await?;

        let ssid_bytes = proxy.ssid().await?;

        // Convert the byte array to a String, replacing invalid UTF-8 with the replacement character ()
        Ok(String::from_utf8_lossy(&ssid_bytes).to_string())
    }

    // ---------- Event Streaming & Parsing ----------

    /// Returns a resilient, auto-reconnecting Stream of NetworkManager events.
    pub fn stream_events(&self) -> impl Stream<Item = NetworkManagerEvent> + Send + 'static {
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender(NM_DEST)
            .expect("Failed to build NetworkManager match rule")
            .build();

        create_event_stream(self.connection.clone(), rule, Self::parse_nm_signal)
    }

    fn parse_nm_signal(msg: &zbus::Message) -> Vec<NetworkManagerEvent> {
        let header = msg.header();

        let member = header.member().map(|m| m.as_str());
        let interface = header.interface().map(|i| i.as_str());
        let Some(path) = header.path().map(|p| p.to_string()) else {
            return vec![];
        };

        match (interface, member) {
            (Some(IFACE_PROPERTIES), Some("PropertiesChanged")) => {
                Self::parse_properties_changed(msg, path)
            }
            (Some(IFACE_NM), Some("DeviceAdded")) => Self::parse_device_added(msg),
            (Some(IFACE_NM), Some("DeviceRemoved")) => Self::parse_device_removed(msg),
            (Some(IFACE_NM), Some("StateChanged")) => Self::parse_nm_state_changed(msg),
            (Some(IFACE_DEVICE), Some("StateChanged")) => {
                Self::parse_device_state_changed(msg, path)
            }
            (Some(IFACE_ACTIVE), Some("StateChanged")) => {
                Self::parse_active_state_changed(msg, path)
            }
            (Some(IFACE_WIRELESS), Some("AccessPointAdded")) => {
                Self::parse_access_point_added(msg, path)
            }
            (Some(IFACE_WIRELESS), Some("AccessPointRemoved")) => {
                Self::parse_access_point_removed(msg, path)
            }
            _ => vec![],
        }
    }

    fn parse_properties_changed(msg: &zbus::Message, path: String) -> Vec<NetworkManagerEvent> {
        type PropsData = (String, HashMap<String, OwnedValue>, Vec<String>);
        let mut events = Vec::new();

        let Ok((iface, changed_props, _)) = msg.body().deserialize::<PropsData>() else {
            return events;
        };

        match iface.as_str() {
            IFACE_NM => {
                if let Some(val) = changed_props.get("NetworkingEnabled")
                    && let Ok(enabled) = bool::try_from(val)
                {
                    events.push(NetworkManagerEvent::NetworkingEnabledChanged { enabled });
                }
                if let Some(val) = changed_props.get("WirelessEnabled")
                    && let Ok(enabled) = bool::try_from(val)
                {
                    events.push(NetworkManagerEvent::WifiEnabledChanged { enabled });
                }
                if let Some(val) = changed_props.get("Connectivity")
                    && let Ok(raw) = u32::try_from(val)
                {
                    let state =
                        NMConnectivityState::try_from(raw).unwrap_or(NMConnectivityState::Unknown);
                    events.push(NetworkManagerEvent::ConnectivityChanged { state });
                }
                if let Some(val) = changed_props.get("State")
                    && let Ok(raw) = u32::try_from(val)
                {
                    let state = NMState::try_from(raw).unwrap_or(NMState::Unknown);
                    events.push(NetworkManagerEvent::StateChanged { state });
                }
            }
            IFACE_WIRELESS => {
                if let Some(val) = changed_props.get("ActiveAccessPoint")
                    && let Ok(ap_path) = val.downcast_ref::<zbus::zvariant::ObjectPath<'_>>()
                {
                    let access_point = if ap_path.as_str() == "/" {
                        None
                    } else {
                        Some(ap_path.to_string())
                    };
                    events.push(NetworkManagerEvent::ActiveAccessPointChanged {
                        device_path: path,
                        access_point,
                    });
                }
            }
            _ => {}
        }

        events
    }

    fn parse_device_added(msg: &zbus::Message) -> Vec<NetworkManagerEvent> {
        let Ok(device_path) = msg.body().deserialize::<OwnedObjectPath>() else {
            return vec![];
        };

        vec![NetworkManagerEvent::DeviceAdded {
            path: device_path.to_string(),
        }]
    }

    fn parse_device_removed(msg: &zbus::Message) -> Vec<NetworkManagerEvent> {
        let Ok(device_path) = msg.body().deserialize::<OwnedObjectPath>() else {
            return vec![];
        };

        vec![NetworkManagerEvent::DeviceRemoved {
            path: device_path.to_string(),
        }]
    }

    fn parse_nm_state_changed(msg: &zbus::Message) -> Vec<NetworkManagerEvent> {
        let Ok(raw) = msg.body().deserialize::<u32>() else {
            return vec![];
        };

        let state = NMState::try_from(raw).unwrap_or(NMState::Unknown);
        vec![NetworkManagerEvent::StateChanged { state }]
    }

    fn parse_device_state_changed(msg: &zbus::Message, path: String) -> Vec<NetworkManagerEvent> {
        let Ok((new_state, old_state, reason)) = msg.body().deserialize::<(u32, u32, u32)>() else {
            return vec![];
        };

        let new_state = NMDeviceState::try_from(new_state).unwrap_or(NMDeviceState::Unknown);
        let old_state = NMDeviceState::try_from(old_state).unwrap_or(NMDeviceState::Unknown);
        let reason = NMDeviceStateReason::try_from(reason).unwrap_or(NMDeviceStateReason::Unknown);

        vec![NetworkManagerEvent::DeviceStateChanged {
            path,
            new_state,
            old_state,
            reason,
        }]
    }

    fn parse_active_state_changed(msg: &zbus::Message, path: String) -> Vec<NetworkManagerEvent> {
        let Ok((state, reason)) = msg.body().deserialize::<(u32, u32)>() else {
            return vec![];
        };

        let state =
            NMActiveConnectionState::try_from(state).unwrap_or(NMActiveConnectionState::Unknown);
        let reason = NMActiveConnectionStateReason::try_from(reason)
            .unwrap_or(NMActiveConnectionStateReason::Unknown);

        vec![NetworkManagerEvent::ActiveConnectionStateChanged {
            path,
            state,
            reason,
        }]
    }

    fn parse_access_point_added(
        msg: &zbus::Message,
        device_path: String,
    ) -> Vec<NetworkManagerEvent> {
        let Ok(ap_path) = msg.body().deserialize::<OwnedObjectPath>() else {
            return vec![];
        };

        vec![NetworkManagerEvent::AccessPointAdded {
            device_path,
            access_point: ap_path.to_string(),
        }]
    }

    fn parse_access_point_removed(
        msg: &zbus::Message,
        device_path: String,
    ) -> Vec<NetworkManagerEvent> {
        let Ok(ap_path) = msg.body().deserialize::<OwnedObjectPath>() else {
            return vec![];
        };

        vec![NetworkManagerEvent::AccessPointRemoved {
            device_path,
            access_point: ap_path.to_string(),
        }]
    }
}
