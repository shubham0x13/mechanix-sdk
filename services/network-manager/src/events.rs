use crate::types::{
    NMActiveConnectionState, NMActiveConnectionStateReason, NMConnectivityState, NMDeviceState,
    NMDeviceStateReason, NMState,
};

/// Represents key signals and property changes emitted by NetworkManager.
#[derive(Debug, Clone)]
pub enum NetworkManagerEvent {
    // --- Global state ---
    NetworkingEnabledChanged {
        enabled: bool,
    },
    WifiEnabledChanged {
        enabled: bool,
    },
    ConnectivityChanged {
        state: NMConnectivityState,
    },
    StateChanged {
        state: NMState,
    },

    // --- Device lifecycle ---
    DeviceAdded {
        path: String,
    },
    DeviceRemoved {
        path: String,
    },

    // --- Device state ---
    DeviceStateChanged {
        path: String,
        new_state: NMDeviceState,
        old_state: NMDeviceState,
        reason: NMDeviceStateReason,
    },

    // --- Active connection state ---
    ActiveConnectionStateChanged {
        path: String,
        state: NMActiveConnectionState,
        reason: NMActiveConnectionStateReason,
    },

    // --- Wi-Fi access points ---
    AccessPointAdded {
        device_path: String,
        access_point: String,
    },
    AccessPointRemoved {
        device_path: String,
        access_point: String,
    },
    ActiveAccessPointChanged {
        device_path: String,
        access_point: Option<String>,
    },
}
