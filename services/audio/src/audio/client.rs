use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use parking_lot::RwLock;

use pipewire::{
    channel,
    main_loop::MainLoopRc,
    metadata::Metadata,
    node::Node,
    proxy::Listener,
    spa::{
        param::ParamType,
        pod::{Object, Pod, Property, PropertyFlags, Value, ValueArray, serialize::PodSerializer},
    },
    types::ObjectType,
};
use tracing::{debug, warn};

use crate::error::AudioError;

use super::types::{AudioDevice, DeviceType};

// Shared state between PW and callers

#[derive(Debug, Clone, Default)]
struct NodeSnapshot {
    id: u32,
    name: String,
    description: Option<String>,
    media_class: Option<String>,
    // linear volume [0.0, 1.0]
    volume: f32,
    muted: bool,
}

impl NodeSnapshot {
    fn device_type(&self) -> Option<DeviceType> {
        match self.media_class.as_deref() {
            Some("Audio/Sink") => Some(DeviceType::Output),
            Some("Audio/Source") => Some(DeviceType::Input),
            _ => None,
        }
    }

    fn into_audio_device(self, is_default: bool) -> Option<AudioDevice> {
        let device_type = self.device_type()?;
        Some(AudioDevice {
            id: self.id,
            name: self.name,
            description: self.description,
            device_type,
            volume: self.volume,
            muted: self.muted,
            is_default,
        })
    }
}

#[derive(Debug, Default)]
struct SharedState {
    nodes: HashMap<u32, NodeSnapshot>,
    default_sink: Option<String>,
    default_source: Option<String>,
}

// Commands for the PW thread

enum Command {
    SetVolume {
        node_id: u32,
        // number of channels to replicate across
        channels: u32,
        volume: f32,
        reply: std::sync::mpsc::SyncSender<Result<(), AudioError>>,
    },
    SetMute {
        node_id: u32,
        muted: bool,
        reply: std::sync::mpsc::SyncSender<Result<(), AudioError>>,
    },
    SetDefaultOutput {
        node_name: String,
        reply: std::sync::mpsc::SyncSender<Result<(), AudioError>>,
    },
    SetDefaultInput {
        node_name: String,
        reply: std::sync::mpsc::SyncSender<Result<(), AudioError>>,
    },
    Quit,
}

// Public client

// Handle to PipeWire audio. Cheap to clone, shares state.
#[derive(Clone)]
pub struct AudioClient {
    state: Arc<RwLock<SharedState>>,
    cmd_tx: channel::Sender<Command>,
}

impl AudioClient {
    // Connect to PW and start the background thread
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

