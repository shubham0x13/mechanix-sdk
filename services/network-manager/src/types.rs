// NetworkManager D-Bus type definitions, manually transcribed from:
// https://networkmanager.dev/docs/api/latest/nm-dbus-types.html
//
// Plain enums use `num_enum::TryFromPrimitive` for fallible integer conversion.
// Flag types use the `bitflags!` macro to allow combining values with | & etc.
// Deprecated aliases are preserved as `#[deprecated]` consts on the enum type.
//
// This file should be updated manually when new NM releases add entries.
// The upstream page is stable versioned API, so changes are infrequent.

use bitflags::bitflags;
use num_enum::TryFromPrimitive;

// ——— NMVersionInfoCapability ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMVersionInfoCapability {
    SyncRouteWithTable = 0,
    Ip4Forwarding = 1,
    SriovPreserveOnDown = 2,
}

// ——— NMCapability ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMCapability {
    Team = 1,
    Ovs = 2,
}

// ——— NMState ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMState {
    Unknown = 0,
    Disabled = 10,
    Disconnected = 20,
    Disconnecting = 30,
    Connecting = 40,
    ConnectedLocal = 50,
    ConnectedSite = 60,
    ConnectedGlobal = 70,
}

impl NMState {
    #[deprecated(note = "Use NMState::Disabled instead")]
    pub const ASLEEP: Self = Self::Disabled;
}

// ——— NMConnectivityState ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMConnectivityState {
    Unknown = 0,
    None = 1,
    Portal = 2,
    Limited = 3,
    Full = 4,
}

// ——— NMDeviceType ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMDeviceType {
    Unknown = 0,
    Ethernet = 1,
    Wifi = 2,
    Unused1 = 3,
    Unused2 = 4,
    Bt = 5,
    OlpcMesh = 6,
    Wimax = 7,
    Modem = 8,
    Infiniband = 9,
    Bond = 10,
    Vlan = 11,
    Adsl = 12,
    Bridge = 13,
    Generic = 14,
    Team = 15,
    Tun = 16,
    IpTunnel = 17,
    Macvlan = 18,
    Vxlan = 19,
    Veth = 20,
    Macsec = 21,
    Dummy = 22,
    Ppp = 23,
    OvsInterface = 24,
    OvsPort = 25,
    OvsBridge = 26,
    Wpan = 27,
    Lowpan6 = 28,
    Wireguard = 29,
    WifiP2p = 30,
    Vrf = 31,
    Loopback = 32,
    Hsr = 33,
    Ipvlan = 34,
}

// ——— NMDeviceCapabilities ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceCapabilities: u32 {
        const NONE           = 0x00000000;
        const NM_SUPPORTED   = 0x00000001;
        const CARRIER_DETECT = 0x00000002;
        const IS_SOFTWARE    = 0x00000004;
        const SRIOV          = 0x00000008;
    }
}

// ——— NMDeviceWifiCapabilities ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceWifiCapabilities: u32 {
        const NONE          = 0x00000000;
        const CIPHER_WEP40  = 0x00000001;
        const CIPHER_WEP104 = 0x00000002;
        const CIPHER_TKIP   = 0x00000004;
        const CIPHER_CCMP   = 0x00000008;
        const WPA           = 0x00000010;
        const RSN           = 0x00000020;
        const AP            = 0x00000040;
        const ADHOC         = 0x00000080;
        const FREQ_VALID    = 0x00000100;
        const FREQ_2GHZ     = 0x00000200;
        const FREQ_5GHZ     = 0x00000400;
        const FREQ_6GHZ     = 0x00000800;
        const MESH          = 0x00001000;
        const IBSS_RSN      = 0x00002000;
    }
}

// ——— NM80211ApFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NM80211ApFlags: u32 {
        const NONE    = 0x00000000;
        const PRIVACY = 0x00000001;
        const WPS     = 0x00000002;
        const WPS_PBC = 0x00000004;
        const WPS_PIN = 0x00000008;
    }
}

