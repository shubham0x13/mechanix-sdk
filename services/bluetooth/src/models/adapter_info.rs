use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{DeserializeDict, OwnedObjectPath, OwnedValue, Type, Value};

#[derive(DeserializeDict, Type, OwnedValue, Debug, Default, Clone)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub struct AdapterProperties {
    pub address: Option<String>,
    pub address_type: Option<String>,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub class: Option<u32>,
    pub powered: Option<bool>,
    pub power_state: Option<String>,
    pub connactable: Option<bool>,
    pub discovering: Option<bool>,
    pub discoverable: Option<bool>,
    pub discoverable_timeout: Option<u32>,
    pub pairable: Option<bool>,
    pub pairable_timeout: Option<u32>,

    #[zvariant(rename = "UUIDs")]
    pub uuids: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub modalias: Option<String>,
    pub manufacturer: Option<u16>,
    pub version: Option<u8>,
    pub experimental_features: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub path: OwnedObjectPath,
    pub properties: AdapterProperties,
}

impl AdapterInfo {
    pub fn from_properties(
        path: OwnedObjectPath,
        props: &HashMap<String, OwnedValue>,
    ) -> Result<Self, zbus::zvariant::Error> {
        let owned: OwnedValue = props.clone().into();
        Ok(Self {
            path,
            properties: AdapterProperties::try_from(owned)?,
        })
    }

    pub fn display_name(&self) -> &str {
        self.properties
            .alias
            .as_deref()
            .or(self.properties.name.as_deref())
            .unwrap_or("Unknown Adapter")
    }
}

#[derive(Deserialize, Serialize, Type, Value, OwnedValue, Debug, Clone, PartialEq)]
#[zvariant(signature = "s", rename_all = "lowercase")]
pub enum AddressType {
    Public,
    Random,
}
