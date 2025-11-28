pub mod classifier;
pub mod date;
pub mod mover;
pub mod pdf;
pub mod utils;
pub mod watcher;

use env_logger::Builder;
use std::io::Write;
use log::info;
use anyhow::Result;

fn main() -> Result<()> {
    Builder::new()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting watcher!");
    watcher::start()?;

    return Ok(())
}
