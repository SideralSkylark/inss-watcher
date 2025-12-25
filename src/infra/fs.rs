use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

pub fn move_if_missing(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if dest.exists() {
        return Ok(()); 
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn move_if_missing_moves_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dest = dir.path().join("b.txt");

        fs::write(&src, "test").unwrap();
        move_if_missing(&src, &dest).unwrap();

        assert!(!src.exists());
        assert!(dest.exists());
    }

    #[test]
    fn move_if_missing_is_idempotent() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dest = dir.path().join("b.txt");

        fs::write(&src, "test").unwrap();
        fs::write(&dest, "existing").unwrap();

        move_if_missing(&src, &dest).unwrap();

        assert!(src.exists());
    }
}
