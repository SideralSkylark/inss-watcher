use clap::{Parser, Subcommand};
use tracing::{info};
use inss_watcher::infra::logging;
use inss_watcher::app::orchestrator::{self};

#[derive(Parser)]
#[command(name = "inss-watcher", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Start,
    DryRun {
        #[arg(short, long)]
        path: std::path::PathBuf,
    },
    Ctl {
        #[command(subcommand)]
        action: CtlAction,
    },
}

#[derive(Subcommand)]
enum CtlAction {
    Stop,
    Pause,
    Resume,
    Rescan,
    Status,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Start => {
            let _log_guard = logging::init()?;
            info!(
                version = env!("CARGO_PKG_VERSION"),
                "INSS Watcher daemon started"
            );

            orchestrator::start()?;
        }
        Cmd::DryRun { path } => {
            let _log_guard = logging::init()?;
            inss_watcher::app::processor::dry_run(path)?;
        }
        Cmd::Ctl { action } => {
            let command = match action {
                CtlAction::Stop => "stop",
                CtlAction::Pause => "pause",
                CtlAction::Resume => "resume",
                CtlAction::Rescan => "rescan",
                CtlAction::Status => "status",
            };

            run_ctl(command)?;
        }
    }

    Ok(())
}

fn run_ctl(command: &str) -> anyhow::Result<()> {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
    };
    use serde::Serialize;
 
    #[derive(Serialize)]
    struct IpcRequest<'a> {
        command: &'a str,
    }
 
    let socket_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("inss-watcher/inss-watcher.sock");
 
    let mut stream = UnixStream::connect(&socket_path).map_err(|e| {
        anyhow::anyhow!(
            "could not connect to daemon at {} ({e}). Is it running?",
            socket_path.display()
        )
    })?;
 
    let request = serde_json::to_string(&IpcRequest { command })?;
    writeln!(stream, "{request}")?;
    stream.flush()?;
 
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        println!("{}", line?);
    }
 
    Ok(())
}
