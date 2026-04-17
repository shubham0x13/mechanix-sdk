use crate::DbusProperties;
use common::{extract_bool, extract_string};

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub path: String,
    pub name: String,
    pub alias: String,
    pub address: String,
    pub powered: bool,
    pub discoverable: bool,
    pub pairable: bool,
    pub connectable: bool,
    pub discovering: bool,
}

impl AdapterInfo {
    pub(crate) fn from_properties(path: String, props: &DbusProperties) -> Self {
        let name = path.split('/').next_back().unwrap_or("unknown").to_string();
        Self {
            path,
            name,
            alias: extract_string(props, "Alias").unwrap_or_default(),
            address: extract_string(props, "Address").unwrap_or_default(),
            powered: extract_bool(props, "Powered").unwrap_or(false),
            discoverable: extract_bool(props, "Discoverable").unwrap_or(false),
            pairable: extract_bool(props, "Pairable").unwrap_or(false),
            connectable: extract_bool(props, "Connectable").unwrap_or(false),
            discovering: extract_bool(props, "Discovering").unwrap_or(false),
        }
    }

    pub fn display_name(&self) -> String {
        if !self.alias.is_empty() {
            self.alias.clone()
        } else {
            self.name.clone()
        }
    }
}
