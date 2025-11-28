use log::info;
use anyhow::Result;
use std::{sync::mpsc};
use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::{classifier, date, mover, pdf, utils};

pub fn start() -> Result<()> {
    info!("Watcher started!");

    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    path.push("Downloads");

    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                let should_process = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_)
                );

                if should_process {
                    process_event(event);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn process_event(event: Event) {
    for path in &event.paths {
        if !path.is_file() {
            continue;
        }

        if is_pdf_file(path) {
            println!("PDF detected: {}", path.display());

            match pdf::read_text(path) {
                Ok(text) => {
                    if classifier::is_inss_file(&text){
                        if let Some((month, year)) = date::parse_reference_date(&text) {
                            if let Ok(out_dir) = utils::build_output_dir(month, year) {
                                let _ = mover::move_file(path, &out_dir);
                            }
                        } else {
                            println!("Failed to extract reference date");
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading PDF {}: {:?}", path.display(), e);
                }
            }

        } else {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                info!("Other file [{}]: {}", ext, path.display());
            }
        }
    }
}

fn is_pdf_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