// ——— NM80211ApSecurityFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NM80211ApSecurityFlags: u32 {
        const NONE                    = 0x00000000;
        const PAIR_WEP40              = 0x00000001;
        const PAIR_WEP104             = 0x00000002;
        const PAIR_TKIP               = 0x00000004;
        const PAIR_CCMP               = 0x00000008;
        const GROUP_WEP40             = 0x00000010;
        const GROUP_WEP104            = 0x00000020;
        const GROUP_TKIP              = 0x00000040;
        const GROUP_CCMP              = 0x00000080;
        const KEY_MGMT_PSK            = 0x00000100;
        const KEY_MGMT_802_1X         = 0x00000200;
        const KEY_MGMT_SAE            = 0x00000400;
        const KEY_MGMT_OWE            = 0x00000800;
        const KEY_MGMT_OWE_TM         = 0x00001000;
        const KEY_MGMT_EAP_SUITE_B192 = 0x00002000;
    }
}

// ——— NM80211Mode ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NM80211Mode {
    Unknown = 0,
    Adhoc = 1,
    Infra = 2,
    Ap = 3,
    Mesh = 4,
}

// ——— NMBluetoothCapabilities ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMBluetoothCapabilities: u32 {
        const NONE = 0x00000000;
        const DUN  = 0x00000001;
        const NAP  = 0x00000002;
    }
}

// ——— NMDeviceModemCapabilities ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceModemCapabilities: u32 {
        const NONE      = 0x00000000;
        const POTS      = 0x00000001;
        const CDMA_EVDO = 0x00000002;
        const GSM_UMTS  = 0x00000004;
        const LTE       = 0x00000008;
        const NR5G      = 0x00000040;
    }
}

// ——— NMWimaxNspNetworkType ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMWimaxNspNetworkType {
    Unknown = 0,
    Home = 1,
    Partner = 2,
    RoamingPartner = 3,
}

// ——— NMDeviceState ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMDeviceState {
    Unknown = 0,
    Unmanaged = 10,
    Unavailable = 20,
    Disconnected = 30,
    Prepare = 40,
    Config = 50,
    NeedAuth = 60,
    IpConfig = 70,
    IpCheck = 80,
    Secondaries = 90,
    Activated = 100,
    Deactivating = 110,
    Failed = 120,
}

// ——— NMDeviceStateReason ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMDeviceStateReason {
    None = 0,
    Unknown = 1,
    NowManaged = 2,
    NowUnmanaged = 3,
    ConfigFailed = 4,
    IpConfigUnavailable = 5,
    IpConfigExpired = 6,
    NoSecrets = 7,
    SupplicantDisconnect = 8,
    SupplicantConfigFailed = 9,
    SupplicantFailed = 10,
    SupplicantTimeout = 11,
    PppStartFailed = 12,
    PppDisconnect = 13,
    PppFailed = 14,
    DhcpStartFailed = 15,
    DhcpError = 16,
    DhcpFailed = 17,
    SharedStartFailed = 18,
    SharedFailed = 19,
    AutoipStartFailed = 20,
    AutoipError = 21,
    AutoipFailed = 22,
    ModemBusy = 23,
    ModemNoDialTone = 24,
    ModemNoCarrier = 25,
    ModemDialTimeout = 26,
    ModemDialFailed = 27,
    ModemInitFailed = 28,
    GsmApnFailed = 29,
    GsmRegistrationNotSearching = 30,
    GsmRegistrationDenied = 31,
    GsmRegistrationTimeout = 32,
    GsmRegistrationFailed = 33,
    GsmPinCheckFailed = 34,
    FirmwareMissing = 35,
    Removed = 36,
    Sleeping = 37,
    ConnectionRemoved = 38,
    UserRequested = 39,
    Carrier = 40,
    ConnectionAssumed = 41,
    SupplicantAvailable = 42,
    ModemNotFound = 43,
    BtFailed = 44,
    GsmSimNotInserted = 45,
    GsmSimPinRequired = 46,
    GsmSimPukRequired = 47,
    GsmSimWrong = 48,
    InfinibandMode = 49,
    DependencyFailed = 50,
    Br2684Failed = 51,
    ModemManagerUnavailable = 52,
    SsidNotFound = 53,
    SecondaryConnectionFailed = 54,
    DcbFcoeFailed = 55,
    TeamdControlFailed = 56,
    ModemFailed = 57,
    ModemAvailable = 58,
    SimPinIncorrect = 59,
    NewActivation = 60,
    ParentChanged = 61,
    ParentManagedChanged = 62,
    OvsdbFailed = 63,
    IpAddressDuplicate = 64,
    IpMethodUnsupported = 65,
    SriovConfigurationFailed = 66,
    PeerNotFound = 67,
    DeviceHandlerFailed = 68,
    UnmanagedByDefault = 69,
    UnmanagedExternalDown = 70,
    UnmanagedLinkNotInit = 71,
    UnmanagedQuitting = 72,
    UnmanagedManagerDisabled = 73,
    UnmanagedUserConf = 74,
    UnmanagedUserExplicit = 75,
    UnmanagedUserSettings = 76,
    UnmanagedUserUdev = 77,
    NetworkingOff = 78,
    ModemNoOperatorCode = 79,
}

