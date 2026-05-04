use zbus::zvariant::OwnedObjectPath;

use crate::dbus::Adapter1Properties;

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub path: OwnedObjectPath,
    pub properties: Adapter1Properties,
}

impl AdapterInfo {
    pub(crate) fn new(path: OwnedObjectPath, properties: Adapter1Properties) -> Self {
        Self { path, properties }
    }

    pub fn display_name(&self) -> &str {
        let p = &self.properties;

        p.alias
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| p.name.as_deref().filter(|s| !s.is_empty()))
            .or_else(|| p.address.as_deref().filter(|s| !s.is_empty()))
            .unwrap_or("Unknown Adapter")
    }
}
