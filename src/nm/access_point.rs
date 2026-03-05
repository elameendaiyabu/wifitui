use zbus::Connection;
use zbus::zvariant::OwnedObjectPath;

use super::error::Result;
use super::types::SecurityType;

#[derive(Clone, Debug)]
pub struct AccessPointInfo {
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub security: SecurityType,
}

pub async fn fetch_ap_info(
    conn: &Connection,
    ap_path: &OwnedObjectPath,
) -> Result<Option<AccessPointInfo>> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        ap_path.as_ref(),
        "org.freedesktop.NetworkManager.AccessPoint",
    )
    .await?;

    let ssid_bytes: Vec<u8> = match proxy.get_property("Ssid").await {
        Ok(val) => val,
        Err(_) => return Ok(None),
    };

    let ssid = String::from_utf8_lossy(&ssid_bytes).to_string();

    // Skip hidden networks (empty SSID)
    if ssid.is_empty() {
        return Ok(None);
    }

    let strength: u8 = proxy.get_property("Strength").await.unwrap_or(0);
    let frequency: u32 = proxy.get_property("Frequency").await.unwrap_or(0);
    let flags: u32 = proxy.get_property("Flags").await.unwrap_or(0);
    let wpa_flags: u32 = proxy.get_property("WpaFlags").await.unwrap_or(0);
    let rsn_flags: u32 = proxy.get_property("RsnFlags").await.unwrap_or(0);
    let bssid: String = proxy
        .get_property("HwAddress")
        .await
        .unwrap_or_default();

    let security = SecurityType::from_ap_flags(flags, wpa_flags, rsn_flags);

    Ok(Some(AccessPointInfo {
        ssid,
        bssid,
        strength,
        frequency,
        security,
    }))
}
