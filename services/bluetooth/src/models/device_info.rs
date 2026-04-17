use crate::DbusProperties;
use common::{extract_bool, extract_i16, extract_string, extract_u32};

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: String,
    pub name: Option<String>,
    pub alias: String,
    pub address: String,
    pub icon: Option<String>,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
    pub blocked: bool,
    pub wake_allowed: bool,
    pub services_resolved: bool,
    pub rssi: Option<i16>,
    pub battery_percentage: Option<u8>,
}

impl DeviceInfo {
    pub(crate) fn from_properties(path: String, props: &DbusProperties) -> Self {
        Self {
            path,
            name: extract_string(props, "Name"),
            alias: extract_string(props, "Alias").unwrap_or_default(),
            address: extract_string(props, "Address").unwrap_or_default(),
            icon: extract_string(props, "Icon"),
            paired: extract_bool(props, "Paired").unwrap_or(false),
            connected: extract_bool(props, "Connected").unwrap_or(false),
            trusted: extract_bool(props, "Trusted").unwrap_or(false),
            blocked: extract_bool(props, "Blocked").unwrap_or(false),
            wake_allowed: extract_bool(props, "WakeAllowed").unwrap_or(false),
            services_resolved: extract_bool(props, "ServicesResolved").unwrap_or(false),
            rssi: extract_i16(props, "RSSI"),
            battery_percentage: extract_u32(props, "BatteryPercentage").map(|v| v as u8),
        }
    }

    pub fn display_name(&self) -> &str {
        if !self.alias.is_empty() {
            &self.alias
        } else if let Some(name) = &self.name {
            name
        } else {
            &self.address
        }
    }
}
