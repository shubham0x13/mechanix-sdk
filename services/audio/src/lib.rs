pub mod api;
pub mod error;
pub mod types;

pub(crate) mod hal;
pub(crate) mod service;

pub use api::AudioClient;
pub use error::AudioError;
pub use types::{AudioDevice, DeviceType};
