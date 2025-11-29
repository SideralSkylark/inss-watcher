use std::path::Path;
use anyhow::Result;
use pdf_extract::extract_text;
use log::info;

pub fn read_text(path: &Path) -> Result<String> {
    info!("Extracting text");
    let text = extract_text(path)?;
    Ok(text)
}
