use inss_watcher::config::settings::Settings;
use tracing::{info, error};
use inss_watcher::infra::{persistence, watch, logging};
use inss_watcher::app::processor;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init()?;
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "INSS Watcher daemon started"
    );

    if let Err(e) = persistence::init("inss.db") {
        error!(error = %e, "failed to initialize database");
        return Err(e);
    }
    info!("database initialized");

    let settings = Settings::load()?;


    watch::start(settings.dirs_to_watch, |path| {
        processor::process_file(path);
    })?;

    Ok(())
}
