use std::{io::{BufRead, BufReader, Write}, os::unix::net::UnixStream, path::PathBuf};

use serde::Serialize;

#[derive(Serialize)]
struct IpcRequest<'a> {
    command: &'a str,
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("help");

    if command == "help" {
        eprintln!("Usage: inss-ctl <stop|pause|resume|rescan>");
        return Ok(());
    }

    let socket_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("inss-watcher/inss-watcher.sock");

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| anyhow::anyhow!("could not connect to daemon ({e}). Is it running?"))?;

    let request = serde_json::to_string(&IpcRequest { command })?;
    writeln!(stream, "{request}")?;
    stream.flush()?;

    let reader = BufReader::new(stream);
    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}
