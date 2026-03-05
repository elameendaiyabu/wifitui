use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;

pub async fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Password modal takes priority
    if app.password_modal.visible {
        handle_password_input(app, key).await;
        return;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_previous();
        }
        KeyCode::Char('g') => {
            app.select_first();
        }
        KeyCode::Char('G') => {
            app.select_last();
        }

        // Actions
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.connect_selected().await;
        }
        KeyCode::Char('d') => {
            app.disconnect_current().await;
        }
        KeyCode::Char('f') => {
            app.forget_selected().await;
        }
        KeyCode::Char('s') => {
            app.start_scan().await;
        }
        KeyCode::Char('w') => {
            app.toggle_wifi().await;
        }
        KeyCode::Char('r') => {
            app.refresh().await;
        }

        // Panel switching
        KeyCode::Tab => {
            app.toggle_focus();
        }

        _ => {}
    }
}

async fn handle_password_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let password = app.password_modal.input.value().to_string();
            if password.is_empty() {
                app.password_modal.error_message =
                    Some("Password cannot be empty".to_string());
                return;
            }
            app.password_modal.visible = false;
            app.connect_with_password(password).await;
        }
        KeyCode::Esc => {
            app.password_modal.visible = false;
            app.password_modal.input = tui_input::Input::default();
            app.password_modal.error_message = None;
        }
        KeyCode::Tab => {
            app.password_modal.show_password = !app.password_modal.show_password;
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.password_modal.input = tui_input::Input::default();
        }
        _ => {
            // Forward to tui-input for standard text editing
            app.password_modal
                .input
                .handle_event(&crossterm::event::Event::Key(key));
        }
    }
}
