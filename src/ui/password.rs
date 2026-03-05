use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

use super::styles::*;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let modal = &app.password_modal;

    // Center the modal: 55 chars wide, 8 lines tall
    let modal_area = centered_rect(55, 8, area);

    // Clear the background behind the modal
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MODAL_BORDER))
        .title(format!(" Connect to: {} ", modal.target_ssid))
        .title_style(
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Layout inside the modal
    let chunks = Layout::vertical([
        Constraint::Length(1), // Security label
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Input field
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Error message or hint
    ])
    .split(inner);

    // Security info and hint
    let label = Line::from(vec![
        Span::styled("Security: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            modal.target_security.to_string(),
            Style::default().fg(KEY_COLOR),
        ),
        Span::raw("  │  "),
        Span::styled("Tab", Style::default().fg(KEY_COLOR)),
        Span::styled(
            if modal.show_password {
                " to hide"
            } else {
                " to show"
            },
            Style::default().fg(TEXT_SECONDARY),
        ),
    ]);
    frame.render_widget(Paragraph::new(label), chunks[0]);

    // Password input field
    let display_value = if modal.show_password {
        modal.input.value().to_string()
    } else {
        "●".repeat(modal.input.value().len())
    };

    let input_line = Line::from(vec![
        Span::styled("Password: ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(&display_value, Style::default().fg(TEXT_PRIMARY)),
        Span::styled("█", Style::default().fg(KEY_COLOR)), // cursor
    ]);

    let input_widget = Paragraph::new(input_line);
    frame.render_widget(input_widget, chunks[2]);

    // Error message
    if let Some(ref err) = modal.error_message {
        let err_widget =
            Paragraph::new(err.as_str()).style(Style::default().fg(STATUS_ERROR));
        frame.render_widget(err_widget, chunks[4]);
    } else {
        let hint = Paragraph::new("Press Enter to connect, Esc to cancel")
            .style(Style::default().fg(TEXT_SECONDARY));
        frame.render_widget(hint, chunks[4]);
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(
        x,
        y,
        width.min(area.width),
        height.min(area.height),
    )
}
