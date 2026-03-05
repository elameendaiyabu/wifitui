use std::collections::HashMap;

use zbus::Connection;
use zbus::zvariant::OwnedObjectPath;

use super::access_point::{self, AccessPointInfo};
use super::connection;
use super::error::{NmError, Result};
use super::types::{ConnectionStatus, NmDeviceState, SecurityType};
use super::wireless;

#[derive(Clone, Debug)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: u8,
    pub frequency: u32,
    pub security: SecurityType,
    pub status: ConnectionStatus,
    pub ap_path: String,
    pub saved_connection_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AdapterInfo {
    pub interface: String,
    pub hw_address: String,
    pub state: String,
    pub ip_address: Option<String>,
    pub frequency: Option<String>,
    pub driver: String,
    pub bitrate: Option<String>,
    pub wifi_enabled: bool,
}

impl Default for AdapterInfo {
    fn default() -> Self {
        Self {
            interface: "N/A".to_string(),
            hw_address: "N/A".to_string(),
            state: "Unknown".to_string(),
            ip_address: None,
            frequency: None,
            driver: "N/A".to_string(),
            bitrate: None,
            wifi_enabled: false,
        }
    }
}

pub struct NmManager {
    conn: Connection,
    wifi_device_path: String,
}

impl NmManager {
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        let wifi_device_path = Self::find_wifi_device(&conn).await?;
        Ok(Self {
            conn,
            wifi_device_path,
        })
    }

    async fn find_wifi_device(conn: &Connection) -> Result<String> {
        let proxy = zbus::Proxy::new(
            conn,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
        )
        .await?;

        let reply = proxy.call_method("GetDevices", &()).await?;
        let devices: Vec<OwnedObjectPath> = reply.body().deserialize()?;

        for device_path in devices {
            let dev_proxy = zbus::Proxy::new(
                conn,
                "org.freedesktop.NetworkManager",
                device_path.as_str(),
                "org.freedesktop.NetworkManager.Device",
            )
            .await?;

            let device_type: u32 = match dev_proxy.get_property("DeviceType").await {
                Ok(t) => t,
                Err(_) => continue,
            };

            // DeviceType 2 = WiFi
            if device_type == 2 {
                return Ok(device_path.to_string());
            }
        }

        Err(NmError::NoWifiDevice)
    }

    pub async fn request_scan(&self) -> Result<()> {
        wireless::request_scan(&self.conn, &self.wifi_device_path).await
    }

    pub async fn get_networks(&self) -> Result<Vec<WifiNetwork>> {
        let ap_paths =
            wireless::get_all_access_points(&self.conn, &self.wifi_device_path).await?;

        let active_ap =
            wireless::get_active_access_point(&self.conn, &self.wifi_device_path).await?;

        let saved = connection::get_saved_wifi_connections(&self.conn).await?;

        let mut networks: Vec<WifiNetwork> = Vec::new();
        let mut seen_ssids: HashMap<String, usize> = HashMap::new();

        for ap_path in &ap_paths {
            let info: AccessPointInfo =
                match access_point::fetch_ap_info(&self.conn, ap_path).await? {
                    Some(info) => info,
                    None => continue,
                };

            let is_active = active_ap
                .as_ref()
                .is_some_and(|a| a.as_str() == ap_path.as_str());

            let saved_path = saved.get(&info.ssid).cloned();

            let status = if is_active {
                ConnectionStatus::Connected
            } else if saved_path.is_some() {
                ConnectionStatus::Known
            } else {
                ConnectionStatus::Disconnected
            };

            // De-duplicate by SSID: keep the one with strongest signal
            if let Some(&existing_idx) = seen_ssids.get(&info.ssid) {
                let existing = &networks[existing_idx];
                if info.strength > existing.signal_strength
                    && !matches!(existing.status, ConnectionStatus::Connected)
                {
                    networks[existing_idx] = WifiNetwork {
                        ssid: info.ssid.clone(),
                        bssid: info.bssid,
                        signal_strength: info.strength,
                        frequency: info.frequency,
                        security: info.security,
                        status,
                        ap_path: ap_path.to_string(),
                        saved_connection_path: saved_path,
                    };
                }
            } else {
                seen_ssids.insert(info.ssid.clone(), networks.len());
                networks.push(WifiNetwork {
                    ssid: info.ssid,
                    bssid: info.bssid,
                    signal_strength: info.strength,
                    frequency: info.frequency,
                    security: info.security,
                    status,
                    ap_path: ap_path.to_string(),
                    saved_connection_path: saved_path,
                });
            }
        }

        // Sort: connected first, then known, then by signal strength descending
        networks.sort_by(|a, b| {
            let status_order = |s: &ConnectionStatus| -> u8 {
                match s {
                    ConnectionStatus::Connected => 0,
                    ConnectionStatus::Connecting => 1,
                    ConnectionStatus::Known => 2,
                    ConnectionStatus::Disconnected => 3,
                }
            };

            status_order(&a.status)
                .cmp(&status_order(&b.status))
                .then(b.signal_strength.cmp(&a.signal_strength))
        });

        Ok(networks)
    }

    pub async fn get_adapter_info(&self) -> Result<AdapterInfo> {
        let dev_proxy = zbus::Proxy::new(
            &self.conn,
            "org.freedesktop.NetworkManager",
            self.wifi_device_path.as_str(),
            "org.freedesktop.NetworkManager.Device",
        )
        .await?;

        let interface: String = dev_proxy
            .get_property("Interface")
            .await
            .unwrap_or_else(|_| "N/A".to_string());

        let hw_address: String = dev_proxy
            .get_property("HwAddress")
            .await
            .unwrap_or_else(|_| "N/A".to_string());

        let state_u32: u32 = dev_proxy.get_property("State").await.unwrap_or(0);
        let state = NmDeviceState::from_u32(state_u32);

        let driver: String = dev_proxy
            .get_property("Driver")
            .await
            .unwrap_or_else(|_| "N/A".to_string());

        let wifi_enabled = connection::is_wifi_enabled(&self.conn).await?;

        // Get IP address from IP4Config
        let ip_address = self.get_ip4_address(&dev_proxy).await;

        // Get wireless-specific info
        let wifi_proxy = zbus::Proxy::new(
            &self.conn,
            "org.freedesktop.NetworkManager",
            self.wifi_device_path.as_str(),
            "org.freedesktop.NetworkManager.Device.Wireless",
        )
        .await?;

        let bitrate: Option<String> = wifi_proxy
            .get_property::<u32>("Bitrate")
            .await
            .ok()
            .filter(|&b| b > 0)
            .map(|b| format!("{} Mbit/s", b / 1000));

        // Get frequency from active AP
        let frequency = if let Ok(Some(active_ap)) =
            wireless::get_active_access_point(&self.conn, &self.wifi_device_path).await
        {
            if let Ok(Some(ap_info)) =
                access_point::fetch_ap_info(&self.conn, &active_ap).await
            {
                Some(format!("{} MHz", ap_info.frequency))
            } else {
                None
            }
        } else {
            None
        };

        Ok(AdapterInfo {
            interface,
            hw_address,
            state: state.to_string(),
            ip_address,
            frequency,
            driver,
            bitrate,
            wifi_enabled,
        })
    }

    async fn get_ip4_address(&self, dev_proxy: &zbus::Proxy<'_>) -> Option<String> {
        let ip4_config_path: OwnedObjectPath =
            dev_proxy.get_property("Ip4Config").await.ok()?;

        if ip4_config_path.as_str() == "/" {
            return None;
        }

        let ip4_proxy = zbus::Proxy::new(
            &self.conn,
            "org.freedesktop.NetworkManager",
            ip4_config_path.as_str(),
            "org.freedesktop.NetworkManager.IP4Config",
        )
        .await
        .ok()?;

        // AddressData is aa{sv} - array of dicts
        let address_data: Vec<HashMap<String, zbus::zvariant::OwnedValue>> =
            ip4_proxy.get_property("AddressData").await.ok()?;

        if let Some(first) = address_data.first() {
            if let Some(addr_val) = first.get("address") {
                let addr: &str = addr_val.downcast_ref().ok()?;
                return Some(addr.to_string());
            }
        }

        None
    }

    pub async fn connect_to_network(
        &self,
        ssid: &str,
        password: &str,
        security: &SecurityType,
        ap_path: &str,
    ) -> Result<()> {
        connection::connect_with_password(
            &self.conn,
            &self.wifi_device_path,
            ap_path,
            ssid,
            password,
            security,
        )
        .await
    }

    pub async fn activate_saved_connection(
        &self,
        connection_path: &str,
        ap_path: &str,
    ) -> Result<()> {
        connection::activate_connection(
            &self.conn,
            connection_path,
            &self.wifi_device_path,
            ap_path,
        )
        .await
    }

    pub async fn disconnect(&self) -> Result<()> {
        let active_path =
            connection::get_active_connection_path(&self.conn, &self.wifi_device_path)
                .await?;

        match active_path {
            Some(path) => connection::disconnect(&self.conn, &path).await,
            None => Err(NmError::NoActiveConnection),
        }
    }

    pub async fn forget_connection(&self, connection_path: &str) -> Result<()> {
        connection::forget_connection(&self.conn, connection_path).await
    }

    pub async fn toggle_wifi(&self) -> Result<bool> {
        let current = connection::is_wifi_enabled(&self.conn).await?;
        let new_state = !current;
        connection::set_wifi_enabled(&self.conn, new_state).await?;
        Ok(new_state)
    }
}
