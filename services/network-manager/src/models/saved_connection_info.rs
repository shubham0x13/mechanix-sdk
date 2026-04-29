use common::{NestedVariantDict, ValueMapExt};

#[derive(Debug, Clone)]
pub struct SavedConnectionInfo {
    pub path: String,
    pub id: String,
    pub uuid: String,
    pub connection_type: String,
    pub autoconnect: bool,
}

impl SavedConnectionInfo {
    pub fn from_settings(path: String, settings: &NestedVariantDict) -> Option<Self> {
        let conn = settings.get("connection")?;

        Some(Self {
            path,
            id: conn.get_string("id")?,
            uuid: conn.get_string("uuid")?,
            connection_type: conn.get_string("type")?,
            autoconnect: conn.get_as_or("autoconnect", true),
        })
    }
}
