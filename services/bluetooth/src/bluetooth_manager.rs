use futures::StreamExt;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::{
    sync::{broadcast, mpsc},
    time::{Duration, sleep},
};
use zbus::{
    Connection, MatchRule, MessageStream,
    fdo::{ManagedObjects, ObjectManagerProxy},
    message::Type,
    zvariant::OwnedValue,
};

use crate::{
    AdapterInfo, DeviceInfo,
    adapter::Adapter,
    agent::{AgentCapability, BluezAgent, PairingRequest},
    dbus::AgentManager1Proxy,
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
const AGENT_ROOT_PATH: &str = "/org/bluez/agents";
const PAIRING_REQUEST_CHANNEL_CAPACITY: usize = 16;
const EVENT_RECONNECT_INITIAL_BACKOFF_MS: u64 = 250;
const EVENT_RECONNECT_MAX_BACKOFF_MS: u64 = 5_000;

static NEXT_AGENT_ID: AtomicU64 = AtomicU64::new(1);

/// The root client for interacting with the BlueZ Bluetooth stack.
#[derive(Clone)]
pub struct BluetoothManager {
    connection: Connection,
    event_tx: broadcast::Sender<BluetoothEvent>,
}

impl BluetoothManager {
    /// Establishes a connection to the system D-Bus and spawns the global event bus.
    pub async fn new() -> Result<Self, BluetoothError> {
        let connection = Connection::system().await?;
        let (event_tx, _) = broadcast::channel(100);

        let client = Self {
            connection,
            event_tx,
        };

        client.spawn_event_bus();
        Ok(client)
    }

    async fn object_manager(&self) -> Result<ObjectManagerProxy<'_>, BluetoothError> {
        Ok(ObjectManagerProxy::new(&self.connection, BLUEZ_DEST, BLUEZ_PATH).await?)
    }

    async fn managed_objects(&self) -> Result<ManagedObjects, BluetoothError> {
        Ok(self.object_manager().await?.get_managed_objects().await?)
    }

    /// Returns an `Adapter` struct to control a specific radio.
    pub fn adapter(&self, name: &str) -> Adapter {
        Adapter::new(self.connection.clone(), name)
    }

    /// Returns a `Device` struct to control a specific remote device.
    pub fn device(&self, path: &str) -> Device {
        Device::new(self.connection.clone(), path)
    }

    /// Retrieves a list of all available Bluetooth adapters (e.g., ["hci0", "hci1"]).
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

    pub async fn get_default_adapter(&self) -> Result<Option<AdapterInfo>, BluetoothError> {
        let mut adapters = self.get_adapters().await?;

        if adapters.is_empty() {
            return Ok(None);
        }

        // Sort them alphabetically by path so "/org/bluez/hci0" usually comes
        // before "/org/bluez/hci1", ensuring deterministic behavior.
        adapters.sort_by(|a, b| a.path.cmp(&b.path));

        // Find the first adapter that is actively powered on.
        if let Some(powered_adapter) = adapters.iter().find(|a| a.powered) {
            return Ok(Some(powered_adapter.clone()));
        }

        // If everything is turned off, just return the first one (usually hci0).
        Ok(adapters.into_iter().next())
    }

    /// Subscribes to the global event bus. Receives all adapter and device state changes.
    pub fn subscribe(&self) -> broadcast::Receiver<BluetoothEvent> {
        self.event_tx.subscribe()
    }

    /// Registers this application as the Bluetooth pairing agent.
    pub async fn register_agent(&self) -> Result<mpsc::Receiver<PairingRequest>, BluetoothError> {
        let (_path, rx) = self
            .register_agent_with_options(AgentCapability::KeyboardDisplay, true)
            .await?;
        Ok(rx)
    }

    /// Registers a pairing agent and returns both the agent path and request receiver.
    ///
    /// Save the returned path and pass it to `unregister_agent` when shutting down.
    pub async fn register_agent_with_options(
        &self,
        capability: AgentCapability,
        set_as_default: bool,
    ) -> Result<(String, mpsc::Receiver<PairingRequest>), BluetoothError> {
        let (ui_tx, ui_rx) = mpsc::channel(PAIRING_REQUEST_CHANNEL_CAPACITY);
        let agent = BluezAgent { ui_tx };

        let agent_path = Self::next_agent_path();
        let path = zbus::zvariant::ObjectPath::try_from(agent_path.as_str())
            .map_err(|_| BluetoothError::InvalidObjectPath(agent_path.clone()))?;

        self.connection
            .object_server()
            .at(agent_path.as_str(), agent)
            .await?;

        let manager_proxy = AgentManager1Proxy::new(&self.connection).await?;

        manager_proxy
            .register_agent(&path, capability.as_str())
            .await?;

        if set_as_default {
            manager_proxy.request_default_agent(&path).await?;
        }

        Ok((agent_path, ui_rx))
    }

    /// Unregisters a previously registered pairing agent.
    pub async fn unregister_agent(&self, agent_path: &str) -> Result<(), BluetoothError> {
        let path = zbus::zvariant::ObjectPath::try_from(agent_path)
            .map_err(|_| BluetoothError::InvalidObjectPath(agent_path.to_string()))?;

        let manager_proxy = AgentManager1Proxy::new(&self.connection).await?;

        manager_proxy.unregister_agent(&path).await?;
        self.connection
            .object_server()
            .remove::<BluezAgent, _>(agent_path)
            .await?;
        Ok(())
    }

    fn next_agent_path() -> String {
        let id = NEXT_AGENT_ID.fetch_add(1, Ordering::Relaxed);
        format!("{AGENT_ROOT_PATH}/p{}_{}", std::process::id(), id)
    }

    // ==========================================
    // The Core Event Router
    // ==========================================

    fn spawn_event_bus(&self) {
        let connection = self.connection.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let rule = match MatchRule::builder()
                .msg_type(Type::Signal)
                .sender("org.bluez")
            {
                Ok(builder) => builder.build(),
                Err(err) => {
                    eprintln!("BlueZ event bus setup failed to build match rule: {err}");
                    return;
                }
            };

            let mut reconnect_backoff = Duration::from_millis(EVENT_RECONNECT_INITIAL_BACKOFF_MS);

            loop {
                let mut stream =
                    match MessageStream::for_match_rule(rule.clone(), &connection, None).await {
                        Ok(stream) => {
                            reconnect_backoff =
                                Duration::from_millis(EVENT_RECONNECT_INITIAL_BACKOFF_MS);
                            stream
                        }
                        Err(err) => {
                            eprintln!(
                                "BlueZ event bus stream creation failed: {err}. Retrying in {:?}.",
                                reconnect_backoff
                            );
                            sleep(reconnect_backoff).await;
                            reconnect_backoff = (reconnect_backoff * 2)
                                .min(Duration::from_millis(EVENT_RECONNECT_MAX_BACKOFF_MS));
                            continue;
                        }
                    };

                loop {
                    match stream.next().await {
                        Some(Ok(msg)) => {
                            Self::route_event_signal(&tx, &msg);
                        }
                        Some(Err(err)) => {
                            eprintln!("BlueZ event bus stream error: {err}. Reconnecting.");
                            break;
                        }
                        None => {
                            eprintln!("BlueZ event bus stream ended. Reconnecting.");
                            break;
                        }
                    }
                }

                sleep(reconnect_backoff).await;
                reconnect_backoff = (reconnect_backoff * 2)
                    .min(Duration::from_millis(EVENT_RECONNECT_MAX_BACKOFF_MS));
            }
        });
    }

    fn route_event_signal(tx: &broadcast::Sender<BluetoothEvent>, msg: &zbus::Message) {
        let header = msg.header();

        let member = header.member().map(|m| m.as_str());
        let interface = header.interface().map(|i| i.as_str());
        let Some(path) = header.path().map(|p| p.to_string()) else {
            return;
        };

        // Route the raw D-Bus message to our parsers
        match (interface, member) {
            (Some("org.freedesktop.DBus.Properties"), Some("PropertiesChanged")) => {
                Self::handle_properties_changed(tx, msg, path);
            }
            (Some("org.freedesktop.DBus.ObjectManager"), Some("InterfacesAdded")) => {
                Self::handle_interfaces_added(tx, msg);
            }
            (Some("org.freedesktop.DBus.ObjectManager"), Some("InterfacesRemoved")) => {
                Self::handle_interfaces_removed(tx, msg);
            }
            _ => {} // Ignore other signals
        }
    }

    // ==========================================
    // Signal Parsers
    // ==========================================

    fn handle_properties_changed(
        tx: &broadcast::Sender<BluetoothEvent>,
        msg: &zbus::Message,
        path: String,
    ) {
        type PropsData = (String, HashMap<String, OwnedValue>, Vec<String>);

        let Ok((iface, changed_props, _)) = msg.body().deserialize::<PropsData>() else {
            return;
        };

        match iface.as_str() {
            IFACE_ADAPTER => {
                let adapter_name = path.split('/').next_back().unwrap_or("unknown").to_string();

                if let Some(val) = changed_props.get("Powered")
                    && let Ok(powered) = bool::try_from(val)
                {
                    let _ = tx.send(BluetoothEvent::AdapterPowerChanged {
                        adapter_name: adapter_name.clone(),
                        powered,
                    });
                }
                if let Some(val) = changed_props.get("Discovering")
                    && let Ok(discovering) = bool::try_from(val)
                {
                    let _ = tx.send(BluetoothEvent::DiscoveryStateChanged {
                        adapter_name,
                        discovering,
                    });
                }
            }
            IFACE_DEVICE => {
                // Incorporating the next_back() fix from earlier!
                let address = path.split("dev_").last().unwrap_or("").replace('_', ":");

                if let Some(connected) = changed_props
                    .get("Connected")
                    .and_then(|v| bool::try_from(v).ok())
                {
                    let event = if connected {
                        BluetoothEvent::DeviceConnected {
                            path: path.to_string(),
                            address,
                        }
                    } else {
                        BluetoothEvent::DeviceDisconnected {
                            path: path.to_string(),
                            address,
                        }
                    };
                    let _ = tx.send(event);
                }

                if let Some(rssi) = changed_props
                    .get("RSSI")
                    .and_then(|v| i16::try_from(v).ok())
                {
                    let _ = tx.send(BluetoothEvent::DeviceRssiChanged {
                        path: path.to_string(),
                        rssi,
                    });
                }
            }
            IFACE_BATTERY => {
                if let Some(percentage) = changed_props
                    .get("Percentage")
                    .and_then(|v| u8::try_from(v).ok())
                {
                    let _ = tx.send(BluetoothEvent::BatteryChanged {
                        path: path.to_string(),
                        percentage,
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_interfaces_added(tx: &broadcast::Sender<BluetoothEvent>, msg: &zbus::Message) {
        use zbus::zvariant::OwnedObjectPath;
        type AddedData = (
            OwnedObjectPath,
            HashMap<String, HashMap<String, OwnedValue>>,
        );

        let Ok((obj_path, interfaces)) = msg.body().deserialize::<AddedData>() else {
            return;
        };
        let path_str = obj_path.as_str();

        if let Some(adapter_props) = interfaces.get(IFACE_ADAPTER) {
            let info = AdapterInfo::from_properties(path_str.to_string(), adapter_props);
            let _ = tx.send(BluetoothEvent::AdapterAdded(info));
        }

        if let Some(device_props) = interfaces.get(IFACE_DEVICE) {
            let info = DeviceInfo::from_properties(path_str.to_string(), device_props);
            // println!("{:?}", device_props);
            let _ = tx.send(BluetoothEvent::DeviceDiscovered(info));
        }
    }

    fn handle_interfaces_removed(tx: &broadcast::Sender<BluetoothEvent>, msg: &zbus::Message) {
        use zbus::zvariant::OwnedObjectPath;
        type RemovedData = (OwnedObjectPath, Vec<String>);

        let Ok((obj_path, interfaces)) = msg.body().deserialize::<RemovedData>() else {
            return;
        };
        let path_str = obj_path.as_str();

        if interfaces.iter().any(|i| i == IFACE_ADAPTER) {
            let name = path_str
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string();
            let _ = tx.send(BluetoothEvent::AdapterRemoved { name });
        }

        if interfaces.iter().any(|i| i == IFACE_DEVICE) {
            let _ = tx.send(BluetoothEvent::DeviceLost {
                path: path_str.to_string(),
            });
        }
    }
}
