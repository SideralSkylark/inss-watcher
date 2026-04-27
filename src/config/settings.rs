use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub watcher: WatcherSettings,
    pub db: DatabaseSettings,
    pub processing: ProcessingSettings,
    pub daemon: DaemonSettings,
    pub quarantine: QuarantineSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WatcherSettings {
    pub dirs_to_watch: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseSettings {
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessingSettings {
    pub stable_checks: usize,
    pub stable_delay_ms: u64,
    pub worker_threads: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonSettings {
    pub socket_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct QuarantineSettings {
    pub quarantine_path: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self { 
            watcher: WatcherSettings::default(),
            db: DatabaseSettings::default(),
            processing: ProcessingSettings::default(),
            daemon: DaemonSettings::default(),
            quarantine: QuarantineSettings::default(),
        }
    }
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::default_config_path();

        if !config_path.exists() {
            let defaults = Settings::default();
            let content = toml::to_string_pretty(&defaults)?;
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&config_path, content)?;
            return Ok(defaults);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| anyhow::anyhow!("failed to read config {:?}: {}", config_path, e))?;

        let settings: Settings = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("invalid config file: {}", e))?;

        Ok(settings)
    }

    /// ~/.config/inss-watcher/config.toml
    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inss-watcher/config.toml")
    }
}

impl Default for WatcherSettings {
    fn default() -> Self {
        let default_dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        Self { dirs_to_watch: vec![default_dir] }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self { path: PathBuf::from("inss.db") }
    }
}

impl Default for ProcessingSettings {
    fn default() -> Self {
        Self { 
            stable_checks: 6, 
            stable_delay_ms: 400,
            worker_threads: 4,
        }
    }
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self { socket_path: PathBuf::from("/tmp/inss-watcher.sock") }
    }
}

impl Default for QuarantineSettings {
    fn default() -> Self {
        let path = dirs::document_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("INSS/quarantine");
        Self { quarantine_path: path }
    }
}

