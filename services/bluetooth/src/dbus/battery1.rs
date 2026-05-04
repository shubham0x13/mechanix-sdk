use zbus::{
    proxy,
    zvariant::{DeserializeDict, Type},
};

#[derive(DeserializeDict, Type, Default, Debug, Clone)]
#[zvariant(signature = "a{sv}", rename_all = "PascalCase")]
pub struct Battery1Properties {
    pub percentage: Option<u8>,
    pub source: Option<String>,
}

#[proxy(interface = "org.bluez.Battery1", default_service = "org.bluez")]
pub trait Battery1 {
    /// Percentage property
    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<u8>;

    /// Source property
    #[zbus(property)]
    fn source(&self) -> zbus::Result<String>;
}
