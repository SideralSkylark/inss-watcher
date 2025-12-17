use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

pub fn move_if_missing(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if dest.exists() {
        return Ok(()); // already processed
    }
    fs::rename(src, dest)?;
    Ok(())
}

pub fn inss_output_dir(month: u32, year: u32) -> PathBuf {
    let mut base = dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap();

    base.push("INSS");
    base.push(year.to_string());
    base.push(format!("{:02}", month));
    base
}
