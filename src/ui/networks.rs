use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::app::{App, FocusedPanel};
use crate::nm::ConnectionStatus;

use super::spinner;
use super::styles::*;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let header = Row::new(vec!["", "SSID", "Signal", "Security", "Status"])
        .style(
            Style::default()
                .fg(HEADER_COLOR)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .networks
        .iter()
        .map(|net| {
            let icon = signal_icon(net.signal_strength);
            let signal_text = format!("{}%", net.signal_strength);
            let security_text = net.security.to_string();
            let status_text = net.status.to_string();

            let style = match net.status {
                ConnectionStatus::Connected => Style::default().fg(STATUS_CONNECTED),
                ConnectionStatus::Known => Style::default().fg(STATUS_KNOWN),
                _ => Style::default().fg(TEXT_PRIMARY),
            };

            Row::new(vec![
                icon.to_string(),
                net.ssid.clone(),
                signal_text,
                security_text,
                status_text,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(2),  // Icon
        Constraint::Min(20),   // SSID
        Constraint::Length(8),  // Signal
        Constraint::Length(12), // Security
        Constraint::Length(14), // Status
    ];

    let title = if app.is_scanning {
        format!(" Networks {} ", spinner::frame(app.scan_tick))
    } else {
        format!(" Networks ({}) ", app.networks.len())
    };

    let border_color = if app.focused_panel == FocusedPanel::Networks {
        BORDER_FOCUSED
    } else {
        BORDER_NORMAL
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        )
        .row_highlight_style(
            Style::default()
                .bg(HIGHLIGHT_BG)
                .fg(HIGHLIGHT_FG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    frame.render_stateful_widget(table, area, &mut app.network_table_state);
}

fn signal_icon(strength: u8) -> &'static str {
    match strength {
        0..=19 => "󰤯",
        20..=39 => "󰤟",
        40..=59 => "󰤢",
        60..=79 => "󰤥",
        80..=100 => "󰤨",
        _ => "󰤯",
    }
}
