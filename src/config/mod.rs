pub mod keymap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub show_hidden: bool,
    pub confirm_delete: bool,
    pub confirm_overwrite: bool,
    pub editor_tab_size: usize,
    pub viewer_wrap_lines: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            confirm_delete: true,
            confirm_overwrite: true,
            editor_tab_size: 4,
            viewer_wrap_lines: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| ".".into())
            .join("mc-rs")
            .join("config.json");

        if config_path.exists()
            && let Ok(data) = std::fs::read_to_string(&config_path)
            && let Ok(config) = serde_json::from_str(&data)
        {
            return config;
        }
        Self::default()
    }

    pub fn save(&self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| ".".into())
            .join("mc-rs");

        let _ = std::fs::create_dir_all(&config_path);
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path.join("config.json"), data);
        }
    }
}
