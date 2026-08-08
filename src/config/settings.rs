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
    pub storage: StorageSettings,
    pub logs: LogsSettings,
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
            storage: StorageSettings::default(),
            logs: LogsSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageSettings {
    pub output_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LogsSettings {
    pub output_path: PathBuf,
}

impl Default for LogsSettings {
    fn default() -> Self {
        let output_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inss-watcher/logs");
        Self { output_path }
    }
}

impl Default for StorageSettings {
    fn default() -> Self {
        let output_path = dirs::document_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("INSS");
        Self { output_path }
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

        let settings: Settings =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("invalid config file: {}", e))?;

        Ok(settings)
    }

    /// ~/.config/inss-watcher/config.toml
    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inss-watcher/config.toml")
    }

    pub fn esure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.logs.output_path)?;
        std::fs::create_dir_all(&self.storage.output_path)?;
        std::fs::create_dir_all(&self.quarantine.quarantine_path)?;

        if let Some(parent) = self.db.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if let Some(parent) = self.daemon.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.processing.worker_threads == 0 {
            anyhow::bail!("no worker thread config");
        }
        if self.processing.stable_delay_ms == 0 {
            anyhow::bail!("no stable delay config");
        }
        if self.processing.stable_checks == 0 {
            anyhow::bail!("no stable check config");
        }
        if self.watcher.dirs_to_watch.is_empty() {
            anyhow::bail!("no dirs to watch");
        }
        if !self.quarantine.quarantine_path.as_path().is_absolute()
            || self
                .quarantine
                .quarantine_path
                .as_path()
                .as_os_str()
                .is_empty()
        {
            anyhow::bail!("not a valid quarantine path");
        }
        if !self.storage.output_path.as_path().is_absolute()
            || self.storage.output_path.as_path().as_os_str().is_empty()
        {
            anyhow::bail!("not a valid storage path");
        }
        if !self.daemon.socket_path.as_path().is_absolute()
            || self.daemon.socket_path.as_path().as_os_str().is_empty()
        {
            anyhow::bail!("not a valid socket path");
        }
        if !self.logs.output_path.as_path().is_absolute()
            || self.logs.output_path.as_path().as_os_str().is_empty()
        {
            anyhow::bail!("not a valid log output path");
        }
        if !self.db.path.as_path().is_absolute() || self.db.path.as_path().as_os_str().is_empty() {
            anyhow::bail!("not a valid db path");
        }

        Ok(())
    }
}

impl Default for WatcherSettings {
    fn default() -> Self {
        let default_dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            dirs_to_watch: vec![default_dir],
        }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inss-watcher/inss.db");
        Self { path }
    }
}

impl Default for ProcessingSettings {
    fn default() -> Self {
        Self {
            stable_checks: 6,
            stable_delay_ms: 400,
            worker_threads: 1,
        }
    }
}

impl Default for DaemonSettings {
    fn default() -> Self {
        let socket_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inss-watcher/inss-watcher.sock");
        Self { socket_path }
    }
}

impl Default for QuarantineSettings {
    fn default() -> Self {
        let quarantine_path = dirs::document_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("INSS/quarantine");
        Self { quarantine_path }
    }
}
