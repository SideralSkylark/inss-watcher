use clap::{Parser, Subcommand};
use inss_watcher::app::orchestrator::{self, StatusResponse};
use inss_watcher::config::Settings;
use inss_watcher::infra::logging;
use inss_watcher::infra::notifications;
use rusqlite::Connection;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tracing::info;

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
    Doctor,
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

    let settings = Settings::load()?;
    settings.validate()?;
    settings.esure_dirs()?;

    match cli.command {
        Cmd::Start => {
            let _log_guard = logging::init(&settings.logs.output_path)?;
            info!(
                version = env!("CARGO_PKG_VERSION"),
                "INSS Watcher daemon started"
            );

            if let Err(e) = orchestrator::start(settings) {
                notifications::notify_failure(
                    "INSS Watcher daemon failed",
                    Some(&format!("{}", e)),
                );
                return Err(e);
            }
        }
        Cmd::DryRun { path } => {
            let _log_guard = logging::init(&settings.logs.output_path)?;
            inss_watcher::app::processor::dry_run(path)?;
        }
        Cmd::Doctor => {
            let _log_guard = logging::init(&settings.logs.output_path)?;
            run_doctor()?;
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
    use serde::Serialize;
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
    };

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

fn run_doctor() -> anyhow::Result<()> {
    let settings = Settings::load()?;
    settings.validate()?;
    settings.esure_dirs()?;

    println!("doctor: running checks...");

    let mut issues = Vec::new();

    if let Err(e) = check_ocr_available() {
        issues.push(format!("OCR unavailable: {e}"));
    }

    if let Err(e) = check_db_reachable(&settings.db.path) {
        issues.push(format!("database unreachable: {e}"));
    }

    match check_daemon_status(&settings.daemon.socket_path) {
        Ok(status) => {
            println!("- daemon: running");
            println!("  queue_depth: {}", status.queue_depth);
            println!("  matched documents: {}", status.matched);
            println!("  unmatched documents: {}", status.unmatched.len());
        }
        Err(e) => {
            issues.push(format!("daemon check failed: {e}"));
        }
    }

    let failed_lines = check_recent_log_errors(&settings.logs.output_path)?;
    if !failed_lines.is_empty() {
        issues.push(format!("recent failure log entries:\n{}", failed_lines.join("\n")));
    }

    if issues.is_empty() {
        println!("doctor: ok");
        return Ok(());
    }

    println!("doctor: issues found:");
    for issue in issues {
        println!("- {issue}");
    }

    anyhow::bail!("doctor found issues")
}

fn check_ocr_available() -> anyhow::Result<()> {
    let status = std::process::Command::new("tesseract")
        .arg("--version")
        .status()?;

    if !status.success() {
        anyhow::bail!("tesseract is not available or returned non-zero status");
    }

    Ok(())
}

fn check_db_reachable(path: &PathBuf) -> anyhow::Result<()> {
    let conn = Connection::open(path)?;
    conn.execute("SELECT 1", [],)?;
    Ok(())
}

fn check_daemon_status(socket_path: &PathBuf) -> anyhow::Result<StatusResponse> {
    let mut stream = UnixStream::connect(socket_path)?;
    let request = json!({ "command": "status" }).to_string();
    writeln!(stream, "{request}")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    let status: StatusResponse = serde_json::from_str(&response)?;
    Ok(status)
}

fn check_recent_log_errors(log_dir: &PathBuf) -> anyhow::Result<Vec<String>> {
    let log_path = log_dir.join("inss_daemon.log");
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let text = std::fs::read_to_string(&log_path)?;
    let errors = text
        .lines()
        .rev()
        .filter(|line| {
            line.contains("ERROR") || line.contains("error") || line.contains("failed")
        })
        .take(10)
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    let mut errors = errors;
    errors.reverse();
    Ok(errors)
}
