use notify::{EventKind, RecursiveMode, Watcher};
use tracing::instrument;
use tracing::{debug, info, warn};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

use crate::config::settings::ProcessingSettings;

#[instrument(
    name = "watcher",
    skip_all,
    fields(
        watch_path = ?paths
    )
)]
pub fn start(paths: Vec<PathBuf>, processing: &ProcessingSettings, mut handler: impl FnMut(PathBuf)) -> anyhow::Result<()> {
    info!("starting filesystem watcher");

    let (tx_evt, rx_evt) = mpsc::channel();
    let (tx_work, rx_work) = mpsc::channel::<PathBuf>();

    let mut watcher = notify::recommended_watcher(tx_evt)?;

    for path in &paths {
        watcher.watch(path, RecursiveMode::Recursive)?;
        info!(path = %path.display(), "watching directory");
    }

    thread::spawn(move || {
        for res in rx_evt {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, "wather error");
                    continue;
                }
            };

            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                debug!(
                    kind = ?event.kind,
                    "event ignored"
                );
                continue;
            }

            for path in event.paths {
                if is_candidate_pdf(&path) {
                    debug!(
                        file = %path.display(),
                        "candidate PDF detected"
                    );
                    let _ = tx_work.send(path);
                }
            }
        }
    });

    for path in rx_work {
        if wait_until_stable(&path, processing.stable_checks, processing.stable_delay_ms) {
            info!(
                file = %path.display(),
                "file stable, dispatching for processing"
            );
            handler(path);
        } else {
            warn!(
                file = %path.display(),
                "file did not stabilize"
            );
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


fn wait_until_stable(path: &PathBuf, stable_checks: usize, stable_delay_ms: u64) -> bool {
    let mut last_size = None;

    for attempt in 0..stable_checks {
        match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();

                debug!(
                    file = %path.display(),
                    attempt,
                    size,
                    "checking file stability"
                );

                if Some(size) == last_size {
                    return true;
                }
                last_size = Some(size);
            }
            Err(e) => {
                warn!(
                    file = %path.display(),
                    error = %e,
                    "failed to read file metadata"
                );
                return false;
            }
        }

        thread::sleep(Duration::from_millis(stable_delay_ms));
    }

    false
}
