use std::collections::HashMap;

use zbus::zvariant::{self, DeserializeDict, OwnedObjectPath, Type};

use crate::dbus::{Adapter1Properties, Battery1Properties, Device1Properties};

#[derive(DeserializeDict, Type, Default, Debug, Clone)]
#[zvariant(signature = "a{sa{sv}}")]
pub struct BluezInterfaces {
    #[zvariant(rename = "org.bluez.Adapter1")]
    pub adapter1: Option<Adapter1Properties>,

    #[zvariant(rename = "org.bluez.Device1")]
    pub device1: Option<Device1Properties>,

    #[zvariant(rename = "org.bluez.Battery1")]
    pub battery1: Option<Battery1Properties>,
}

impl BluezInterfaces {
    pub const BLUEZ_DEST: &'static str = "org.bluez";
    pub const ADAPTER_IFACE: &'static str = "org.bluez.Adapter1";
    pub const DEVICE_IFACE: &'static str = "org.bluez.Device1";
    pub const BATTERY_IFACE: &'static str = "org.bluez.Battery1";
}

#[zbus::proxy(
    interface = "org.freedesktop.DBus.ObjectManager",
    default_service = "org.bluez",
    default_path = "/"
)]
pub trait TypedObjectManager {
    #[zbus(name = "GetManagedObjects")]
    fn get_managed_objects(&self) -> zbus::Result<HashMap<OwnedObjectPath, BluezInterfaces>>;
}
