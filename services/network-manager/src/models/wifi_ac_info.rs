use crate::types::{NM80211ApFlags, NM80211ApSecurityFlags};

#[derive(Debug, Clone)]
pub struct WifiAccessPointInfo {
    pub path: String,
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub max_bitrate: u32,
    pub flags: NM80211ApFlags,
    pub wpa_flags: NM80211ApSecurityFlags,
    pub rsn_flags: NM80211ApSecurityFlags,
    pub last_seen: i32,
}

impl WifiAccessPointInfo {
    pub fn is_secure(&self) -> bool {
        !self.wpa_flags.is_empty() || !self.rsn_flags.is_empty() || !self.flags.is_empty()
    }
}
