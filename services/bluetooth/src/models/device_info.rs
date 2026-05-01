use std::collections::HashMap;

use zbus::zvariant::{DeserializeDict, OwnedObjectPath, OwnedValue, Type};

#[derive(DeserializeDict, Type, OwnedValue, Debug, Default, Clone)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub struct DeviceProperties {
    pub adapter: Option<OwnedObjectPath>,
    pub address: Option<String>,
    pub address_type: Option<String>,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub icon: Option<String>,
    pub class: Option<u32>,
    pub appearance: Option<u16>,
    pub connected: Option<bool>,
    pub paired: Option<bool>,
    pub trusted: Option<bool>,
    pub blocked: Option<bool>,
    pub services_resolved: Option<bool>,
    pub rssi: Option<i16>,
    pub tx_power: Option<i16>,
    pub manufacturer_data: Option<HashMap<u16, OwnedValue>>,
    pub service_data: Option<HashMap<String, OwnedValue>>,
    pub uuids: Option<Vec<String>>,
    pub wake_allowed: Option<bool>,
    pub legacy_pairing: Option<bool>,
    pub modalias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: OwnedObjectPath,
    pub properties: DeviceProperties,
}

impl DeviceInfo {
    pub fn from_properties(
        path: OwnedObjectPath,
        props: &HashMap<String, OwnedValue>,
    ) -> Result<Self, zbus::zvariant::Error> {
        let owned: OwnedValue = props.clone().into();
        Ok(Self {
            path,
            properties: DeviceProperties::try_from(owned)?,
        })
    }

    pub fn display_name(&self) -> &str {
        self.properties
            .alias
            .as_deref()
            .or(self.properties.name.as_deref())
            .unwrap_or("Unknown Device")
    }
}
