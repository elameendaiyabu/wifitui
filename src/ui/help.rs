use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

use super::styles::*;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let keybindings: Vec<(&str, &str)> = if app.password_modal.visible {
        vec![
            ("Enter", "Connect"),
            ("Esc", "Cancel"),
            ("Tab", "Show/Hide"),
            ("Ctrl+u", "Clear"),
        ]
    } else {
        vec![
            ("Enter", "Connect"),
            ("d", "Disconnect"),
            ("f", "Forget"),
            ("s", "Scan"),
            ("w", "WiFi on/off"),
            ("j/↓", "Down"),
            ("k/↑", "Up"),
            ("q", "Quit"),
        ]
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, action)) in keybindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(TEXT_SECONDARY)));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(KEY_COLOR)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*action, Style::default().fg(TEXT_SECONDARY)));
    }

    let help_line = Line::from(spans);
    let paragraph = Paragraph::new(help_line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
