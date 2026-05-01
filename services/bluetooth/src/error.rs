use thiserror::Error;

#[derive(Error, Debug)]
pub enum BluetoothError {
    #[error("D-Bus communication failed: {0}")]
    DBus(#[from] zbus::Error),

    #[error("D-Bus FDO error: {0}")]
    DBusFdo(#[from] zbus::fdo::Error),

    #[error("D-Bus variant error: {0}")]
    Zvariant(#[from] zbus::zvariant::Error),

    #[error("Invalid MAC address format: {0}")]
    InvalidMacAddress(String),

    #[error("Invalid D-Bus Object Path: {0}")]
    InvalidObjectPath(String),

    #[error("Device not found on adapter {adapter}")]
    DeviceNotFound { adapter: String },
}
