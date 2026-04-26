use notify::{EventKind, RecursiveMode, Watcher};
use tracing::instrument;
use tracing::{debug, info, warn};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

use crate::config::settings::ProcessingSettings;
use crate::app::orchestrator::{Event, Message};

#[instrument(
    name = "watcher",
    skip_all,
    fields(
        watch_path = ?paths
    )
)]
pub fn start(paths: Vec<PathBuf>, processing: &ProcessingSettings, sender: Sender<Message>) -> anyhow::Result<()> {
    debug!("starting filesystem watcher");

    let (tx_evt, rx_evt) = mpsc::channel();

    let stable_checks = processing.stable_checks;
    let stable_delay_ms = processing.stable_delay_ms;

    thread::spawn(move || {
        let mut watcher = match notify::recommended_watcher(tx_evt) {
            Ok(w) => w,
            Err(e) => {
                warn!(error=%e, "failed to create watcher");
                return;
            }
        };

        for path in &paths {
            if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                warn!(error=%e, path=%path.display(), "failed to watch directory");
                return;
            }

            info!(path = %path.display(), "watching directory");
        }

        for res in rx_evt {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, "watcher error");
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
                    if wait_until_stable(&path, stable_checks, stable_delay_ms) {
                        debug!(
                            file = %path.display(),
                            "file stable, dispatching for processing"
                        );
                        if sender.send(Message::Event(Event { path })).is_err() {
                            warn!("orchestrator channel closed, dropping event");
                        }
                    } else {
                        warn!(
                            file = %path.display(),
                            "file did not stabilize"
                        );
                    }
                }
            }
        }
    });

    Ok(())
}

fn is_candidate_pdf(path: &Path) -> bool {
    path.is_file()
    && path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}


fn wait_until_stable(path: &Path, stable_checks: usize, stable_delay_ms: u64) -> bool {
    let mut last_size: Option<u64> = None;

    for attempt in 0..stable_checks {
        thread::sleep(Duration::from_millis(stable_delay_ms));
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
    }

    false
}