        Ok(Self { state, cmd_tx })
    }

    // Read-only queries

    // All sinks and sources
    pub fn list_devices(&self) -> Vec<AudioDevice> {
        let s = self.state.read();
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

    // Output devices only
    pub fn list_output_devices(&self) -> Vec<AudioDevice> {
        self.list_devices()
            .into_iter()
            .filter(|d| d.is_output())
            .collect()
    }

    // Input devices only
    pub fn list_input_devices(&self) -> Vec<AudioDevice> {
        self.list_devices()
            .into_iter()
            .filter(|d| d.is_input())
            .collect()
    }

    // Get device by ID or name
    pub fn get_device(&self, id_or_name: &str) -> Option<AudioDevice> {
        let devices = self.list_devices();
        if let Ok(id) = id_or_name.parse::<u32>() {
            devices.into_iter().find(|d| d.id == id)
        } else {
            devices.into_iter().find(|d| d.name == id_or_name)
        }
    }

    // System default output
    pub fn default_output_device(&self) -> Result<AudioDevice, AudioError> {
        let name = self
            .state
            .read()
            .default_sink
            .clone()
            .ok_or(AudioError::NoDefaultOutput)?;
        self.get_device(&name).ok_or(AudioError::NoDefaultOutput)
    }

    // System default input
    pub fn default_input_device(&self) -> Result<AudioDevice, AudioError> {
        let name = self
            .state
            .read()
            .default_source
            .clone()
            .ok_or(AudioError::NoDefaultInput)?;
        self.get_device(&name).ok_or(AudioError::NoDefaultInput)
    }

    // Volume in [0.0, 1.0]
    pub fn volume(&self, id_or_name: &str) -> Result<f32, AudioError> {
        self.get_device(id_or_name)
            .map(|d| d.volume)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))
    }

    // Mute state
    pub fn mute(&self, id_or_name: &str) -> Result<bool, AudioError> {
        self.get_device(id_or_name)
            .map(|d| d.muted)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))
    }

    // Mutations

    // Set volume [0.0, 1.0]
    pub async fn set_volume(&self, id_or_name: &str, volume: f32) -> Result<(), AudioError> {
        let volume = volume.clamp(0.0, 1.0);
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        // replicate the value across all channels
        let channels = {
            let s = self.state.read();
            s.nodes.get(&device.id).map(|_| 2u32).unwrap_or(2)
        };

        let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
        self.cmd_tx
            .send(Command::SetVolume {
                node_id: device.id,
                channels,
                volume,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        tokio::task::spawn_blocking(move || reply_rx.recv())
            .await
            .map_err(|_| AudioError::Pipewire("task panicked".into()))?
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    // Set default output
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

        let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
        self.cmd_tx
            .send(Command::SetDefaultOutput {
                node_name: device.name,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        tokio::task::spawn_blocking(move || reply_rx.recv())
            .await
            .map_err(|_| AudioError::Pipewire("task panicked".into()))?
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    // Set default input
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

        let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
        self.cmd_tx
            .send(Command::SetDefaultInput {
                node_name: device.name,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        tokio::task::spawn_blocking(move || reply_rx.recv())
            .await
            .map_err(|_| AudioError::Pipewire("task panicked".into()))?
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }

    // Set mute state
    pub async fn set_mute(&self, id_or_name: &str, muted: bool) -> Result<(), AudioError> {
        let device = self
            .get_device(id_or_name)
            .ok_or_else(|| AudioError::DeviceNotFound(id_or_name.into()))?;

        let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
        self.cmd_tx
            .send(Command::SetMute {
                node_id: device.id,
                muted,
                reply: reply_tx,
            })
            .map_err(|_| AudioError::Pipewire("command channel closed".into()))?;

        tokio::task::spawn_blocking(move || reply_rx.recv())
            .await
            .map_err(|_| AudioError::Pipewire("task panicked".into()))?
            .map_err(|_| AudioError::Pipewire("reply channel dropped".into()))?
    }
}

impl Drop for AudioClient {
    fn drop(&mut self) {
        // Stop PW thread if this is the last client
        let _ = self.cmd_tx.send(Command::Quit);
    }
}

// PipeWire background thread

// Proxies and listeners we need to keep alive
struct PwProxies {
    nodes: HashMap<u32, (Node, Box<dyn Listener>)>,
    metadata: Vec<(Metadata, Box<dyn Listener>)>,
}

fn pw_thread(
    state: Arc<RwLock<SharedState>>,
    cmd_rx: channel::Receiver<Command>,
    ready_tx: std::sync::mpsc::SyncSender<Result<(), AudioError>>,
) {
    let main_loop = match MainLoopRc::new(None) {
        Ok(ml) => ml,
        Err(e) => {
            let _ = ready_tx.send(Err(AudioError::InitFailed(e.to_string())));
            return;
        }
    };

    let context = match pipewire::context::ContextRc::new(&main_loop, None) {
        Ok(ctx) => ctx,
        Err(e) => {
            let _ = ready_tx.send(Err(AudioError::InitFailed(e.to_string())));
            return;
        }
    };

    let core = match context.connect_rc(None) {
        Ok(c) => c,
        Err(e) => {
            let _ = ready_tx.send(Err(AudioError::InitFailed(e.to_string())));
            return;
        }
    };

    let registry = match core.get_registry_rc() {
        Ok(r) => r,
        Err(e) => {
            let _ = ready_tx.send(Err(AudioError::InitFailed(e.to_string())));
            return;
        }
    };

    let proxies = Rc::new(RefCell::new(PwProxies {
        nodes: HashMap::new(),
        metadata: Vec::new(),
    }));

    // Watch nodes and metadata
    let state_reg = Arc::clone(&state);
    let registry_weak = registry.downgrade();
    let proxies_reg = Rc::clone(&proxies);

    let _registry_listener = registry
        .add_listener_local()
        .global(move |obj| {
            let registry = match registry_weak.upgrade() {
                Some(r) => r,
                None => return,
            };

            match obj.type_ {
                ObjectType::Node => {
                    // Only audio sinks/sources
                    let media_class = obj
                        .props
                        .as_ref()
                        .and_then(|p| p.get("media.class"))
                        .unwrap_or("")
                        .to_string();

                    if !matches!(media_class.as_str(), "Audio/Sink" | "Audio/Source") {
                        return;
                    }

                    let name = obj
                        .props
                        .as_ref()
                        .and_then(|p| p.get("node.name"))
                        .unwrap_or("")
                        .to_string();
                    let description = obj
                        .props
                        .as_ref()
                        .and_then(|p| p.get("node.description"))
                        .map(str::to_string);
                    let node_id = obj.id;

                    {
                        let mut s = state_reg.write();
                        s.nodes.insert(
                            node_id,
                            NodeSnapshot {
                                id: node_id,
                                name,
                                description,
                                media_class: Some(media_class),
                                volume: 1.0,
                                muted: false,
                            },
                        );
                    }

                    let node: Node = match registry.bind(obj) {
                        Ok(n) => n,
                        Err(e) => {
                            warn!("Failed to bind node {node_id}: {e}");
                            return;
                        }
                    };

                    let state_param = Arc::clone(&state_reg);
                    let listener = node
                        .add_listener_local()
                        .param(move |_seq, id, _index, _next, pod| {
                            if id != ParamType::Props {
                                return;
                            }
                            if let Some(pod) = pod {
                                parse_props_pod(pod, node_id, &state_param);
                            }
                        })
                        .register();

                    // Initial param request
                    node.enum_params(0, Some(ParamType::Props), 0, u32::MAX);

                    proxies_reg
                        .borrow_mut()
                        .nodes
                        .insert(node_id, (node, Box::new(listener)));

                    debug!("Audio node registered: id={node_id}");
                }

                ObjectType::Metadata => {
                    let is_default =
                        obj.props.as_ref().and_then(|p| p.get("metadata.name")) == Some("default");
                    if !is_default {
                        return;
                    }

                    let metadata: Metadata = match registry.bind(obj) {
                        Ok(m) => m,
                        Err(e) => {
                            warn!("Failed to bind metadata: {e}");
                            return;
                        }
                    };

                    let state_meta = Arc::clone(&state_reg);
                    let listener = metadata
                        .add_listener_local()
                        .property(move |_subject, key, _type_, value| {
                            match key {
                                Some("default.audio.sink") => {
                                    let name = value.and_then(|v| {
                                        // JSON is like {"name":"..."}
                                        extract_name_from_json(v)
                                    });
                                    state_meta.write().default_sink = name;
                                }
                                Some("default.audio.source") => {
                                    let name = value.and_then(extract_name_from_json);
                                    state_meta.write().default_source = name;
                                }
                                _ => {}
                            }
                            0
                        })
                        .register();

                    proxies_reg
                        .borrow_mut()
                        .metadata
                        .push((metadata, Box::new(listener)));
                }

                _ => {}
            }
        })
        .global_remove({
            let state = Arc::clone(&state);
            let proxies = Rc::clone(&proxies);
            move |id| {
                state.write().nodes.remove(&id);
                proxies.borrow_mut().nodes.remove(&id);
                debug!("Audio node removed: id={id}");
            }
        })
        .register();

    // Command receiver
    let ml_weak = main_loop.downgrade();
    let proxies_cmd = Rc::clone(&proxies);

    let _cmd_attached = cmd_rx.attach(main_loop.loop_(), move |cmd| match cmd {
        Command::SetVolume {
            node_id,
            channels,
            volume,
            reply,
        } => {
            let proxies = proxies_cmd.borrow();
            let result = match proxies.nodes.get(&node_id) {
                Some((node, _)) => {
                    let pod_bytes = build_volume_pod(channels, volume);
                    let pod = unsafe { Pod::from_raw(pod_bytes.as_ptr().cast()) };
                    node.set_param(ParamType::Props, 0, pod);
                    Ok(())
                }
                None => Err(AudioError::SetVolumeFailed {
                    device: node_id.to_string(),
                    reason: "node not found in proxy table".into(),
                }),
            };
            let _ = reply.send(result);
        }

        Command::SetMute {
            node_id,
            muted,
            reply,
        } => {
            let proxies = proxies_cmd.borrow();
            let result = match proxies.nodes.get(&node_id) {
                Some((node, _)) => {
                    let pod_bytes = build_mute_pod(muted);
                    let pod = unsafe { Pod::from_raw(pod_bytes.as_ptr().cast()) };
                    node.set_param(ParamType::Props, 0, pod);
                    Ok(())
                }
                None => Err(AudioError::SetMuteFailed {
                    device: node_id.to_string(),
                    reason: "node not found in proxy table".into(),
                }),
            };
            let _ = reply.send(result);
        }

        Command::SetDefaultOutput { node_name, reply } => {
            let proxies = proxies_cmd.borrow();
            let result = match proxies.metadata.first() {
                Some((meta, _)) => {
                    let value = format!("{{\"name\":\"{node_name}\"}}");
                    meta.set_property(
                        0,
                        "default.audio.sink",
                        Some("Spa:String:JSON"),
                        Some(&value),
                    );
                    Ok(())
                }
                None => Err(AudioError::SetDefaultFailed {
                    direction: "output".into(),
                    reason: "no default metadata object found".into(),
                }),
            };
            let _ = reply.send(result);
        }

        Command::SetDefaultInput { node_name, reply } => {
            let proxies = proxies_cmd.borrow();
            let result = match proxies.metadata.first() {
                Some((meta, _)) => {
                    let value = format!("{{\"name\":\"{node_name}\"}}");
                    meta.set_property(
                        0,
                        "default.audio.source",
                        Some("Spa:String:JSON"),
                        Some(&value),
                    );
                    Ok(())
                }
                None => Err(AudioError::SetDefaultFailed {
                    direction: "input".into(),
                    reason: "no default metadata object found".into(),
                }),
            };
            let _ = reply.send(result);
        }

        Command::Quit => {
            if let Some(ml) = ml_weak.upgrade() {
                ml.quit();
            }
        }
    });

    // Signal that init is done
    let _ = ready_tx.send(Ok(()));

    main_loop.run();
}

// SPA pod helpers

// Parse Props pod and update state
fn parse_props_pod(pod: &Pod, node_id: u32, state: &RwLock<SharedState>) {
    use pipewire::spa::pod::deserialize::PodDeserializer;

    let bytes = pod.as_bytes();
    let Ok((_, value)) = PodDeserializer::deserialize_from::<Value>(bytes) else {
        return;
    };

    let Value::Object(obj) = value else {
        return;
    };

    let mut volume: Option<f32> = None;
    let mut muted: Option<bool> = None;

    for prop in &obj.properties {
        match prop.key {
            pipewire::spa::sys::SPA_PROP_channelVolumes => {
                if let Value::ValueArray(ValueArray::Float(vols)) = &prop.value
                    && !vols.is_empty()
                {
                    volume = Some(vols.iter().copied().sum::<f32>() / vols.len() as f32);
                }
            }
            pipewire::spa::sys::SPA_PROP_mute => {
                if let Value::Bool(m) = prop.value {
                    muted = Some(m);
                }
            }
            _ => {}
        }
    }

    let mut s = state.write();
    if let Some(node) = s.nodes.get_mut(&node_id) {
        if let Some(v) = volume {
            node.volume = v;
        }
        if let Some(m) = muted {
            node.muted = m;
        }
    }
}

// Build Props pod for volume
fn build_volume_pod(channels: u32, volume: f32) -> Vec<u8> {
    let vols: Vec<f32> = vec![volume; channels as usize];
    let mut bytes = Vec::new();
    let cursor = std::io::Cursor::new(&mut bytes);

    PodSerializer::serialize(
        cursor,
        &Value::Object(Object {
            type_: pipewire::spa::sys::SPA_TYPE_OBJECT_Props,
            id: pipewire::spa::sys::SPA_PARAM_Props,
            properties: vec![Property {
                key: pipewire::spa::sys::SPA_PROP_channelVolumes,
                flags: PropertyFlags::empty(),
                value: Value::ValueArray(ValueArray::Float(vols)),
            }],
        }),
    )
    .expect("volume pod serialization failed");

    bytes
}

// Build Props pod for mute
fn build_mute_pod(muted: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    let cursor = std::io::Cursor::new(&mut bytes);

    PodSerializer::serialize(
        cursor,
        &Value::Object(Object {
            type_: pipewire::spa::sys::SPA_TYPE_OBJECT_Props,
            id: pipewire::spa::sys::SPA_PARAM_Props,
            properties: vec![Property {
                key: pipewire::spa::sys::SPA_PROP_mute,
                flags: PropertyFlags::empty(),
                value: Value::Bool(muted),
            }],
        }),
    )
    .expect("mute pod serialization failed");

    bytes
}

// Extract 'name' from PW metadata JSON
fn extract_name_from_json(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let inner = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    for part in inner.split(',') {
        let mut kv = part.splitn(2, ':');
        let key = kv.next()?.trim().trim_matches('"');
        let val = kv.next()?.trim().trim_matches('"');
        if key == "name" {
            return Some(val.to_string());
        }
    }
    None
}
