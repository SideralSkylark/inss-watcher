use anyhow::Context;
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
};
use tracing::{debug, warn};

use crate::{
    app::processor,
    config::Settings,
    infra::{deps, ipc, persistence, watch},
};

pub struct Daemon {
    state: State,
    #[allow(dead_code)]
    settings: Settings,
    sender: SyncSender<PathBuf>,
    queue_depth: Arc<AtomicUsize>,
}

pub enum State {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub queue_depth: usize,
    pub matched: usize,
    pub unmatched: Vec<UnmatchedArtifact>,
}

#[derive(Debug, Serialize)]
pub struct UnmatchedArtifact {
    #[serde(rename = "type")]
    pub kind: String,
    pub reference_num: String,
    pub period: String,
    pub path: String,
}

impl Daemon {
    fn run(&mut self, rx: Receiver<Message>) -> anyhow::Result<()> {
        self.state = State::Running;

        while let Ok(message) = rx.recv() {
            match message {
                Message::Command(c) => {
                    if self.handle_command(c)? {
                        break;
                    }
                }
                Message::Event(e) => self.handle_event(e)?,
            }
        }

        self.state = State::Stopped;
        Ok(())
    }

    /// dispatches file processing asynchronously (non-blocking)
    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        if !matches!(self.state, State::Running) {
            debug!("daemon unavailable ignoring");
            return Ok(());
        }

        if let Err(e) = self.sender.try_send(event.path) {
            match e {
                mpsc::TrySendError::Full(p) => {
                    warn!(file = %p.display(), "work queue full, skipping file. use 'rescan' to process it later");
                }
                mpsc::TrySendError::Disconnected(_) => {
                    warn!("work queue closed unexpectedly");
                }
            }
        } else {
            self.queue_depth.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    fn handle_command(&mut self, command: Command) -> anyhow::Result<bool> {
        match command {
            Command::Stop => {
                self.state = State::Stopping;
                return Ok(true);
            }
            Command::Rescan => {
                self.rescan();
            }
            Command::Pause => {
                self.pause();
            }
            Command::Resume => {
                self.resume();
            }
            Command::Status(reply) => {
                self.status(reply);
            }
        }

        Ok(false)
    }

    fn rescan(&mut self) {
        use tracing::info;
        use walkdir::WalkDir;

        info!("rescanning watched directories");

        for dir in &self.settings.watcher.dirs_to_watch {
            let pdfs = WalkDir::new(dir)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|x| x.eq_ignore_ascii_case("pdf"))
                        .unwrap_or(false)
                });

            for entry in pdfs {
                let path = entry.into_path();
                if self.sender.try_send(path.clone()).is_ok() {
                    self.queue_depth.fetch_add(1, Ordering::SeqCst);
                } else {
                    warn!(file = %path.display(), "work queue full or closed during rescan");
                }
            }
        }
    }

    fn pause(&mut self) {
        use tracing::info;

        info!("daemon paused");
        self.state = State::Paused;
    }

    fn resume(&mut self) {
        use tracing::info;

        info!("daemon resumed");
        self.state = State::Running;
    }

    fn status(&mut self, reply: std::sync::mpsc::SyncSender<StatusResponse>) {
        let response = match persistence::query_status(self.queue_depth.load(Ordering::SeqCst)) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "failed to query status");
                return;
            }
        };
        if reply.send(response).is_err() {
            tracing::warn!("status reply channel closed before send");
        }
    }
}

pub enum Message {
    Command(Command),
    Event(Event),
}

pub enum Command {
    Stop,
    Rescan,
    Pause,
    Resume,
    Status(std::sync::mpsc::SyncSender<StatusResponse>),
}

pub struct Event {
    pub path: PathBuf,
}

pub fn start(settings: Settings) -> anyhow::Result<()> {
    deps::check()?;

    if let Some(parent) = settings.db.path.parent() {
        std::fs::create_dir_all(parent).context("failed to create data directory")?;
    }

    let num_workers = settings.processing.worker_threads;
    let dirs_to_watch = settings.watcher.dirs_to_watch.clone();
    let processing = settings.processing.clone();
    let db_path = settings.db.path.clone();

    let (work_tx, work_rx) = mpsc::sync_channel::<PathBuf>(64);
    let queue_depth = Arc::new(AtomicUsize::new(0));

    let shared_rx = Arc::new(Mutex::new(work_rx));
    for _ in 0..num_workers {
        let rx = Arc::clone(&shared_rx);
        let s = settings.clone();
        let qd = Arc::clone(&queue_depth);
        std::thread::spawn(move || {
            loop {
                let path = match rx.lock().unwrap().recv() {
                    Ok(p) => p,
                    Err(_) => break,
                };

                processor::process_file(path, &s);
                qd.fetch_sub(1, Ordering::SeqCst);
            }
        });
    }

    let (tx, rx) = mpsc::channel::<Message>();

    ipc::start(&settings.daemon.socket_path, tx.clone())?;

    let mut daemon = Daemon {
        state: State::Starting,
        settings,
        sender: work_tx,
        queue_depth,
    };

    persistence::init(&db_path).context("database initialization failed")?;
    debug!("database initialized");

    watch::start(dirs_to_watch, &processing, tx.clone())?;

    let tx_signal = tx.clone();
    ctrlc::set_handler(move || {
        let _ = tx_signal.send(Message::Command(Command::Stop));
    })?;
    daemon.run(rx)
}

