use futures::Stream;
use std::collections::HashMap;
use tokio::sync::mpsc;
use zbus::{
    Connection, MatchRule,
    message::Type,
    zvariant::{ObjectPath, OwnedObjectPath},
};

use common::dbus::create_event_stream;

use crate::{
    AdapterInfo, DeviceInfo, RegisteredAgent,
    adapter::Adapter,
    agent::{AgentCapability, PairingRequest},
    dbus::{
        Adapter1Properties, Battery1Properties, BluezInterfaces, Device1Properties,
        TypedObjectManagerProxy,
    },
    device::Device,
    error::BluetoothError,
    events::BluetoothEvent,
    utils::extract_mac,
};

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
    async fn object_manager(&self) -> Result<TypedObjectManagerProxy<'_>, BluetoothError> {
        Ok(TypedObjectManagerProxy::new(&self.connection).await?)
    }

    /// Fetches all BlueZ managed objects and their interface properties.
    async fn managed_objects(
        &self,
    ) -> Result<HashMap<OwnedObjectPath, BluezInterfaces>, BluetoothError> {
        Ok(self.object_manager().await?.get_managed_objects().await?)
    }

    /// Returns an `Adapter` struct to control a specific radio.
    pub async fn adapter<P>(&self, path: P) -> Result<Adapter, BluetoothError>
    where
        P: TryInto<OwnedObjectPath>,
        P::Error: std::fmt::Display,
    {
        Adapter::new(self.connection.clone(), path).await
    }

    /// Returns a `Device` struct to control a specific remote device.
    pub async fn device<P>(&self, path: P) -> Result<Device, BluetoothError>
    where
        P: TryInto<OwnedObjectPath>,
        P::Error: std::fmt::Display,
    {
        Device::new(self.connection.clone(), path).await
    }

    /// Retrieves a list of all available Bluetooth adapters.
    pub async fn get_adapters(&self) -> Result<Vec<AdapterInfo>, BluetoothError> {
        let managed_objects = self.managed_objects().await?;
        let adapters = managed_objects
            .into_iter()
            .filter_map(|(path, interfaces)| {
                interfaces
                    .adapter1
                    .map(|properties| AdapterInfo::new(path, properties))
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
                    .device1
                    .map(|properties| DeviceInfo::new(path, properties))
            })
            .collect();

        Ok(devices)
    }

    /// Returns the preferred adapter: first powered one, otherwise the first available adapter.
    ///
    /// Returns [`BluetoothError::NoAdapterFound`] if no adapters are present.
    pub async fn get_default_adapter(&self) -> Result<Option<AdapterInfo>, BluetoothError> {
        let mut adapters = self.get_adapters().await?;

        if adapters.is_empty() {
            return Ok(None);
        }

        adapters.sort_by(|a, b| a.path.cmp(&b.path));

        if let Some(powered) = adapters
            .iter()
            .find(|a| a.properties.powered.unwrap_or(false))
        {
            return Ok(Some(powered.clone()));
        }

        Ok(adapters.into_iter().next())
    }

    /// Registers a pairing agent.
    ///
    /// Returns [`BluetoothError::AgentAlreadyRegistered`] if an agent is already active.
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
            .sender(BluezInterfaces::BLUEZ_DEST)
            .expect("Failed to build BlueZ match rule")
            .build();

        create_event_stream(self.connection.clone(), rule, Self::parse_bluez_signal)
    }

    /// Main entry point for parsing D-Bus signals into our domain events
    fn parse_bluez_signal(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        let header = msg.header();

        let member = header.member().map(|m| m.as_str());
        let interface = header.interface().map(|i| i.as_str());
        let Some(path) = header.path() else {
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

    fn parse_properties_changed(msg: &zbus::Message, path: &ObjectPath) -> Vec<BluetoothEvent> {
        let mut events = Vec::new();
        let owned_path = OwnedObjectPath::from(path.clone());

        let Ok((iface, _props, _invalidated)) = msg.body().deserialize::<(
            String,
            HashMap<String, zbus::zvariant::OwnedValue>,
            Vec<String>,
        )>() else {
            tracing::warn!("Failed to peek PropertiesChanged interface for {}", path);
            return events;
        };

        match iface.as_str() {
            BluezInterfaces::ADAPTER_IFACE => {
                match msg
                    .body()
                    .deserialize::<(String, Adapter1Properties, Vec<String>)>()
                {
                    Ok((_, changes, _)) => {
                        events.push(BluetoothEvent::AdapterPropertiesChanged {
                            path: owned_path,
                            changes,
                        });
                    }
                    Err(e) => tracing::warn!(
                        "Failed to deserialize Adapter1Properties for {}: {}",
                        path,
                        e
                    ),
                }
            }
            BluezInterfaces::DEVICE_IFACE => {
                let address = extract_mac(&owned_path).unwrap_or_default();

                match msg
                    .body()
                    .deserialize::<(String, Device1Properties, Vec<String>)>()
                {
                    Ok((_, changes, _)) => {
                        events.push(BluetoothEvent::DevicePropertiesChanged {
                            path: owned_path,
                            address,
                            changes,
                        });
                    }
                    Err(e) => tracing::warn!(
                        "Failed to deserialize Device1Properties for {}: {}",
                        path,
                        e
                    ),
                }
            }
            BluezInterfaces::BATTERY_IFACE => {
                let address = crate::utils::extract_mac(path.as_str()).unwrap_or_default();

                match msg
                    .body()
                    .deserialize::<(String, Battery1Properties, Vec<String>)>()
                {
                    Ok((_, changes, _)) => {
                        events.push(BluetoothEvent::BatteryChanged {
                            path: owned_path,
                            address,
                            changes,
                        });
                    }
                    Err(e) => tracing::warn!(
                        "Failed to deserialize Battery1Properties for {}: {}",
                        path,
                        e
                    ),
                }
            }
            _ => {}
        }

        events
    }

    fn parse_interfaces_added(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        let mut events = Vec::new();

        let Ok((obj_path, interfaces)) = msg
            .body()
            .deserialize::<(OwnedObjectPath, BluezInterfaces)>()
        else {
            tracing::warn!("Failed to deserialize InterfacesAdded signal");
            return events;
        };

        if let Some(adapter_props) = interfaces.adapter1 {
            events.push(BluetoothEvent::AdapterAdded(AdapterInfo::new(
                obj_path.clone(),
                adapter_props,
            )));
        }

        if let Some(device_props) = interfaces.device1 {
            events.push(BluetoothEvent::DeviceDiscovered(DeviceInfo::new(
                obj_path,
                device_props,
            )));
        }

        events
    }

    fn parse_interfaces_removed(msg: &zbus::Message) -> Vec<BluetoothEvent> {
        type RemovedData = (OwnedObjectPath, Vec<String>);
        let mut events = Vec::new();

        let Ok((obj_path, interfaces)) = msg.body().deserialize::<RemovedData>() else {
            tracing::warn!("Failed to deserialize InterfacesRemoved signal");
            return events;
        };

        if interfaces
            .iter()
            .any(|i| i == BluezInterfaces::ADAPTER_IFACE)
        {
            events.push(BluetoothEvent::AdapterRemoved {
                path: obj_path.clone(),
            });
        }

        if interfaces
            .iter()
            .any(|i| i == BluezInterfaces::DEVICE_IFACE)
        {
            events.push(BluetoothEvent::DeviceLost { path: obj_path });
        }

        events
    }
}
