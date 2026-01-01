use std::{path::{Path, PathBuf}, process::Command};

use anyhow::Context;

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    pdf_extract::extract_text(path).map_err(Into::into)
}

// pub fn to_images(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
// }
