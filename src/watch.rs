use notify::{EventKind, RecursiveMode, Watcher};
use std::{collections::HashSet, path::PathBuf, sync::mpsc, time::Duration};

pub fn start(path: PathBuf, mut handler: impl FnMut(PathBuf)) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    let mut seen = HashSet::new();

    for res in rx {
        let event = match res {
            Ok(e) => e,
            Err(_) => continue,
        };

        log::debug!("RAW EVENT: {:?}", event);
        if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
            continue;
        }
        
        for path in event.paths {
            if !path.is_file() || !path.extension().map(|e| e == "pdf").unwrap_or(false) {
                continue;
            }

            if seen.insert(path.clone()) {
                std::thread::sleep(Duration::from_secs(1));
                handler(path);
            }
        }
    }

    Ok(())
}
