use serde::Deserialize;
use zbus::{
    proxy,
    zvariant::{DeserializeDict, Type},
};

use crate::dbus::AddressType;

#[derive(Deserialize, Type, Debug, Clone, PartialEq, Eq)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Roles {
    Central,
    Peripheral,
    #[serde(rename = "central-peripheral")]
    CentralPeripheral,
    #[serde(other)]
    Unknown,
}

#[derive(DeserializeDict, Type, Default, Debug, Clone)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub struct Adapter1Properties {
    pub address: Option<String>,
    pub address_type: Option<AddressType>,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub class: Option<u32>,
    pub powered: Option<bool>,
    pub power_state: Option<String>,
    pub connectable: Option<bool>,
    pub discovering: Option<bool>,
    pub discoverable: Option<bool>,
    pub discoverable_timeout: Option<u32>,
    pub pairable: Option<bool>,
    pub pairable_timeout: Option<u32>,

    #[zvariant(rename = "UUIDs")]
    pub uuids: Option<Vec<String>>,
    pub roles: Option<Vec<Roles>>,
    pub modalias: Option<String>,
    pub manufacturer: Option<u16>,
    pub version: Option<u8>,
    pub experimental_features: Option<Vec<String>>,
}

#[proxy(interface = "org.bluez.Adapter1", default_service = "org.bluez")]
pub trait Adapter1 {
    /// ConnectDevice method
    fn connect_device(
        &self,
        properties: std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    /// GetDiscoveryFilters method
    fn get_discovery_filters(&self) -> zbus::Result<Vec<String>>;

    /// RemoveDevice method
    fn remove_device(&self, device: &zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// SetDiscoveryFilter method
    fn set_discovery_filter(
        &self,
        properties: std::collections::HashMap<&str, &zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    /// StartDiscovery method
    fn start_discovery(&self) -> zbus::Result<()>;

    /// StopDiscovery method
    fn stop_discovery(&self) -> zbus::Result<()>;

    /// Address property
    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    /// AddressType property
    #[zbus(property)]
    fn address_type(&self) -> zbus::Result<String>;

    /// Alias property
    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_alias(&self, value: &str) -> zbus::Result<()>;

    /// Class property
    #[zbus(property)]
    fn class(&self) -> zbus::Result<u32>;

    /// Connectable property
    #[zbus(property)]
    fn connectable(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_connectable(&self, value: bool) -> zbus::Result<()>;

    /// Discoverable property
    #[zbus(property)]
    fn discoverable(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_discoverable(&self, value: bool) -> zbus::Result<()>;

    /// DiscoverableTimeout property
    #[zbus(property)]
    fn discoverable_timeout(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_discoverable_timeout(&self, value: u32) -> zbus::Result<()>;

    /// Discovering property
    #[zbus(property)]
    fn discovering(&self) -> zbus::Result<bool>;

    /// ExperimentalFeatures property
    #[zbus(property)]
    fn experimental_features(&self) -> zbus::Result<Vec<String>>;

    /// Manufacturer property
    #[zbus(property)]
    fn manufacturer(&self) -> zbus::Result<u16>;

    /// Modalias property
    #[zbus(property)]
    fn modalias(&self) -> zbus::Result<String>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Pairable property
    #[zbus(property)]
    fn pairable(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_pairable(&self, value: bool) -> zbus::Result<()>;

    /// PairableTimeout property
    #[zbus(property)]
    fn pairable_timeout(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn set_pairable_timeout(&self, value: u32) -> zbus::Result<()>;

    /// PowerState property
    #[zbus(property)]
    fn power_state(&self) -> zbus::Result<String>;

    /// Powered property
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;

    /// Roles property
    #[zbus(property)]
    fn roles(&self) -> zbus::Result<Vec<String>>;

    /// UUIDs property
    #[zbus(property, name = "UUIDs")]
    fn uuids(&self) -> zbus::Result<Vec<String>>;

    /// Version property
    #[zbus(property)]
    fn version(&self) -> zbus::Result<u8>;
}
