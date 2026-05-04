/// Safely extracts and formats the MAC address from a BlueZ device path.
pub fn extract_mac(path: &str) -> Option<String> {
    let last_segment = path.split("dev_").last()?;

    // If "dev_" wasn't in the string, it's not a valid device path
    if last_segment == path {
        return None;
    }

    let mac = last_segment.replace('_', ":");
    if mac.len() == 17 { Some(mac) } else { None }
}
