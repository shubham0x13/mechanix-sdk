use std::collections::HashMap;

use crate::types::{AudioDevice, DeviceType};

#[derive(Debug, Clone)]
pub(crate) struct NodeSnapshot {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub media_class: Option<String>,
    pub volume: f32,
    pub muted: bool,
    pub channels: u32,
}

impl Default for NodeSnapshot {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            description: None,
            media_class: None,
            volume: 1.0,
            muted: false,
            channels: 2,
        }
    }
}

impl NodeSnapshot {
    pub fn device_type(&self) -> Option<DeviceType> {
        match self.media_class.as_deref() {
            Some("Audio/Sink") => Some(DeviceType::Output),
            Some("Audio/Source") => Some(DeviceType::Input),
            _ => None,
        }
    }

    pub fn into_audio_device(self, is_default: bool) -> Option<AudioDevice> {
        let device_type = self.device_type()?;
        Some(AudioDevice {
            id: self.id,
            name: self.name,
            description: self.description,
            device_type,
            channels: self.channels,
            volume: self.volume,
            muted: self.muted,
            is_default,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct SharedState {
    pub nodes: HashMap<u32, NodeSnapshot>,
    pub default_sink: Option<String>,
    pub default_source: Option<String>,
}
