#![allow(clippy::all)]
#![allow(missing_docs)]
#![allow(unused)]

mod access_point;
mod active;
mod connection;
mod device;
mod ip4config;
mod ip6config;
mod network_manager;
mod settings;
mod statistics;
mod wired;
mod wireless;

pub use access_point::AccessPointProxy;
pub use active::ActiveProxy;
pub use connection::ConnectionProxy;
pub use device::DeviceProxy;
pub use ip4config::IP4ConfigProxy;
pub use ip6config::IP6ConfigProxy;
pub use network_manager::NetworkManagerProxy;
pub use settings::SettingsProxy;
pub use statistics::StatisticsProxy;
pub use wired::WiredProxy;
pub use wireless::WirelessProxy;
