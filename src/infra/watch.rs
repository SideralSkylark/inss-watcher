use notify::{EventKind, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

pub fn start(path: PathBuf, mut handler: impl FnMut(PathBuf)) -> anyhow::Result<()> {
    let (tx_evt, rx_evt) = mpsc::channel();
    let (tx_work, rx_work) = mpsc::channel::<PathBuf>();

    let mut watcher = notify::recommended_watcher(tx_evt)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    thread::spawn(move || {
        for res in rx_evt {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("event=watch_error error={}", e);
                    continue;
                }
            };

            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                continue;
            }

            for path in event.paths {
                if is_candidate_pdf(&path) {
                    let _ = tx_work.send(path);
                }
            }
        }
    });

    for path in rx_work {
        if wait_until_stable(&path) {
            handler(path);
        } else {
            log::warn!("event=file_unstable path={:?}", path);
        }
    }

    Ok(())
}

fn is_candidate_pdf(path: &PathBuf) -> bool {
    path.is_file()
        && path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
}


fn wait_until_stable(path: &PathBuf) -> bool {
    let mut last_size = None;

    for _ in 0..6 {
        match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                if Some(size) == last_size {
                    return true;
                }
                last_size = Some(size);
            }
            Err(_) => return false,
        }

        thread::sleep(Duration::from_millis(400));
    }

    false
}
