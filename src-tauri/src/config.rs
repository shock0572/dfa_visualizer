use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const ALL_CATEGORIES: &[&str] = &[
    "Completion Score",
    "Gameplay Score",
    "Completion Count",
    "Achievement Points",
    "Mounts",
    "Pet Score",
    "Titles",
    "Reputations",
    "Recipes",
    "Quests",
    "Toys",
    "Appearance Sources",
    "Heirloom Score",
    "Decor",
    "Achievements",
    "Feats of Strength",
    "Legacy Achievements",
    "Pets",
    "Appearances",
    "Heirlooms",
    "Alts",
    "Alt Score",
    "Honor Level",
    "Honorable Kills",
];

const DEFAULT_TRACKED: &[&str] = &[
    "Achievement Points",
    "Mounts",
    "Pet Score",
    "Titles",
    "Reputations",
    "Recipes",
    "Quests",
    "Toys",
    "Appearance Sources",
    "Heirloom Score",
    "Decor",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub region: String,
    pub realm: String,
    pub character: String,
    pub tracked_rankings: Vec<String>,
    pub refresh_interval_minutes: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            region: "EU".into(),
            realm: String::new(),
            character: String::new(),
            tracked_rankings: DEFAULT_TRACKED.iter().map(|s| s.to_string()).collect(),
            refresh_interval_minutes: 30,
        }
    }
}

impl AppConfig {
    pub fn is_configured(&self) -> bool {
        !self.realm.is_empty() && !self.character.is_empty()
    }
}

fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("dfa_visualizer")
}

fn config_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<AppConfig>(&contents) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Failed to parse config: {e}"),
            },
            Err(e) => eprintln!("Failed to read config: {e}"),
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {e}"))?;

    let json =
        serde_json::to_string_pretty(cfg).map_err(|e| format!("Failed to serialize config: {e}"))?;

    fs::write(config_path(), json).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}
