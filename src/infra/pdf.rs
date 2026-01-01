use std::path::Path;

pub fn extract_text(path: &Path) -> anyhow::Result<String> {
    pdf_extract::extract_text(path).map_err(Into::into)
}

pub fn to_image(path: &Path) -> anyhow::Result<String> {

}
