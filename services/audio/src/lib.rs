pub mod audio;
pub mod error;

pub use audio::{
    client::AudioClient,
    types::{AudioDevice, DeviceType},
};
pub use error::AudioError;
