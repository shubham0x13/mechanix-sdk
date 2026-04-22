use common::{ValueMapExt, VariantDict};

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
    pub(crate) fn from_properties(path: String, props: &VariantDict) -> Self {
        let name = path.split('/').next_back().unwrap_or("unknown").to_string();
        Self {
            path,
            name,
            alias: props.get_string_or_default("Alias"),
            address: props.get_string_or_default("Address"),
            powered: props.get_as_or_default("Powered"),
            discoverable: props.get_as_or_default("Discoverable"),
            pairable: props.get_as_or_default("Pairable"),
            connectable: props.get_as_or_default("Connectable"),
            discovering: props.get_as_or_default("Discovering"),
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
