use log::{debug, info};
use inss_watcher::infra::{persistence, watch};
use inss_watcher::app::processor;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("event=app_started");

    persistence::init("inss.db")?;
    info!("event=db_initialized");

    let downloads = dirs::download_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    debug!("event=watch_folder_resolved path={:?}", downloads);

    info!("event=watcher_started path={:?}", downloads);
    watch::start(downloads.clone(), |path| {
        processor::process_file(path);
    })?;

    Ok(())
}
