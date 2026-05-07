use std::process::Command;

const REQUIRED: &[(&str, &str)] = &[
    ("pdftoppm", "install poppler-utils (apt/brew/nix)"),
    ("tesseract", "install tesseract-ocr (apt/brew/nix)"),
];

pub fn check() -> anyhow::Result<()> {
    let mut missing = Vec::new();

    for (bin, hint) in REQUIRED {
        // `which` on Linux/mac, `where` on Windows
    let found = Command::new(bin)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|_| true)
        .unwrap_or(false);

        if !found {
            missing.push((*bin, *hint));
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    let mut msg = String::from("missing required dependencies:\n");
    for (bin, hint) in &missing {
        msg.push_str(&format!("  - {bin}: {hint}\n"));
    }

    anyhow::bail!(msg)
}
