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
        pod::{
            serialize::PodSerializer, Object, Pod, Property, PropertyFlags, Value, ValueArray,
        },
    },
    types::ObjectType,
};
use tracing::{debug, warn};

use crate::error::AudioError;
use crate::service::{
    manager::Command,
    state::{NodeSnapshot, SharedState},
};

struct PwProxies {
    nodes: HashMap<u32, (Node, Box<dyn Listener>)>,
    metadata: Vec<(Metadata, Box<dyn Listener>)>,
}

pub(crate) fn pw_thread(
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
                                channels: 2, // updated when first Props event arrives
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

                    node.enum_params(0, Some(ParamType::Props), 0, u32::MAX);

                    proxies_reg
                        .borrow_mut()
                        .nodes
                        .insert(node_id, (node, Box::new(listener)));

                    debug!("Audio node registered: id={node_id}");
                }

                ObjectType::Metadata => {
                    let is_default =
                        obj.props.as_ref().and_then(|p| p.get("metadata.name"))
                            == Some("default");
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
                                    let name = value.and_then(extract_name_from_json);
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

    let _ = ready_tx.send(Ok(()));

    main_loop.run();
}

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
    let mut channels: Option<u32> = None;
    let mut muted: Option<bool> = None;

    for prop in &obj.properties {
        match prop.key {
            pipewire::spa::sys::SPA_PROP_channelVolumes => {
                if let Value::ValueArray(ValueArray::Float(vols)) = &prop.value
                    && !vols.is_empty()
                {
                    channels = Some(vols.len() as u32);
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
        if let Some(c) = channels {
            node.channels = c;
        }
        if let Some(v) = volume {
            node.volume = v;
        }
        if let Some(m) = muted {
            node.muted = m;
        }
    }
}

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

fn extract_name_from_json(value: &str) -> Option<String> {
    let obj: serde_json::Value = serde_json::from_str(value).ok()?;
    obj.get("name")?.as_str().map(str::to_string)
}
