use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tracing::debug;
use anyhow::{anyhow, bail};

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    let p = path.to_path_buf();
    std::panic::catch_unwind(|| pdf_extract::extract_text(&p))
        .map_err(|_| anyhow!("pdf extraction panicked (malformed font data) for {:?}", path))?
        .map_err(Into::into)
}

pub fn page_count(path: &Path) -> anyhow::Result<usize> {
    let doc = lopdf::Document::load(path)?;
    Ok(doc.get_pages().len())
}

pub fn is_candidate(path: &Path) -> bool {
    let size = match std::fs::metadata(path) {
        Ok(m) => m.len(),
        Err(_) => return false,
    };

    if size > 5 * 1024 * 1024 {
        debug!(file = %path.display(), "skipping: file too large");
        return false;
    }

    match page_count(path) {
        Ok(n) if n > 2 => {
            debug!(file = %path.display(), pages = n, "skipping: too many pages");
            return false;
        }
        Err(e) => {
            debug!(file = %path.display(), error = %e, "skipping: could not read pages");
            return false;
        }
        Ok(_) => true,
    }
}

pub fn pdf_to_img(path: &Path) -> anyhow::Result<PathBuf> {
    let stem = path
        .file_stem()
        .ok_or_else(|| anyhow!("PDF has no file stem: {:?}", path))?
        .to_string_lossy();

    let temp_dir = tempfile::tempdir()?;
    let img_base = temp_dir.path().join(&*stem);

    let status = Command::new("pdftoppm")
        .arg("-f").arg("1")
        .arg("-singlefile")
        .arg("-r").arg("300")
        .arg("-gray")
        .arg("-png")
        .arg(path)
        .arg(&img_base)
        .status()?;

    if !status.success() {
        bail!("pdftoppm failed for {:?}", path);
    }

    let generated_img = temp_dir.path().join(format!("{stem}.png"));
    if !generated_img.exists() {
        bail!("pdftoppm did not generate image for {:?}", path);
    }

    // Move it out of the temp dir before it gets deleted on drop
    let final_dest = std::env::temp_dir().join(format!("inss_ocr_{stem}.png"));
    std::fs::rename(&generated_img, &final_dest)?;

    debug!(
        "event=pdf_rendered page=1 image={:?}",
        final_dest
    );

    Ok(final_dest)
}

