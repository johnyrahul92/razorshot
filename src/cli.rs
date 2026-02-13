use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "razorshot", about = "Wayland screenshot & annotation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Capture a region of the screen
    Region {
        /// Skip annotation editor, save immediately
        #[arg(long)]
        no_edit: bool,
    },
    /// Capture the full screen
    Full {
        /// Skip annotation editor, save immediately
        #[arg(long)]
        no_edit: bool,
        /// Capture a specific monitor (0-indexed)
        #[arg(long)]
        monitor: Option<u32>,
    },
    /// Start in system tray mode
    Tray,
    /// View or modify configuration
    Config {
        /// Print current configuration
        #[arg(long)]
        show: bool,
        /// Set the save directory
        #[arg(long)]
        save_dir: Option<String>,
    },
}
