use crate::types::NMActiveConnectionState;

#[derive(Debug, Clone)]
pub struct ActiveConnectionInfo {
    pub path: String,
    pub id: String,
    pub uuid: String,
    pub connection_type: String,
    pub state: NMActiveConnectionState,
    pub is_default_ipv4: bool,
    pub is_default_ipv6: bool,
    pub is_vpn: bool,
}
