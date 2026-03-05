use std::collections::HashMap;

use zbus::Connection;
use zbus::zvariant::{OwnedObjectPath, Value};

use super::error::{NmError, Result};

pub async fn request_scan(conn: &Connection, device_path: &str) -> Result<()> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        device_path,
        "org.freedesktop.NetworkManager.Device.Wireless",
    )
    .await?;

    let options: HashMap<&str, Value<'_>> = HashMap::new();
    match proxy.call_method("RequestScan", &(options,)).await {
        Ok(_) => Ok(()),
        Err(zbus::Error::MethodError(ref name, _, _))
            if name.as_str().contains("ScanningNotAllowed") =>
        {
            Ok(())
        }
        Err(e) => Err(NmError::Dbus(e)),
    }
}

pub async fn get_all_access_points(
    conn: &Connection,
    device_path: &str,
) -> Result<Vec<OwnedObjectPath>> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        device_path,
        "org.freedesktop.NetworkManager.Device.Wireless",
    )
    .await?;

    let reply = proxy.call_method("GetAllAccessPoints", &()).await?;
    let paths: Vec<OwnedObjectPath> = reply.body().deserialize()?;
    Ok(paths)
}

pub async fn get_active_access_point(
    conn: &Connection,
    device_path: &str,
) -> Result<Option<OwnedObjectPath>> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        device_path,
        "org.freedesktop.NetworkManager.Device.Wireless",
    )
    .await?;

    let path: OwnedObjectPath = proxy.get_property("ActiveAccessPoint").await?;
    if path.as_str() == "/" {
        Ok(None)
    } else {
        Ok(Some(path))
    }
}
