use thiserror::Error;

#[derive(Error, Debug)]
pub enum BluetoothError {
    // ---------- D-Bus ----------
    #[error("D-Bus communication failed: {0}")]
    DBus(#[from] zbus::Error),

    #[error("D-Bus FDO error: {0}")]
    DBusFdo(#[from] zbus::fdo::Error),

    #[error("D-Bus variant type error: {0}")]
    ZVariant(#[from] zbus::zvariant::Error),

    // ---------- Path & Address ----------
    #[error("Invalid D-Bus object path: {0}")]
    InvalidObjectPath(String),

    // #[error("Invalid MAC address format: {0}")]
    // InvalidMacAddress(String),

    // ---------- Adapter ----------
    // #[error("No Bluetooth adapter found")]
    // NoAdapterFound,

    // #[error("Adapter not found: {0}")]
    // AdapterNotFound(String),
    #[error("Adapter is powered off")]
    AdapterPoweredOff,

    // ---------- Device ----------
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device is not connected: {0}")]
    DeviceNotConnected(String),

    #[error("Device is already connected: {0}")]
    DeviceAlreadyConnected(String),

    // #[error("Device is not paired: {0}")]
    // DeviceNotPaired(String),
    #[error("Device is already paired: {0}")]
    DeviceAlreadyPaired(String),

    // #[error("Device services not yet resolved: {0}")]
    // ServicesNotResolved(String),

    // ---------- Agent & Pairing ----------
    #[error("Agent already registered")]
    AgentAlreadyRegistered,

    #[error("Agent not registered")]
    AgentNotRegistered,
}
