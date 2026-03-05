mod adapter;
mod help;
mod networks;
mod notification;
mod password;
mod spinner;
mod styles;

use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Top-level vertical split:
    //   [0] Networks panel  -- takes remaining space
    //   [1] Adapter panel   -- 4 lines
    //   [2] Help bar        -- 1 line
    let main_layout = Layout::vertical([
        Constraint::Min(8),
        Constraint::Length(5),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .split(area);

    // Render each section
    networks::render(app, frame, main_layout[0]);
    adapter::render(app, frame, main_layout[1]);
    help::render(app, frame, main_layout[2]);
    help::render_bottom( frame, main_layout[3]);

    // Modal overlays (rendered last, on top)
    if app.password_modal.visible {
        password::render(app, frame, area);
    }

    // Notifications (bottom-right corner)
    notification::render(app, frame, area);
}
