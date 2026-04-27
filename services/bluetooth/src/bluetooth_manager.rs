use futures::Stream;
use std::collections::HashMap;
use tokio::sync::mpsc;
use zbus::{
    Connection, MatchRule,
    fdo::{ManagedObjects, ObjectManagerProxy},
    message::Type,
    zvariant::{OwnedObjectPath, OwnedValue},
};

// Import your generic stream builder from your common crate
use common::dbus::create_event_stream;

use crate::{
    AdapterInfo, DeviceInfo, RegisteredAgent,
    adapter::Adapter,
    agent::{AgentCapability, PairingRequest},
    device::Device,
    error::BluetoothError,
    events::BluetoothEvent,
};

// ---------- D-Bus Constants ----------
const BLUEZ_DEST: &str = "org.bluez";
const BLUEZ_PATH: &str = "/";
const IFACE_ADAPTER: &str = "org.bluez.Adapter1";
const IFACE_DEVICE: &str = "org.bluez.Device1";
const IFACE_BATTERY: &str = "org.bluez.Battery1";

/// The root client for interacting with the BlueZ Bluetooth stack.
#[derive(Clone)]
pub struct BluetoothManager {
    connection: Connection,
}

impl BluetoothManager {
    /// Establishes a connection to the system D-Bus.
    pub async fn new() -> Result<Self, BluetoothError> {
        let connection = Connection::system().await?;
        Ok(Self { connection })
    }

