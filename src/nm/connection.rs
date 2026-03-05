use std::collections::HashMap;

use zbus::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use super::error::{NmError, Result};
use super::types::SecurityType;

/// Connect to a new network with password via AddAndActivateConnection
pub async fn connect_with_password(
    conn: &Connection,
    device_path: &str,
    ap_path: &str,
    ssid: &str,
    password: &str,
    security: &SecurityType,
) -> Result<()> {
    let nm_proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;

    // Build connection settings: a{sa{sv}}
    let mut settings: HashMap<&str, HashMap<&str, Value<'_>>> = HashMap::new();

    // "connection" section
    let mut s_con: HashMap<&str, Value<'_>> = HashMap::new();
    s_con.insert("type", Value::from("802-11-wireless"));
    s_con.insert("id", Value::from(ssid));

    // "802-11-wireless" section
    let mut s_wifi: HashMap<&str, Value<'_>> = HashMap::new();
    s_wifi.insert("ssid", Value::from(ssid.as_bytes().to_vec()));
    s_wifi.insert("mode", Value::from("infrastructure"));

    // "802-11-wireless-security" section
    let mut s_wsec: HashMap<&str, Value<'_>> = HashMap::new();
    match security {
        SecurityType::Wpa | SecurityType::Wpa2 => {
            s_wsec.insert("key-mgmt", Value::from("wpa-psk"));
            s_wsec.insert("psk", Value::from(password));
        }
        SecurityType::Wpa3 => {
            s_wsec.insert("key-mgmt", Value::from("sae"));
            s_wsec.insert("psk", Value::from(password));
        }
        SecurityType::Wep => {
            s_wsec.insert("key-mgmt", Value::from("none"));
            s_wsec.insert("wep-key0", Value::from(password));
        }
        _ => {}
    }

    // "ipv4" section
    let mut s_ip4: HashMap<&str, Value<'_>> = HashMap::new();
    s_ip4.insert("method", Value::from("auto"));

    // "ipv6" section
    let mut s_ip6: HashMap<&str, Value<'_>> = HashMap::new();
    s_ip6.insert("method", Value::from("auto"));

    settings.insert("connection", s_con);
    settings.insert("802-11-wireless", s_wifi);
    if !s_wsec.is_empty() {
        settings.insert("802-11-wireless-security", s_wsec);
    }
    settings.insert("ipv4", s_ip4);
    settings.insert("ipv6", s_ip6);

    let device_obj: OwnedObjectPath = OwnedObjectPath::try_from(device_path.to_string())
        .map_err(|e| NmError::Property(format!("invalid device path: {e}")))?;
    let ap_obj: OwnedObjectPath = OwnedObjectPath::try_from(ap_path.to_string())
        .map_err(|e| NmError::Property(format!("invalid AP path: {e}")))?;

    nm_proxy
        .call_method(
            "AddAndActivateConnection",
            &(settings, &device_obj, &ap_obj),
        )
        .await?;

    Ok(())
}

/// Activate an existing saved connection
pub async fn activate_connection(
    conn: &Connection,
    connection_path: &str,
    device_path: &str,
    ap_path: &str,
) -> Result<()> {
    let nm_proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;

    let conn_obj: OwnedObjectPath = OwnedObjectPath::try_from(connection_path.to_string())
        .map_err(|e| NmError::Property(format!("invalid connection path: {e}")))?;
    let device_obj: OwnedObjectPath = OwnedObjectPath::try_from(device_path.to_string())
        .map_err(|e| NmError::Property(format!("invalid device path: {e}")))?;
    let ap_obj: OwnedObjectPath = OwnedObjectPath::try_from(ap_path.to_string())
        .map_err(|e| NmError::Property(format!("invalid AP path: {e}")))?;

    nm_proxy
        .call_method("ActivateConnection", &(&conn_obj, &device_obj, &ap_obj))
        .await?;

    Ok(())
}

/// Disconnect (deactivate) the current active connection
pub async fn disconnect(conn: &Connection, active_connection_path: &str) -> Result<()> {
    let nm_proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;

    let active_obj: OwnedObjectPath =
        OwnedObjectPath::try_from(active_connection_path.to_string())
            .map_err(|e| NmError::Property(format!("invalid active connection path: {e}")))?;

    nm_proxy
        .call_method("DeactivateConnection", &(&active_obj,))
        .await?;

    Ok(())
}

/// Delete/forget a saved connection profile
pub async fn forget_connection(conn: &Connection, connection_path: &str) -> Result<()> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        connection_path,
        "org.freedesktop.NetworkManager.Settings.Connection",
    )
    .await?;

    proxy.call_method("Delete", &()).await?;
    Ok(())
}

/// Toggle WiFi enabled/disabled
pub async fn set_wifi_enabled(conn: &Connection, enabled: bool) -> Result<()> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;

    proxy.set_property("WirelessEnabled", enabled).await?;
    Ok(())
}

/// Check if WiFi is enabled
pub async fn is_wifi_enabled(conn: &Connection) -> Result<bool> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
    )
    .await?;

    let enabled: bool = proxy.get_property("WirelessEnabled").await?;
    Ok(enabled)
}

/// Get saved WiFi connection profiles and return mapping of SSID -> connection path
pub async fn get_saved_wifi_connections(
    conn: &Connection,
) -> Result<HashMap<String, String>> {
    let settings_proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager/Settings",
        "org.freedesktop.NetworkManager.Settings",
    )
    .await?;

    let reply = settings_proxy
        .call_method("ListConnections", &())
        .await?;
    let conn_paths: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    let mut saved: HashMap<String, String> = HashMap::new();

    for conn_path in conn_paths {
        let conn_proxy = zbus::Proxy::new(
            conn,
            "org.freedesktop.NetworkManager",
            conn_path.as_ref(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await?;

        let reply = match conn_proxy.call_method("GetSettings", &()).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        let settings: HashMap<String, HashMap<String, OwnedValue>> =
            match reply.body().deserialize() {
                Ok(s) => s,
                Err(_) => continue,
            };

        // Check if this is a WiFi connection
        if let Some(connection_section) = settings.get("connection") {
            if let Some(conn_type) = connection_section.get("type") {
                let type_str: &str = match conn_type.downcast_ref() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if type_str != "802-11-wireless" {
                    continue;
                }
            }
        }

        // Get the SSID
        if let Some(wifi_section) = settings.get("802-11-wireless") {
            if let Some(ssid_val) = wifi_section.get("ssid") {
                let ssid_bytes: Vec<u8> = match ssid_val.try_clone()
                    .ok()
                    .and_then(|v| TryInto::<Vec<u8>>::try_into(v).ok())
                {
                    Some(b) => b,
                    None => continue,
                };
                let ssid = String::from_utf8_lossy(&ssid_bytes).to_string();
                if !ssid.is_empty() {
                    saved.insert(ssid, conn_path.to_string());
                }
            }
        }
    }

    Ok(saved)
}

/// Get the active connection path for a given device
pub async fn get_active_connection_path(
    conn: &Connection,
    device_path: &str,
) -> Result<Option<String>> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        device_path,
        "org.freedesktop.NetworkManager.Device",
    )
    .await?;

    let active_conn: OwnedObjectPath = match proxy.get_property("ActiveConnection").await {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    if active_conn.as_str() == "/" {
        Ok(None)
    } else {
        Ok(Some(active_conn.to_string()))
    }
}
