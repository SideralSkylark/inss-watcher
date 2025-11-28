use std::fs;
use std::path::PathBuf;

pub fn build_output_dir(month: u32, year: u32) -> std::io::Result<PathBuf> {
    let mut path = dirs::home_dir().unwrap();
    path.push("INSS");
    path.push(year.to_string());
    path.push(format!("{:02}", month));

    fs::create_dir_all(&path)?;

    Ok(path)
}