    /// Builds an ObjectManager proxy for querying BlueZ managed objects.
    async fn object_manager(&self) -> Result<ObjectManagerProxy<'_>, BluetoothError> {
        Ok(ObjectManagerProxy::new(&self.connection, BLUEZ_DEST, BLUEZ_PATH).await?)
    }

    /// Fetches all BlueZ managed objects and their interface properties.
    async fn managed_objects(&self) -> Result<ManagedObjects, BluetoothError> {
        Ok(self.object_manager().await?.get_managed_objects().await?)
    }

    /// Returns an `Adapter` struct to control a specific radio.
    pub async fn adapter(&self, name: &str) -> Result<Adapter, BluetoothError> {
        Adapter::new(self.connection.clone(), name).await
    }

    /// Returns a `Device` struct to control a specific remote device.
    pub async fn device(&self, path: &str) -> Result<Device, BluetoothError> {
        Device::new(self.connection.clone(), path).await
    }

    /// Retrieves a list of all available Bluetooth adapters.
    pub async fn get_adapters(&self) -> Result<Vec<AdapterInfo>, BluetoothError> {
        let managed_objects = self.managed_objects().await?;
        let adapters = managed_objects
            .into_iter()
            .filter_map(|(path, interfaces)| {
                interfaces
                    .get(IFACE_ADAPTER)
                    .map(|props| AdapterInfo::from_properties(path.to_string(), props))
            })
            .collect();

        Ok(adapters)
    }

    /// Retrieves a snapshot of all devices currently cached by the BlueZ daemon.
    pub async fn get_devices(&self) -> Result<Vec<DeviceInfo>, BluetoothError> {
        let managed_objects = self.managed_objects().await?;
        let devices = managed_objects
            .into_iter()
            .filter_map(|(path, interfaces)| {
                interfaces
                    .get(IFACE_DEVICE)
                    .map(|props| DeviceInfo::from_properties(path.to_string(), props))
            })
            .collect();

        Ok(devices)
    }

    /// Returns the preferred adapter: first powered one, otherwise the first available adapter.
    pub async fn get_default_adapter(&self) -> Result<Option<AdapterInfo>, BluetoothError> {
        let mut adapters = self.get_adapters().await?;

        if adapters.is_empty() {
            return Ok(None);
        }

        adapters.sort_by(|a, b| a.path.cmp(&b.path));

        if let Some(powered_adapter) = adapters.iter().find(|a| a.powered) {
            return Ok(Some(powered_adapter.clone()));
        }

        Ok(adapters.into_iter().next())
    }

    /// Registers a pairing agent.
    pub async fn register_agent(
        &self,
        capability: AgentCapability,
        set_as_default: bool,
    ) -> Result<(RegisteredAgent, mpsc::Receiver<PairingRequest>), BluetoothError> {
        RegisteredAgent::register(self.connection.clone(), capability, set_as_default).await
    }

    // ==========================================
    // Event Streaming & Parsing
    // ==========================================

    /// Returns a resilient, auto-reconnecting Stream of Bluetooth events.
    pub fn stream_events(&self) -> impl Stream<Item = BluetoothEvent> + Send + 'static {
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender(BLUEZ_DEST)
            .expect("Failed to build BlueZ match rule")
            .build();

        create_event_stream(self.connection.clone(), rule, Self::parse_bluez_signal)
    }

    /// Main entry point for parsing D-Bus signals into our domain events
    fn parse_bluez_signal(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        let header = msg.header();

        let member = header.member().map(|m| m.as_str());
        let interface = header.interface().map(|i| i.as_str());
        let Some(path) = header.path().map(|p| p.to_string()) else {
            return vec![];
        };

        match (interface, member) {
            (Some("org.freedesktop.DBus.Properties"), Some("PropertiesChanged")) => {
                Self::parse_properties_changed(msg, path)
            }
            (Some("org.freedesktop.DBus.ObjectManager"), Some("InterfacesAdded")) => {
                Self::parse_interfaces_added(msg)
            }
            (Some("org.freedesktop.DBus.ObjectManager"), Some("InterfacesRemoved")) => {
                Self::parse_interfaces_removed(msg)
            }
            _ => vec![],
        }
    }

    fn parse_properties_changed(msg: &zbus::Message, path: String) -> Vec<BluetoothEvent> {
        type PropsData = (String, HashMap<String, OwnedValue>, Vec<String>);
        let mut events = Vec::new();

        let Ok((iface, changed_props, _)) = msg.body().deserialize::<PropsData>() else {
            return events;
        };

        match iface.as_str() {
            IFACE_ADAPTER => {
                let adapter_name = path.split('/').next_back().unwrap_or("unknown").to_string();

                if let Some(val) = changed_props.get("Powered")
                    && let Ok(powered) = bool::try_from(val)
                {
                    events.push(BluetoothEvent::AdapterPowerChanged {
                        adapter_name: adapter_name.clone(),
                        powered,
                    });
                }
                if let Some(val) = changed_props.get("Discovering")
                    && let Ok(discovering) = bool::try_from(val)
                {
                    events.push(BluetoothEvent::DiscoveryStateChanged {
                        adapter_name,
                        discovering,
                    });
                }
            }
            IFACE_DEVICE => {
                let address = path.split("dev_").last().unwrap_or("").replace('_', ":");

                if let Some(connected) = changed_props
                    .get("Connected")
                    .and_then(|v| bool::try_from(v).ok())
                {
                    if connected {
                        events.push(BluetoothEvent::DeviceConnected {
                            path: path.clone(),
                            address,
                        });
                    } else {
                        events.push(BluetoothEvent::DeviceDisconnected {
                            path: path.clone(),
                            address,
                        });
                    }
                }

                if let Some(rssi) = changed_props
                    .get("RSSI")
                    .and_then(|v| i16::try_from(v).ok())
                {
                    events.push(BluetoothEvent::DeviceRssiChanged {
                        path: path.clone(),
                        rssi,
                    });
                }
            }
            IFACE_BATTERY => {
                if let Some(percentage) = changed_props
                    .get("Percentage")
                    .and_then(|v| u8::try_from(v).ok())
                {
                    events.push(BluetoothEvent::BatteryChanged { path, percentage });
                }
            }
            _ => {}
        }

        events
    }

    fn parse_interfaces_added(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        type AddedData = (
            OwnedObjectPath,
            HashMap<String, HashMap<String, OwnedValue>>,
        );
        let mut events = Vec::new();

        let Ok((obj_path, interfaces)) = msg.body().deserialize::<AddedData>() else {
            return events;
        };
        let path_str = obj_path.as_str();

        if let Some(adapter_props) = interfaces.get(IFACE_ADAPTER) {
            events.push(BluetoothEvent::AdapterAdded(AdapterInfo::from_properties(
                path_str.to_string(),
                adapter_props,
            )));
        }

        if let Some(device_props) = interfaces.get(IFACE_DEVICE) {
            events.push(BluetoothEvent::DeviceDiscovered(
                DeviceInfo::from_properties(path_str.to_string(), device_props),
            ));
        }

        events
    }

    fn parse_interfaces_removed(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        type RemovedData = (OwnedObjectPath, Vec<String>);
        let mut events = Vec::new();

        let Ok((obj_path, interfaces)) = msg.body().deserialize::<RemovedData>() else {
            return events;
        };
        let path_str = obj_path.as_str();

        if interfaces.iter().any(|i| i == IFACE_ADAPTER) {
            let name = path_str
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string();
            events.push(BluetoothEvent::AdapterRemoved { name });
        }

        if interfaces.iter().any(|i| i == IFACE_DEVICE) {
            events.push(BluetoothEvent::DeviceLost {
                path: path_str.to_string(),
            });
        }

        events
    }
}
