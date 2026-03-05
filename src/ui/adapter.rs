use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, FocusedPanel};

use super::styles::*;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let info = &app.adapter_info;

    let row1 = Line::from(vec![
        Span::styled("Interface: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(&info.interface, Style::default().fg(TEXT_PRIMARY)),
        Span::raw("  "),
        Span::styled("MAC: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(&info.hw_address, Style::default().fg(TEXT_PRIMARY)),
        Span::raw("  "),
        Span::styled("State: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            &info.state,
            Style::default().fg(if info.state == "Connected" {
                STATUS_CONNECTED
            } else {
                STATUS_ERROR
            }),
        ),
    ]);

    let ip = info.ip_address.as_deref().unwrap_or("N/A");
    let freq = info.frequency.as_deref().unwrap_or("N/A");
    let rate = info.bitrate.as_deref().unwrap_or("N/A");

    let row2 = Line::from(vec![
        Span::styled("IP: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(ip, Style::default().fg(TEXT_PRIMARY)),
        Span::raw("  "),
        Span::styled("Freq: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(freq, Style::default().fg(TEXT_PRIMARY)),
        Span::raw("  "),
        Span::styled("Rate: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(rate, Style::default().fg(TEXT_PRIMARY)),
        Span::raw("  "),
        Span::styled("WiFi: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            if info.wifi_enabled { "On" } else { "Off" },
            Style::default().fg(if info.wifi_enabled {
                STATUS_CONNECTED
            } else {
                STATUS_ERROR
            }),
        ),
    ]);

    let border_color = if app.focused_panel == FocusedPanel::Adapter {
        BORDER_FOCUSED
    } else {
        BORDER_NORMAL
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Adapter ");

    let paragraph = Paragraph::new(vec![row1, row2]).block(block);
    frame.render_widget(paragraph, area);
}
