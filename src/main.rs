use tracing::{info};
use inss_watcher::infra::logging;
use inss_watcher::app::orchestrator;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init()?;
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "INSS Watcher daemon started"
    );

    orchestrator::start()?;

    Ok(())
}
