use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    pdf_extract::extract_text(path).map_err(Into::into)
}

pub fn pdf_to_img(path: &Path) -> anyhow::Result<PathBuf> {
    let stem = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("PDF has no file stem: {:?}", path))?
        .to_string_lossy();

    let out_dir = std::env::temp_dir().join("inss_watcher");
    std::fs::create_dir_all(&out_dir)?;

    let output = out_dir.join(format!("{stem}.png"));

    let status = Command::new("pdftoppm")
        .arg("-f").arg("1")
        .arg("-singlefile")
        .arg("-r").arg("300")
        .arg("-gray")
        .arg("-png")
        .arg(path)
        .arg(out_dir.join(&*stem))
        .status()?;

    if !status.success() {
        anyhow::bail!("pdftoppm failed for {:?}", path);
    }

    if !output.exists() {
        anyhow::bail!("pdftoppm did not generate image for {:?}", path);
    }

    log::info!(
        "event=pdf_rendered page=1 image={:?}",
        output
    );

    Ok(output)
}

// #[test]
// fn extract_text_from_simple_pdf() {
//     let pdf = Path::new("../tests/fixtures/simple.pdf");
//
//     let text = extract_text(pdf).unwrap();
//
//     assert!(!text.trim().is_empty());
// }

// #[test]
// fn fails_when_pdf_does_not_exist() {
//     let path = Path::new("../tests/fixtures/missing.pdf");
//
//     let err = pdf_to_img(path).unwrap_err();
//
//     assert!(err.to_string().contains("pdftoppm"));
// }

