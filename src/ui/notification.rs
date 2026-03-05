use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::app::{App, NotifLevel};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    if app.notifications.is_empty() {
        return;
    }

    let mut y_offset = area.y + area.height.saturating_sub(4);

    for notif in app.notifications.iter().rev().take(3) {
        let (fg, bg) = match notif.level {
            NotifLevel::Info => (ratatui::style::Color::White, ratatui::style::Color::Blue),
            NotifLevel::Success => {
                (ratatui::style::Color::White, ratatui::style::Color::Green)
            }
            NotifLevel::Warning => {
                (ratatui::style::Color::Black, ratatui::style::Color::Yellow)
            }
            NotifLevel::Error => (ratatui::style::Color::White, ratatui::style::Color::Red),
        };

        let msg = format!(" {} ", notif.message);
        let width = (msg.len() as u16 + 2).min(area.width / 2);
        let notif_area = Rect::new(
            area.x + area.width.saturating_sub(width + 1),
            y_offset,
            width,
            1,
        );

        if notif_area.y >= area.y && notif_area.y < area.y + area.height {
            frame.render_widget(Clear, notif_area);
            let widget = Paragraph::new(msg).style(Style::default().fg(fg).bg(bg));
            frame.render_widget(widget, notif_area);
        }

        y_offset = y_offset.saturating_sub(2);
    }
}
