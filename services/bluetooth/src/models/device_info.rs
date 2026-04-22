use common::{ValueMapExt, VariantDict};

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
    pub(crate) fn from_properties(path: String, props: &VariantDict) -> Self {
        Self {
            path,
            name: props.get_string("Name"),
            alias: props.get_string_or_default("Alias"),
            address: props.get_string_or_default("Address"),
            icon: props.get_string("Icon"),
            paired: props.get_as_or_default("Paired"),
            connected: props.get_as_or_default("Connected"),
            trusted: props.get_as_or_default("Trusted"),
            blocked: props.get_as_or_default("Blocked"),
            wake_allowed: props.get_as_or_default("WakeAllowed"),
            services_resolved: props.get_as_or_default("ServicesResolved"),
            rssi: props.get_as("RSSI"),
            battery_percentage: props.get_as("BatteryPercentage"),
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
