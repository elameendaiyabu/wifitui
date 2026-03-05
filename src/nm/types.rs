use std::fmt;

use bitflags::bitflags;

#[derive(Clone, Debug, PartialEq)]
pub enum SecurityType {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
    Wpa2Enterprise,
    Unknown,
}

impl fmt::Display for SecurityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityType::Open => write!(f, "Open"),
            SecurityType::Wep => write!(f, "WEP"),
            SecurityType::Wpa => write!(f, "WPA"),
            SecurityType::Wpa2 => write!(f, "WPA2"),
            SecurityType::Wpa3 => write!(f, "WPA3"),
            SecurityType::Wpa2Enterprise => write!(f, "WPA2-EAP"),
            SecurityType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl SecurityType {
    pub fn from_ap_flags(flags: u32, wpa_flags: u32, rsn_flags: u32) -> Self {
        let rsn = ApSecurityFlags::from_bits_truncate(rsn_flags);
        let wpa = ApSecurityFlags::from_bits_truncate(wpa_flags);

        if rsn.contains(ApSecurityFlags::KEY_MGMT_SAE) {
            SecurityType::Wpa3
        } else if rsn.contains(ApSecurityFlags::KEY_MGMT_8021X) {
            SecurityType::Wpa2Enterprise
        } else if rsn.contains(ApSecurityFlags::KEY_MGMT_PSK) {
            SecurityType::Wpa2
        } else if wpa.contains(ApSecurityFlags::KEY_MGMT_PSK) {
            SecurityType::Wpa
        } else if ApFlags::from_bits_truncate(flags).contains(ApFlags::PRIVACY) {
            SecurityType::Wep
        } else {
            SecurityType::Open
        }
    }

    pub fn requires_password(&self) -> bool {
        !matches!(self, SecurityType::Open)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Known,
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Connecting => write!(f, "Connecting"),
            ConnectionStatus::Disconnected => write!(f, ""),
            ConnectionStatus::Known => write!(f, "Saved"),
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ApFlags: u32 {
        const NONE    = 0x0;
        const PRIVACY = 0x1;
        const WPS     = 0x2;
        const WPS_PBC = 0x4;
        const WPS_PIN = 0x8;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ApSecurityFlags: u32 {
        const NONE           = 0x000;
        const PAIR_WEP40     = 0x001;
        const PAIR_WEP104    = 0x002;
        const PAIR_TKIP      = 0x004;
        const PAIR_CCMP      = 0x008;
        const GROUP_WEP40    = 0x010;
        const GROUP_WEP104   = 0x020;
        const GROUP_TKIP     = 0x040;
        const GROUP_CCMP     = 0x080;
        const KEY_MGMT_PSK   = 0x100;
        const KEY_MGMT_8021X = 0x200;
        const KEY_MGMT_SAE   = 0x400;
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum NmDeviceState {
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

impl NmDeviceState {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => NmDeviceState::Unknown,
            10 => NmDeviceState::Unmanaged,
            20 => NmDeviceState::Unavailable,
            30 => NmDeviceState::Disconnected,
            40 => NmDeviceState::Prepare,
            50 => NmDeviceState::Config,
            60 => NmDeviceState::NeedAuth,
            70 => NmDeviceState::IpConfig,
            80 => NmDeviceState::IpCheck,
            90 => NmDeviceState::Secondaries,
            100 => NmDeviceState::Activated,
            110 => NmDeviceState::Deactivating,
            120 => NmDeviceState::Failed,
            _ => NmDeviceState::Unknown,
        }
    }
}

impl fmt::Display for NmDeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NmDeviceState::Activated => write!(f, "Connected"),
            NmDeviceState::Disconnected => write!(f, "Disconnected"),
            NmDeviceState::Unavailable => write!(f, "Unavailable"),
            NmDeviceState::Unmanaged => write!(f, "Unmanaged"),
            NmDeviceState::Prepare => write!(f, "Preparing"),
            NmDeviceState::Config => write!(f, "Configuring"),
            NmDeviceState::NeedAuth => write!(f, "Authenticating"),
            NmDeviceState::IpConfig => write!(f, "Getting IP"),
            NmDeviceState::IpCheck => write!(f, "Checking IP"),
            NmDeviceState::Secondaries => write!(f, "Secondaries"),
            NmDeviceState::Deactivating => write!(f, "Disconnecting"),
            NmDeviceState::Failed => write!(f, "Failed"),
            NmDeviceState::Unknown => write!(f, "Unknown"),
        }
    }
}
