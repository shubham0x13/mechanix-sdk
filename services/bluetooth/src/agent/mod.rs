pub(crate) mod server;
pub mod types;

use self::server::BluezAgent;
use crate::{dbus::AgentManager1Proxy, error::BluetoothError};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
pub use types::*;
use zbus::Connection;

const AGENT_ROOT_PATH: &str = "/org/bluez/agents";
const PAIRING_REQUEST_CHANNEL_CAPACITY: usize = 16;
static NEXT_AGENT_ID: AtomicU64 = AtomicU64::new(1);

/// Represents an active pairing agent. Dropping this does not unregister it;
/// you must call `unregister()` to cleanly detach from BlueZ.
pub struct RegisteredAgent {
    path: String,
    connection: Connection,
}

impl RegisteredAgent {
    /// Registers a new pairing agent with BlueZ.
    pub async fn register(
        connection: Connection,
        capability: AgentCapability,
        set_as_default: bool,
    ) -> Result<(Self, mpsc::Receiver<PairingRequest>), BluetoothError> {
        let (ui_tx, ui_rx) = mpsc::channel(PAIRING_REQUEST_CHANNEL_CAPACITY);

        let id = NEXT_AGENT_ID.fetch_add(1, Ordering::Relaxed);
        let agent_path = format!("{AGENT_ROOT_PATH}/p{}_{}", std::process::id(), id);

        let dbus_path = zbus::zvariant::ObjectPath::try_from(agent_path.as_str())
            .map_err(|_| BluetoothError::InvalidObjectPath(agent_path.clone()))?;

        connection
            .object_server()
            .at(agent_path.as_str(), BluezAgent { ui_tx })
            .await?;

        let proxy = AgentManager1Proxy::new(&connection).await?;
        proxy
            .register_agent(&dbus_path, capability.as_str())
            .await
            .map_err(|e| match &e {
                zbus::Error::FDO(fdo)
                    if matches!(fdo.as_ref(), zbus::fdo::Error::FileExists(_)) =>
                {
                    BluetoothError::AgentAlreadyRegistered
                }
                _ => BluetoothError::DBus(e),
            })?;

        if set_as_default {
            proxy.request_default_agent(&dbus_path).await?;
        }

        Ok((
            Self {
                path: agent_path,
                connection,
            },
            ui_rx,
        ))
    }

    /// Unregisters the pairing agent from BlueZ and cleans up the D-Bus object server.
    pub async fn unregister(self) -> Result<(), BluetoothError> {
        let path = zbus::zvariant::ObjectPath::try_from(self.path.as_str())
            .map_err(|_| BluetoothError::InvalidObjectPath(self.path.clone()))?;

        let proxy = AgentManager1Proxy::new(&self.connection).await?;
        proxy.unregister_agent(&path).await.map_err(|e| match &e {
            zbus::Error::FDO(fdo)
                if matches!(fdo.as_ref(), zbus::fdo::Error::ServiceUnknown(_)) =>
            {
                BluetoothError::AgentNotRegistered
            }
            _ => BluetoothError::DBus(e),
        })?;

        self.connection
            .object_server()
            .remove::<BluezAgent, _>(self.path.as_str())
            .await?;

        Ok(())
    }
}
