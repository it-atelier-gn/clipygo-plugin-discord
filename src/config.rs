use serde::{Deserialize, Serialize};

/// Stored at:
///   Windows : %APPDATA%\clipygo-plugin-discord\config.json
///   macOS   : ~/Library/Application Support/clipygo-plugin-discord/config.json
///   Linux   : ~/.config/clipygo-plugin-discord/config.json
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub bot_token: String,
}

pub fn config_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("clipygo-plugin-discord");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("config.json")
}

pub fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) {
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path(), data);
    }
}
