use zbus::zvariant::OwnedObjectPath;

use crate::{
    AdapterInfo, DeviceInfo,
    dbus::{Adapter1Properties, Battery1Properties, Device1Properties},
};

#[derive(Debug, Clone)]
pub enum BluetoothEvent {
    // --- Adapter Lifecycle ---
    AdapterAdded(AdapterInfo),

    AdapterRemoved {
        path: OwnedObjectPath,
    },

    // --- Device Lifecycle Events ---
    DeviceDiscovered(DeviceInfo),

    DeviceLost {
        path: OwnedObjectPath,
    },

    /// --- Centralized State Changes ---
    AdapterPropertiesChanged {
        path: OwnedObjectPath,
        changes: Adapter1Properties,
    },

    DevicePropertiesChanged {
        path: OwnedObjectPath,
        address: String,
        changes: Device1Properties,
    },

    BatteryChanged {
        path: OwnedObjectPath,
        address: String,
        changes: Battery1Properties,
    },
}
