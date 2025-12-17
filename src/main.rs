use log::info;
use log::debug;

mod watch;
mod inss;
mod pdf;
mod fs;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    debug!{"fetching downloads folder"}
    let downloads = dirs::download_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("No home dir"))?;

    info!("folder found: {:?}", downloads);
    debug!{"starting watcher"}
    watch::start(downloads.clone(), |path| {
        if let Ok(text) = pdf::extract_text(&path) {
            if !inss::is_inss(&text) {
                debug!{"not inss guide: {}", &text}
                return;
            }

            if let Some((month, year)) = inss::extract_reference_date(&text) {
                let out = fs::inss_output_dir(month, year);
                let _ = fs::ensure_dir(&out);

                let mut dest = out;
                dest.push(path.file_name().unwrap());
                let _ = fs::move_if_missing(&path, &dest);
            }
        }
    })?;

    Ok(())
}
