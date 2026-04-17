use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DBus error: {0}")]
    Zbus(#[from] zbus::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
