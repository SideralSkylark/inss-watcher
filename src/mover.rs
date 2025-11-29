use std::fs;
use std::path::Path;
use anyhow::Result;
use log::info;

pub fn move_file(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    info!("Moving file from {} to {}", src.to_string_lossy(), dest.to_string_lossy());
    fs::rename(src, dest)?;
    Ok(())
}
