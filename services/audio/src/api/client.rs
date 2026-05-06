use std::sync::Arc;

use parking_lot::RwLock;
use pipewire::channel;

use crate::error::AudioError;
use crate::hal::pw_thread;
use crate::service::{Command, SharedState};
use crate::types::AudioDevice;

// Owns the PW sender; sends Quit exactly once when the last AudioClient clone drops.
struct Inner {
    state: Arc<RwLock<SharedState>>,
    cmd_tx: channel::Sender<Command>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(Command::Quit);
    }
}

/// Handle to PipeWire audio. Cheap to clone — all clones share the same connection.
#[derive(Clone)]
pub struct AudioClient {
    inner: Arc<Inner>,
}

impl AudioClient {
    /// Connect to PipeWire and start the background thread.
    pub fn new() -> Result<Self, AudioError> {
        pipewire::init();

        let state = Arc::new(RwLock::new(SharedState::default()));
        let state_clone = Arc::clone(&state);

        let (cmd_tx, cmd_rx) = channel::channel::<Command>();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<(), AudioError>>(1);

        std::thread::Builder::new()
            .name("mechanix-pw".into())
            .spawn(move || pw_thread(state_clone, cmd_rx, ready_tx))
            .map_err(|e| AudioError::InitFailed(e.to_string()))?;

        ready_rx
            .recv()
            .map_err(|_| AudioError::InitFailed("PipeWire thread exited prematurely".into()))??;

        Ok(Self {
            inner: Arc::new(Inner { state, cmd_tx }),
        })
    }

    // ── Read-only queries ────────────────────────────────────────────────────

    pub fn list_devices(&self) -> Vec<AudioDevice> {
        let s = self.inner.state.read();
        s.nodes
            .values()
            .cloned()
            .filter_map(|n| {
                let default = (n.media_class.as_deref() == Some("Audio/Sink")
                    && s.default_sink.as_deref() == Some(n.name.as_str()))
                    || (n.media_class.as_deref() == Some("Audio/Source")
                        && s.default_source.as_deref() == Some(n.name.as_str()));
                n.into_audio_device(default)
            })
            .collect()
    }

    pub fn list_output_devices(&self) -> Vec<AudioDevice> {
        self.list_devices()
            .into_iter()
            .filter(|d| d.is_output())
            .collect()
    }

    pub fn list_input_devices(&self) -> Vec<AudioDevice> {
        self.list_devices()
            .into_iter()
            .filter(|d| d.is_input())
            .collect()
    }

    pub fn get_device(&self, id_or_name: &str) -> Option<AudioDevice> {
        let devices = self.list_devices();
        if let Ok(id) = id_or_name.parse::<u32>() {
            devices.into_iter().find(|d| d.id == id)
        } else {
            devices.into_iter().find(|d| d.name == id_or_name)
        }
    }

    pub fn default_output_device(&self) -> Result<AudioDevice, AudioError> {
        let name = self
            .inner
            .state
            .read()
            .default_sink
            .clone()
            .ok_or(AudioError::NoDefaultOutput)?;
        self.get_device(&name).ok_or(AudioError::NoDefaultOutput)
    }

    pub fn default_input_device(&self) -> Result<AudioDevice, AudioError> {
        let name = self
            .inner
            .state
            .read()
            .default_source
            .clone()
            .ok_or(AudioError::NoDefaultInput)?;
        self.get_device(&name).ok_or(AudioError::NoDefaultInput)
    }

    pub fn volume(&self, id_or_name: &str) -> Result<f32, AudioError> {
        self.get_device(id_or_name)
            .map(|d| d.volume)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))
    }

    pub fn mute(&self, id_or_name: &str) -> Result<bool, AudioError> {
        self.get_device(id_or_name)
            .map(|d| d.muted)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))
    }

    // ── Mutations ────────────────────────────────────────────────────────────

    pub async fn set_volume(&self, id_or_name: &str, volume: f32) -> Result<(), AudioError> {
        let volume = volume.clamp(0.0, 1.0);
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        let channels = self
            .inner
            .state
            .read()
            .nodes
            .get(&device.id)
            .map(|n| n.channels)
            .unwrap_or(2);

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::SetVolume {
                node_id: device.id,
                channels,
                volume,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        reply_rx
            .await
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    pub async fn set_default_output_device(&self, id_or_name: &str) -> Result<(), AudioError> {
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        if !device.is_output() {
            return Err(AudioError::SetDefaultFailed {
                direction: "output".into(),
                reason: format!("'{}' is an input device", device.name),
            });
        }

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::SetDefaultOutput {
                node_name: device.name,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        reply_rx
            .await
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    pub async fn set_default_input_device(&self, id_or_name: &str) -> Result<(), AudioError> {
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        if !device.is_input() {
            return Err(AudioError::SetDefaultFailed {
                direction: "input".into(),
                reason: format!("'{}' is an output device", device.name),
            });
        }

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::SetDefaultInput {
                node_name: device.name,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        reply_rx
            .await
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    pub async fn set_mute(&self, id_or_name: &str, muted: bool) -> Result<(), AudioError> {
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.inner
            .cmd_tx
            .send(Command::SetMute {
                node_id: device.id,
                muted,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        reply_rx
            .await
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }
}
