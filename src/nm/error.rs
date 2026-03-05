use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum NmError {
    #[error("D-Bus error: {0}")]
    Dbus(#[from] zbus::Error),

    #[error("D-Bus fdo error: {0}")]
    Fdo(#[from] zbus::fdo::Error),

    #[error("No WiFi device found")]
    NoWifiDevice,

    #[error("WiFi is disabled")]
    WifiDisabled,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("No active connection")]
    NoActiveConnection,

    #[error("Access point not found: {0}")]
    AccessPointNotFound(String),

    #[error("Invalid SSID encoding")]
    InvalidSsid,

    #[error("Property error: {0}")]
    Property(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("ZVariant error: {0}")]
    ZVariant(#[from] zvariant::Error),
}

pub type Result<T> = std::result::Result<T, NmError>;
