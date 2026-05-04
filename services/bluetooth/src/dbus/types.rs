use serde::Deserialize;
use zbus::zvariant::{OwnedValue, Type};

#[derive(Deserialize, Type, OwnedValue, Debug, Clone, PartialEq, Eq)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum AddressType {
    Public,
    Random,

    #[serde(other)]
    Unknown,
}
