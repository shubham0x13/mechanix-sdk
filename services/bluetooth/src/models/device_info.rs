use zbus::zvariant::OwnedObjectPath;

use crate::dbus::Device1Properties;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: OwnedObjectPath,
    pub properties: Device1Properties,
}

impl DeviceInfo {
    pub(crate) fn new(path: OwnedObjectPath, properties: Device1Properties) -> Self {
        Self { path, properties }
    }

    pub fn display_name(&self) -> &str {
        let p = &self.properties;

        p.alias
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| p.name.as_deref().filter(|s| !s.is_empty()))
            .or_else(|| p.address.as_deref().filter(|s| !s.is_empty()))
            .unwrap_or("Unknown Device")
    }
}
