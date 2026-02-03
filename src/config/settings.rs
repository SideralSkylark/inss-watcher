use std::path::PathBuf;

pub struct Settings {
    pub dirs_to_watch: Vec<PathBuf>
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let default_dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| anyhow::anyhow!("No home directory found"))?;

        Ok(Self {
            dirs_to_watch: vec![default_dir],
        })
    }
}

