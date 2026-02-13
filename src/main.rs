mod annotate;
mod app;
mod capture;
mod cli;
mod config;
mod output;
mod tray;
mod ui;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let action = match cli.command {
        Some(Command::Full { no_edit, monitor }) => {
            if no_edit {
                app::AppAction::FullNoEdit { monitor }
            } else {
                app::AppAction::FullEdit { monitor }
            }
        }
        Some(Command::Region { no_edit }) => {
            if no_edit {
                app::AppAction::RegionNoEdit
            } else {
                app::AppAction::RegionEdit
            }
        }
        Some(Command::Tray) => app::AppAction::Tray,
        Some(Command::Config { show, save_dir }) => {
            if show {
                let config = Config::load();
                println!(
                    "{}",
                    toml::to_string_pretty(&config).unwrap_or_else(|_| "Error".into())
                );
                return;
            }
            if let Some(dir) = save_dir {
                let mut config = Config::load();
                config.save_dir = dir;
                config.save();
                println!("Save directory updated. Current config:");
                println!(
                    "{}",
                    toml::to_string_pretty(&config).unwrap_or_else(|_| "Error".into())
                );
                return;
            }
            // No flags → show config
            let config = Config::load();
            println!(
                "{}",
                toml::to_string_pretty(&config).unwrap_or_else(|_| "Error".into())
            );
            return;
        }
        None => {
            // Default action from config
            let config = Config::load();
            match config.behavior.default_action.as_str() {
                "region" => app::AppAction::RegionEdit,
                "full" => app::AppAction::FullEdit { monitor: None },
                _ => app::AppAction::Tray,
            }
        }
    };

    app::run(action);
}
