use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "wifitui", about = "TUI WiFi manager for Linux")]
#[command(version)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
