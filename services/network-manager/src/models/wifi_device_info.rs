use crate::types::{NMDeviceState, NMDeviceWifiCapabilities};

#[derive(Debug, Clone)]
pub struct WifiDeviceInfo {
    pub path: String,
    pub interface: String,
    pub driver: String,
    pub hw_address: String,
    pub state: NMDeviceState,
    pub managed: bool,
    pub wireless_capabilities: NMDeviceWifiCapabilities,
}
