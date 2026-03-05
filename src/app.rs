use std::time::Instant;

use ratatui::widgets::TableState;
use tui_input::Input;

use crate::nm::{AdapterInfo, ConnectionStatus, NmManager, SecurityType, WifiNetwork};

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub level: NotifLevel,
    pub created_at: Instant,
}

#[derive(Clone, Debug)]
pub enum NotifLevel {
    Info,
    Success,
    Warning,
    Error,
}

pub struct PasswordModal {
    pub visible: bool,
    pub input: Input,
    pub target_ssid: String,
    pub target_ap_path: String,
    pub target_security: SecurityType,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl Default for PasswordModal {
    fn default() -> Self {
        Self {
            visible: false,
            input: Input::default(),
            target_ssid: String::new(),
            target_ap_path: String::new(),
            target_security: SecurityType::Open,
            show_password: false,
            error_message: None,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum FocusedPanel {
    Networks,
    Adapter,
}

pub struct App {
    pub running: bool,
    pub nm: NmManager,

    // Data
    pub networks: Vec<WifiNetwork>,
    pub adapter_info: AdapterInfo,

    // UI state
    pub focused_panel: FocusedPanel,
    pub network_table_state: TableState,
    pub is_scanning: bool,
    pub scan_tick: usize,

    // Password modal
    pub password_modal: PasswordModal,

    // Notifications
    pub notifications: Vec<Notification>,

    // Tick counters for auto-refresh
    tick_count: usize,
}

impl App {
    pub async fn new(nm: NmManager) -> Self {
        let mut app = Self {
            running: true,
            nm,
            networks: Vec::new(),
            adapter_info: AdapterInfo::default(),
            focused_panel: FocusedPanel::Networks,
            network_table_state: TableState::default(),
            is_scanning: false,
            scan_tick: 0,
            password_modal: PasswordModal::default(),
            notifications: Vec::new(),
            tick_count: 0,
        };

        // Select first row
        app.network_table_state.select(Some(0));
        app
    }

    pub async fn refresh(&mut self) {
        match self.nm.get_networks().await {
            Ok(networks) => {
                let had_selection = self.network_table_state.selected();
                self.networks = networks;

                // Preserve selection or clamp it
                if !self.networks.is_empty() {
                    let idx = had_selection
                        .unwrap_or(0)
                        .min(self.networks.len().saturating_sub(1));
                    self.network_table_state.select(Some(idx));
                } else {
                    self.network_table_state.select(None);
                }
            }
            Err(e) => {
                self.add_notification(&format!("Refresh failed: {e}"), NotifLevel::Error);
            }
        }

        match self.nm.get_adapter_info().await {
            Ok(info) => self.adapter_info = info,
            Err(e) => {
                self.add_notification(&format!("Adapter info failed: {e}"), NotifLevel::Error);
            }
        }
    }

    pub async fn tick(&mut self) {
        self.tick_count += 1;

        // Advance spinner
        if self.is_scanning {
            self.scan_tick += 1;
        }

        // Expire old notifications (5 seconds)
        self.notifications
            .retain(|n| n.created_at.elapsed().as_secs() < 5);

        // Auto-refresh every 10 seconds (40 ticks at 250ms)
        if self.tick_count % 40 == 0 {
            self.refresh().await;
        }

        // Auto-scan every 30 seconds (120 ticks)
        if self.tick_count % 120 == 0 {
            let _ = self.nm.request_scan().await;
        }

        // Stop scanning animation after 5 seconds (20 ticks)
        if self.is_scanning && self.scan_tick >= 20 {
            self.is_scanning = false;
            self.scan_tick = 0;
        }
    }

    pub fn select_next(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let current = self.network_table_state.selected().unwrap_or(0);
        let next = if current >= self.networks.len() - 1 {
            current
        } else {
            current + 1
        };
        self.network_table_state.select(Some(next));
    }

    pub fn select_previous(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let current = self.network_table_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.network_table_state.select(Some(prev));
    }

    pub fn select_first(&mut self) {
        if !self.networks.is_empty() {
            self.network_table_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.networks.is_empty() {
            self.network_table_state
                .select(Some(self.networks.len() - 1));
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Networks => FocusedPanel::Adapter,
            FocusedPanel::Adapter => FocusedPanel::Networks,
        };
    }

    pub fn selected_network(&self) -> Option<&WifiNetwork> {
        let idx = self.network_table_state.selected()?;
        self.networks.get(idx)
    }

    pub async fn connect_selected(&mut self) {
        let network = match self.selected_network() {
            Some(n) => n.clone(),
            None => return,
        };

        match network.status {
            ConnectionStatus::Connected => {
                self.add_notification("Already connected", NotifLevel::Info);
            }
            ConnectionStatus::Known => {
                if let Some(ref conn_path) = network.saved_connection_path {
                    match self
                        .nm
                        .activate_saved_connection(conn_path, &network.ap_path)
                        .await
                    {
                        Ok(()) => {
                            self.add_notification("Connecting...", NotifLevel::Info);
                        }
                        Err(e) => {
                            self.add_notification(
                                &format!("Connection failed: {e}"),
                                NotifLevel::Error,
                            );
                        }
                    }
                }
            }
            _ => {
                if network.security == SecurityType::Open {
                    match self
                        .nm
                        .connect_to_network(
                            &network.ssid,
                            "",
                            &network.security,
                            &network.ap_path,
                        )
                        .await
                    {
                        Ok(()) => {
                            self.add_notification("Connecting...", NotifLevel::Info);
                        }
                        Err(e) => {
                            self.add_notification(
                                &format!("Connection failed: {e}"),
                                NotifLevel::Error,
                            );
                        }
                    }
                } else {
                    // Show password modal
                    self.password_modal = PasswordModal {
                        visible: true,
                        input: Input::default(),
                        target_ssid: network.ssid.clone(),
                        target_ap_path: network.ap_path.clone(),
                        target_security: network.security.clone(),
                        show_password: false,
                        error_message: None,
                    };
                }
            }
        }
    }

    pub async fn connect_with_password(&mut self, password: String) {
        let ssid = self.password_modal.target_ssid.clone();
        let ap_path = self.password_modal.target_ap_path.clone();
        let security = self.password_modal.target_security.clone();

        match self
            .nm
            .connect_to_network(&ssid, &password, &security, &ap_path)
            .await
        {
            Ok(()) => {
                self.add_notification(
                    &format!("Connecting to {}...", ssid),
                    NotifLevel::Info,
                );
                // Refresh after a short delay to pick up the new state
                self.refresh().await;
            }
            Err(e) => {
                self.add_notification(
                    &format!("Connection failed: {e}"),
                    NotifLevel::Error,
                );
            }
        }
    }

    pub async fn disconnect_current(&mut self) {
        match self.nm.disconnect().await {
            Ok(()) => {
                self.add_notification("Disconnected", NotifLevel::Success);
                self.refresh().await;
            }
            Err(e) => {
                self.add_notification(&format!("Disconnect failed: {e}"), NotifLevel::Error);
            }
        }
    }

    pub async fn forget_selected(&mut self) {
        let network = match self.selected_network() {
            Some(n) => n.clone(),
            None => return,
        };

        if let Some(ref conn_path) = network.saved_connection_path {
            match self.nm.forget_connection(conn_path).await {
                Ok(()) => {
                    self.add_notification(
                        &format!("Forgot {}", network.ssid),
                        NotifLevel::Success,
                    );
                    self.refresh().await;
                }
                Err(e) => {
                    self.add_notification(
                        &format!("Forget failed: {e}"),
                        NotifLevel::Error,
                    );
                }
            }
        } else {
            self.add_notification("Network is not saved", NotifLevel::Warning);
        }
    }

    pub async fn start_scan(&mut self) {
        self.is_scanning = true;
        self.scan_tick = 0;
        match self.nm.request_scan().await {
            Ok(()) => {
                self.add_notification("Scanning...", NotifLevel::Info);
            }
            Err(e) => {
                self.add_notification(&format!("Scan failed: {e}"), NotifLevel::Error);
                self.is_scanning = false;
            }
        }
    }

    pub async fn toggle_wifi(&mut self) {
        match self.nm.toggle_wifi().await {
            Ok(enabled) => {
                let msg = if enabled {
                    "WiFi enabled"
                } else {
                    "WiFi disabled"
                };
                self.add_notification(msg, NotifLevel::Success);
                self.refresh().await;
            }
            Err(e) => {
                self.add_notification(
                    &format!("WiFi toggle failed: {e}"),
                    NotifLevel::Error,
                );
            }
        }
    }

    pub fn add_notification(&mut self, message: &str, level: NotifLevel) {
        self.notifications.push(Notification {
            message: message.to_string(),
            level,
            created_at: Instant::now(),
        });
    }
}
