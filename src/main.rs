#![allow(dead_code)]

mod app;
mod cli;
mod config;
mod event;
mod handler;
mod nm;
mod tui;
mod ui;

use std::time::Duration;

use clap::Parser;

use app::App;
use cli::Cli;
use config::AppConfig;
use event::Event;
use nm::NmManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = AppConfig::load(cli.config.as_ref());

    // Initialize NetworkManager connection
    let nm = match NmManager::new().await {
        Ok(nm) => nm,
        Err(e) => {
            eprintln!("Failed to connect to NetworkManager: {e}");
            eprintln!("Make sure NetworkManager is running: systemctl status NetworkManager");
            std::process::exit(1);
        }
    };

    // Initialize app
    let mut app = App::new(nm).await;

    // Initial scan and data load
    let _ = app.nm.request_scan().await;
    app.refresh().await;

    // Initialize terminal
    let mut terminal = tui::Tui::new()?;
    terminal.enter()?;

    // Event handler
    let tick_rate = Duration::from_millis(config.general.tick_rate);
    let mut events = event::EventHandler::new(tick_rate);

    // Main loop
    while app.running {
        terminal.draw(&mut app)?;

        match events.next().await {
            Some(Event::Tick) => {
                app.tick().await;
            }
            Some(Event::Key(key_event)) => {
                handler::handle_key_event(&mut app, key_event).await;
            }
            Some(Event::Resize(_, _)) => {
                // ratatui handles resize automatically on next draw
            }
            None => {
                app.running = false;
            }
        }
    }

    terminal.exit()?;
    Ok(())
}
