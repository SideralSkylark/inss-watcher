use std::path::Path;
use anyhow::Result;
use pdf_extract::extract_text;

pub fn read_text(path: &Path) -> Result<String> {
    let text = extract_text(path)?;
    Ok(text)
}
