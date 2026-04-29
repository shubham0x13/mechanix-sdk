use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkManagerError {
    #[error("D-Bus communication failed: {0}")]
    DBus(#[from] zbus::Error),

    #[error("D-Bus FDO error: {0}")]
    DBusFdo(#[from] zbus::fdo::Error),

    #[error("Invalid D-Bus Object Path: {0}")]
    InvalidObjectPath(String),

    #[error("No Wi-Fi device is currently available")]
    NoWifiDevice,

    #[error("Saved Wi-Fi connection not found for UUID: {0}")]
    SavedConnectionNotFound(String),
}
