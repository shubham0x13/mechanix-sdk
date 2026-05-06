#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    Output,
    Input,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub device_type: DeviceType,
    pub channels: u32,
    pub volume: f32,
    pub muted: bool,
    pub is_default: bool,
}

impl AudioDevice {
    pub fn is_output(&self) -> bool {
        self.device_type == DeviceType::Output
    }

    pub fn is_input(&self) -> bool {
        self.device_type == DeviceType::Input
    }
}
