use log::{debug, info};
use inss_watcher::infra::watch;
use inss_watcher::app::processor;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    debug!{"fetching downloads folder"}
    let downloads = dirs::download_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("No home dir"))?;

    info!("folder found: {:?}", downloads);

    debug!{"starting watcher"}
    watch::start(downloads.clone(), |path| {
        processor::process_file(path);
    })?;

    Ok(())
}
