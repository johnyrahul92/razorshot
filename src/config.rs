use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub save_dir: String,
    pub filename_template: String,
    #[serde(default = "default_export_format")]
    pub export_format: String,
    pub annotation: AnnotationConfig,
    pub behavior: BehaviorConfig,
}

fn default_export_format() -> String {
    "png".into()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationConfig {
    pub default_color: String,
    pub line_width: f64,
    pub font_size: f64,
    pub blur_block_size: u32,
    #[serde(default = "default_jpeg_quality")]
    pub jpeg_quality: u8,
}

fn default_jpeg_quality() -> u8 {
    90
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorConfig {
    pub open_editor: bool,
    pub copy_to_clipboard: bool,
    pub show_notification: bool,
    pub default_action: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: "~/Pictures/Screenshots".into(),
            filename_template: "Screenshot_%Y-%m-%d_%H-%M-%S".into(),
            export_format: "png".into(),
            annotation: AnnotationConfig {
                default_color: "#ff0000".into(),
                line_width: 3.0,
                font_size: 16.0,
                blur_block_size: 10,
                jpeg_quality: 90,
            },
            behavior: BehaviorConfig {
                open_editor: true,
                copy_to_clipboard: true,
                show_notification: true,
                default_action: "tray".into(),
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("razorshot");
        config_dir.join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        log::warn!("Failed to parse config, using defaults: {}", e);
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read config file, using defaults: {}", e);
                }
            }
        }
        let config = Config::default();
        config.save();
        config
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(e) = fs::write(&path, contents) {
                    log::error!("Failed to write config: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize config: {}", e);
            }
        }
    }

    /// Expand ~ to home directory in save_dir
    pub fn resolve_save_dir(&self) -> PathBuf {
        let expanded = if self.save_dir.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&self.save_dir[2..])
            } else {
                PathBuf::from(&self.save_dir)
            }
        } else {
            PathBuf::from(&self.save_dir)
        };
        expanded
    }
}
