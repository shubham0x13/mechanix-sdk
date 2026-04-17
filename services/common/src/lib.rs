use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

pub type ValueMap = HashMap<String, OwnedValue>;

// ---------- Public Extraction Helpers ----------

pub fn extract_i16(props: &ValueMap, key: &str) -> Option<i16> {
    props.get(key).and_then(|v| i16::try_from(v).ok())
}

pub fn extract_u32(props: &ValueMap, key: &str) -> Option<u32> {
    props.get(key).and_then(|v| u32::try_from(v).ok())
}

pub fn extract_bool(props: &ValueMap, key: &str) -> Option<bool> {
    props.get(key).and_then(|v| bool::try_from(v).ok())
}

pub fn extract_string(props: &ValueMap, key: &str) -> Option<String> {
    props
        .get(key)
        .and_then(|v| <&str>::try_from(v).ok().map(String::from))
}
