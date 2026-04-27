pub mod dbus;
pub use dbus::*;

use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

/// Represents a standard D-Bus dictionary (a{sv})
pub type VariantDict = HashMap<String, OwnedValue>;

/// Represents a nested D-Bus dictionary (a{sa{sv}})
pub type NestedVariantDict = HashMap<String, VariantDict>;

/// Extension trait for extracting strongly-typed values from a D-Bus dictionary.
pub trait ValueMapExt {
    /// Extracts a value of type `T`, returning `None` if missing or if the type mismatches.
    fn get_as<'a, T>(&'a self, key: &str) -> Option<T>
    where
        T: TryFrom<&'a OwnedValue>;

    /// Extracts a value, returning the provided `default` if missing or invalid.
    fn get_as_or<'a, T>(&'a self, key: &str, default: T) -> T
    where
        T: TryFrom<&'a OwnedValue>;

    /// Extracts a value, returning `T::default()` if missing or invalid.
    fn get_as_or_default<'a, T>(&'a self, key: &str) -> T
    where
        T: TryFrom<&'a OwnedValue> + Default;

    /// Convenience method for extracting an owned `String`.
    fn get_string(&self, key: &str) -> Option<String>;

    /// Convenience method for extracting an owned `String` with a fallback.
    fn get_string_or(&self, key: &str, default: &str) -> String;

    /// Convenience method for extracting an owned `String` with an empty string fallback.
    fn get_string_or_default(&self, key: &str) -> String;
}

impl ValueMapExt for VariantDict {
    fn get_as<'a, T>(&'a self, key: &str) -> Option<T>
    where
        T: TryFrom<&'a OwnedValue>,
    {
        self.get(key).and_then(|v| T::try_from(v).ok())
    }

    fn get_as_or<'a, T>(&'a self, key: &str, default: T) -> T
    where
        T: TryFrom<&'a OwnedValue>,
    {
        self.get_as::<T>(key).unwrap_or(default)
    }

    fn get_as_or_default<'a, T>(&'a self, key: &str) -> T
    where
        T: TryFrom<&'a OwnedValue> + Default,
    {
        self.get_as::<T>(key).unwrap_or_default()
    }

    fn get_string(&self, key: &str) -> Option<String> {
        self.get_as::<&str>(key).map(String::from)
    }

    fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get_string(key).unwrap_or_else(|| default.to_owned())
    }

    fn get_string_or_default(&self, key: &str) -> String {
        self.get_string(key).unwrap_or_default()
    }
}
