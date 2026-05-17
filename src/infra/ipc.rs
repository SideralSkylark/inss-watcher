use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::Sender,
    thread,
};
use tracing::{debug, info, warn};

use crate::app::orchestrator::Command;
use crate::app::orchestrator::Message;
use crate::app::orchestrator::StatusResponse;

#[derive(serde::Deserialize)]
pub struct IpcRequest {
    command: String,
}

#[derive(serde::Serialize)]
pub struct IpcResponse {
    message: &'static str,
    status: &'static str,
}

pub fn start(socket_path: &Path, tx: Sender<Message>) -> anyhow::Result<()> {
    debug!("starting ipc socket");
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }

    let listner = UnixListener::bind(socket_path)?;
    info!(path = %socket_path.display(), "IPC socket listening");

    thread::spawn(move || {
        for stream in listner.incoming() {
            let stream = match stream {
                Ok(o) => o,
                Err(e) => {
                    warn!(error = %e, "IPC accept error");
                    continue;
                }
            };

            let tx = tx.clone();
            thread::spawn(move || handle_connection(stream, tx));
        }
    });

    Ok(())
}

fn handle_connection(stream: UnixStream, tx: Sender<Message>) {
    let mut writer = stream.try_clone().expect("clone stream");
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, "IPC read error");
                break;
            }
        };

        let (msg, reply) = match serde_json::from_str::<IpcRequest>(&line) {
            Err(e) => {
                warn!(error = %e, raw = %line, "received invalid json");
                (
                    None,
                    IpcResponse {
                        status: "err",
                        message: "invalid json",
                    },
                )
            }
            Ok(req) => {
                match req.command.as_str() {
                    "stop" => (
                        Some(Message::Command(Command::Stop)),
                        IpcResponse {
                            status: "ok",
                            message: "stopping",
                        },
                    ),
                    "rescan" => (
                        Some(Message::Command(Command::Rescan)),
                        IpcResponse {
                            status: "ok",
                            message: "rescanning",
                        },
                    ),
                    "pause" => (
                        Some(Message::Command(Command::Pause)),
                        IpcResponse {
                            status: "ok",
                            message: "paused",
                        },
                    ),
                    "resume" => (
                        Some(Message::Command(Command::Resume)),
                        IpcResponse {
                            status: "ok",
                            message: "resumed",
                        },
                    ),
                    "status" => {
                        let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);

                        if tx
                            .send(Message::Command(Command::Status(reply_tx)))
                            .is_err()
                        {
                            let _ = writer.write_all(
                                b"{\"status\":\"err\",\"message\":\"daemon shutting down\"}\n",
                            );
                            return;
                        }

                        // Block this IPC thread waiting for the orchestrator's reply
                        match reply_rx.recv_timeout(std::time::Duration::from_secs(5)) {
                            Ok(response) => {
                                // Wrap in the same envelope as other replies
                                #[derive(serde::Serialize)]
                                struct Envelope {
                                    status: &'static str,
                                    #[serde(flatten)]
                                    data: StatusResponse,
                                }
                                let envelope = Envelope {
                                    status: "ok",
                                    data: response,
                                };
                                let mut bytes = serde_json::to_vec(&envelope).unwrap_or_default();
                                bytes.push(b'\n');
                                let _ = writer.write_all(&bytes);
                            }
                            Err(_) => {
                                let _ = writer.write_all(b"{\"status\":\"err\",\"message\":\"status query timed out\"}\n");
                            }
                        }
                        continue;
                    }
                    other => {
                        warn!(cmd = %other, "unknown command");
                        (
                            None,
                            IpcResponse {
                                status: "err",
                                message: "unknown command",
                            },
                        )
                    }
                }
            }
        };

        if let Some(m) = msg {
            if tx.send(m).is_err() {
                let _ = writer
                    .write_all(b"{\"status\":\"err\",\"message\":\"daemon shutting down\"}\n");
                return;
            }
        }

        let mut response = serde_json::to_vec(&reply).unwrap_or_default();
        response.push(b'\n');
        if writer.write_all(&response).is_err() {
            break;
        }
    }
}
