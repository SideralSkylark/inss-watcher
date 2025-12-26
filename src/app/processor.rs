use std::path::PathBuf;
use log::{debug, info, warn, error};

use crate::domain::inss;
use crate::infra::{pdf, fs};

pub fn process_file(path: PathBuf) {
    info!("processing file: {:?}", path);

    let text = match pdf::extract_text(&path) {
        Ok(t) => t,
        Err(e) => {
            error!("failed to extract text from {:?}: {}", path, e);
            return;
        }
    };

    if !inss::is_inss(&text) {
        debug!("not an INSS guide: {:?}", path);
        return;
    }

    let (month, year) = match inss::extract_reference_date(&text) {
        Some(d) => d,
        None => {
            warn!("INSS guide without reference date: {:?}", path);
            return;
        }
    };

    let out = fs::inss_output_dir(month, year);
    if let Err(e) = fs::ensure_dir(&out) {
        error!("failed to create output dir {:?}: {}", out, e);
        return;
    }

    let mut dest = out;
    dest.push(path.file_name().unwrap());

    match fs::move_unique(&path, &dest) {
        Ok(_) => info!("moved {:?} -> {:?}", path, dest),
        Err(e) => error!("failed to move {:?}: {}", path, e),
    }
}
