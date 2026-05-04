use std::collections::HashMap;

use zbus::{
    proxy,
    zvariant::{DeserializeDict, OwnedObjectPath, OwnedValue, Type},
};

use crate::dbus::AddressType;

#[derive(DeserializeDict, Type, Debug, Default, Clone)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub struct Device1Properties {
    pub adapter: Option<OwnedObjectPath>,
    pub address: Option<String>,
    pub address_type: Option<AddressType>,
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

#[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
pub trait Device1 {
    /// CancelPairing method
    fn cancel_pairing(&self) -> zbus::Result<()>;

    /// Connect method
    fn connect(&self) -> zbus::Result<()>;

    /// ConnectProfile method
    fn connect_profile(&self, uuid: &str) -> zbus::Result<()>;

    /// Disconnect method
    fn disconnect(&self) -> zbus::Result<()>;

    /// DisconnectProfile method
    fn disconnect_profile(&self, uuid: &str) -> zbus::Result<()>;

    /// GetServiceRecords method
    fn get_service_records(&self) -> zbus::Result<Vec<Vec<u8>>>;

    /// Pair method
    fn pair(&self) -> zbus::Result<()>;

    /// Disconnected signal
    #[zbus(signal)]
    fn disconnected(&self, name: &str, message: &str) -> zbus::Result<()>;

    /// Adapter property
    #[zbus(property)]
    fn adapter(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Address property
    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    /// AddressType property
    #[zbus(property)]
    fn address_type(&self) -> zbus::Result<String>;

    /// AdvertisingData property
    #[zbus(property)]
    fn advertising_data(
        &self,
    ) -> zbus::Result<std::collections::HashMap<u8, zbus::zvariant::OwnedValue>>;

    /// AdvertisingFlags property
    #[zbus(property)]
    fn advertising_flags(&self) -> zbus::Result<Vec<u8>>;

    /// Alias property
    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_alias(&self, value: &str) -> zbus::Result<()>;

    /// Appearance property
    #[zbus(property)]
    fn appearance(&self) -> zbus::Result<u16>;

    /// Blocked property
    #[zbus(property)]
    fn blocked(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_blocked(&self, value: bool) -> zbus::Result<()>;

    /// Bonded property
    #[zbus(property)]
    fn bonded(&self) -> zbus::Result<bool>;

    /// CablePairing property
    #[zbus(property)]
    fn cable_pairing(&self) -> zbus::Result<bool>;

    /// Class property
    #[zbus(property)]
    fn class(&self) -> zbus::Result<u32>;

    /// Connected property
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    /// Icon property
    #[zbus(property)]
    fn icon(&self) -> zbus::Result<String>;

    /// LegacyPairing property
    #[zbus(property)]
    fn legacy_pairing(&self) -> zbus::Result<bool>;

    /// ManufacturerData property
    #[zbus(property)]
    fn manufacturer_data(
        &self,
    ) -> zbus::Result<std::collections::HashMap<u16, zbus::zvariant::OwnedValue>>;

    /// Modalias property
    #[zbus(property)]
    fn modalias(&self) -> zbus::Result<String>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Paired property
    #[zbus(property)]
    fn paired(&self) -> zbus::Result<bool>;

    /// PreferredBearer property
    #[zbus(property)]
    fn preferred_bearer(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_preferred_bearer(&self, value: &str) -> zbus::Result<()>;

    /// RSSI property
    #[zbus(property, name = "RSSI")]
    fn rssi(&self) -> zbus::Result<i16>;

    /// ServiceData property
    #[zbus(property)]
    fn service_data(
        &self,
    ) -> zbus::Result<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>;

    /// ServicesResolved property
    #[zbus(property)]
    fn services_resolved(&self) -> zbus::Result<bool>;

    /// Sets property
    #[zbus(property)]
    fn sets(
        &self,
    ) -> zbus::Result<
        std::collections::HashMap<
            zbus::zvariant::OwnedObjectPath,
            std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        >,
    >;

    /// Trusted property
    #[zbus(property)]
    fn trusted(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_trusted(&self, value: bool) -> zbus::Result<()>;

    /// TxPower property
    #[zbus(property)]
    fn tx_power(&self) -> zbus::Result<i16>;

    /// UUIDs property
    #[zbus(property, name = "UUIDs")]
    fn uuids(&self) -> zbus::Result<Vec<String>>;

    /// WakeAllowed property
    #[zbus(property)]
    fn wake_allowed(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_wake_allowed(&self, value: bool) -> zbus::Result<()>;
}
