use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Ok;

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

pub fn move_unique(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    
    let parent = dest.parent().unwrap_or_else(|| Path::new(""));
    let file_stem = dest.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let extension = dest.extension().and_then(|s| s.to_str());

    let mut dest_path = dest.to_path_buf();
    let mut counter = 1;

    while dest_path.exists() {
        let new_name = match extension {
            Some(ext) => format!("{}({}).{}", file_stem, counter, ext),
            None => format!("{}({})", file_stem, counter),
        };

        dest_path = parent.join(new_name);
        counter += 1;
    }

    fs::rename(src, &dest_path)?;
    Ok(())
}

pub fn inss_output_dir(month: u32, year: u32, contributor_num: &str) -> PathBuf {
    let mut base = dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap();

    base.push("INSS");
    base.push(year.to_string());
    base.push(format!("{:02}", month));
    base.push(format!("contributor_{contributor_num}"));
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn move_unique_and_no_conflict_moves_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dest = dir.path().join("b.txt");

        fs::write(&src, "test").unwrap();
        move_unique(&src, &dest).unwrap();

        assert!(!src.exists());
        assert!(dest.exists());
    }

    #[test]
    fn move_unique_conflict() {
        let dir = tempdir().unwrap();
        let src1 = dir.path().join("a.txt");
        let src2 = dir.path().join("b.txt");
        let dest = dir.path().join("c.txt");

        fs::write(&dest, "existing file").unwrap();
        fs::write(&src1, "src 1 contents").unwrap();
        fs::write(&src2, "src 2 contents").unwrap();

        move_unique(&src1, &dest).unwrap();
        move_unique(&src2, &dest).unwrap();

        assert!(dir.path().join("c(1).txt").exists());
        assert!(dir.path().join("c(2).txt").exists());
    }

    #[test]
    fn move_unique_is_idempotent() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dest = dir.path().join("b.txt");

        fs::write(&src, "test").unwrap();

        move_unique(&src, &dest).unwrap();
        
        let result = move_unique(&src, &dest);

        assert!(result.is_ok());

        assert!(dest.exists());
        assert!(!dir.path().join("b(1).txt").exists());
    }
}
