use inss_watcher::config::Settings;
use tracing::{info};
use anyhow::Context;
use inss_watcher::infra::{persistence, watch, logging};
use inss_watcher::app::processor;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init()?;
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "INSS Watcher daemon started"
    );

    let settings = Settings::load()?;
    let db_path = settings.db.path.canonicalize().unwrap_or(settings.db.path);

    persistence::init(&db_path)
        .context("database initialization failed")?;

    info!("database initialized");

    watch::start(settings.watcher.dirs_to_watch, &settings.processing, |path| {
        processor::process_file(path);
    })?;

    Ok(())
}
