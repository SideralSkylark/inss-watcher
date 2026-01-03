use std::{path::Path, process::Command};

pub fn extract_text(path: &Path) -> anyhow::Result<String> {

    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .arg("-l").arg("por")
        .arg("--psm").arg("6")
        .output()?;

    if !output.status.success() {
        anyhow::bail!("event=tesseract_failed path={:?}", path);
    }

    let text = String::from_utf8(output.stdout)?;

    Ok(text)
}