impl NMDeviceStateReason {
    #[deprecated(note = "Use NMDeviceStateReason::UnmanagedManagerDisabled instead")]
    pub const UNMANAGED_SLEEPING: Self = Self::UnmanagedManagerDisabled;
}

// ——— NMMetered ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMMetered {
    Unknown = 0,
    Yes = 1,
    No = 2,
    GuessYes = 3,
    GuessNo = 4,
}

// ——— NMConnectionMultiConnect ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMConnectionMultiConnect {
    Default = 0,
    Single = 1,
    ManualMultiple = 2,
    Multiple = 3,
}

// ——— NMActiveConnectionState ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMActiveConnectionState {
    Unknown = 0,
    Activating = 1,
    Activated = 2,
    Deactivating = 3,
    Deactivated = 4,
}

// ——— NMActiveConnectionStateReason ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMActiveConnectionStateReason {
    Unknown = 0,
    None = 1,
    UserDisconnected = 2,
    DeviceDisconnected = 3,
    ServiceStopped = 4,
    IpConfigInvalid = 5,
    ConnectTimeout = 6,
    ServiceStartTimeout = 7,
    ServiceStartFailed = 8,
    NoSecrets = 9,
    LoginFailed = 10,
    ConnectionRemoved = 11,
    DependencyFailed = 12,
    DeviceRealizeFailed = 13,
    DeviceRemoved = 14,
}

// ——— NMSecretAgentGetSecretsFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMSecretAgentGetSecretsFlags: u32 {
        const NONE              = 0x00000000;
        const ALLOW_INTERACTION = 0x00000001;
        const REQUEST_NEW       = 0x00000002;
        const USER_REQUESTED    = 0x00000004;
        const WPS_PBC_ACTIVE    = 0x00000008;
        /// Internal flag, not part of the D-Bus API.
        const NO_ERRORS         = 0x40000000;
        /// Internal flag, not part of the D-Bus API.
        const ONLY_SYSTEM       = 0x80000000;
    }
}

// ——— NMSecretAgentCapabilities ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMSecretAgentCapabilities: u32 {
        const NONE      = 0x00000000;
        const VPN_HINTS = 0x00000001;
    }
}

// ——— NMIPTunnelMode ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMIPTunnelMode {
    Unknown = 0,
    Ipip = 1,
    Gre = 2,
    Sit = 3,
    Isatap = 4,
    Vti = 5,
    Ip6ip6 = 6,
    Ipip6 = 7,
    Ip6gre = 8,
    Vti6 = 9,
    Gretap = 10,
    Ip6gretap = 11,
}

// ——— NMCheckpointCreateFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMCheckpointCreateFlags: u32 {
        const NONE                      = 0x00;
        const DESTROY_ALL                = 0x01;
        const DELETE_NEW_CONNECTIONS     = 0x02;
        const DISCONNECT_NEW_DEVICES     = 0x04;
        const ALLOW_OVERLAPPING          = 0x08;
        const NO_PRESERVE_EXTERNAL_PORTS = 0x10;
        const TRACK_INTERNAL_GLOBAL_DNS  = 0x20;
    }
}

// ——— NMRollbackResult ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMRollbackResult {
    Ok = 0,
    ErrNoDevice = 1,
    ErrDeviceUnmanaged = 2,
    ErrFailed = 3,
}

// ——— NMSettingsConnectionFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMSettingsConnectionFlags: u32 {
        const NONE         = 0x00;
        const UNSAVED      = 0x01;
        const NM_GENERATED = 0x02;
        const VOLATILE     = 0x04;
        const EXTERNAL     = 0x08;
    }
}

// ——— NMActivationStateFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMActivationStateFlags: u32 {
        const NONE                                 = 0x00;
        const IS_CONTROLLER                        = 0x01;
        const IS_PORT                              = 0x02;
        const LAYER2_READY                         = 0x04;
        const IP4_READY                            = 0x08;
        const IP6_READY                            = 0x10;
        const CONTROLLER_HAS_PORTS                 = 0x20;
        const LIFETIME_BOUND_TO_PROFILE_VISIBILITY = 0x40;
        const EXTERNAL                             = 0x80;
    }
}

// ——— NMSettingsAddConnection2Flags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMSettingsAddConnection2Flags: u32 {
        const NONE              = 0x00;
        const TO_DISK           = 0x01;
        const IN_MEMORY         = 0x02;
        const BLOCK_AUTOCONNECT = 0x20;
    }
}

// ——— NMSettingsUpdate2Flags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMSettingsUpdate2Flags: u32 {
        const NONE               = 0x00;
        const TO_DISK            = 0x01;
        const IN_MEMORY          = 0x02;
        const IN_MEMORY_DETACHED = 0x04;
        const IN_MEMORY_ONLY     = 0x08;
        const VOLATILE           = 0x10;
        const BLOCK_AUTOCONNECT  = 0x20;
        const NO_REAPPLY         = 0x40;
    }
}

// ——— NMDeviceReapplyFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceReapplyFlags: u32 {
        const NONE                 = 0x00;
        const PRESERVE_EXTERNAL_IP = 0x01;
    }
}

// ——— NMTernary ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(i32)]
pub enum NMTernary {
    Default = -1,
    False = 0,
    True = 1,
}

// ——— NMManagerReloadFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMManagerReloadFlags: u32 {
        const NONE     = 0x00;
        const CONF     = 0x01;
        const DNS_RC   = 0x02;
        const DNS_FULL = 0x04;
        const ALL      = 0x07;
    }
}

// ——— NMDeviceInterfaceFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMDeviceInterfaceFlags: u32 {
        const NONE                = 0x00000;
        const UP                  = 0x00001;
        const LOWER_UP            = 0x00002;
        const PROMISC             = 0x00004;
        const CARRIER             = 0x10000;
        const LLDP_CLIENT_ENABLED = 0x20000;
    }
}

// ——— NMClientPermission ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMClientPermission {
    None = 0,
    EnableDisableNetwork = 1,
    EnableDisableWifi = 2,
    EnableDisableWwan = 3,
    EnableDisableWimax = 4,
    SleepWake = 5,
    NetworkControl = 6,
    WifiShareProtected = 7,
    WifiShareOpen = 8,
    SettingsModifySystem = 9,
    SettingsModifyOwn = 10,
    SettingsModifyHostname = 11,
    SettingsModifyGlobalDns = 12,
    Reload = 13,
    CheckpointRollback = 14,
    EnableDisableStatistics = 15,
    EnableDisableConnectivityCheck = 16,
    WifiScan = 17,
}

// ——— NMClientPermissionResult ———

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum NMClientPermissionResult {
    Unknown = 0,
    Yes = 1,
    Auth = 2,
    No = 3,
}

// ——— NMRadioFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMRadioFlags: u32 {
        const NONE           = 0x00;
        const WLAN_AVAILABLE = 0x01;
        const WWAN_AVAILABLE = 0x02;
    }
}

// ——— NMMptcpFlags ———

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NMMptcpFlags: u32 {
        const NONE                       = 0x000;
        const DISABLED                   = 0x001;
        const ENABLED                    = 0x002;
        const ALSO_WITHOUT_SYSCTL        = 0x004;
        const ALSO_WITHOUT_DEFAULT_ROUTE = 0x008;
        const SIGNAL                     = 0x010;
        const SUBFLOW                    = 0x020;
        const BACKUP                     = 0x040;
        const FULLMESH                   = 0x080;
        const LAMINAR                    = 0x100;
    }
}
