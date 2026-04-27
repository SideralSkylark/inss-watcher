use std::{path::PathBuf, sync::{Arc, Mutex, mpsc::{self, Receiver, SyncSender}}};
use anyhow::Context;
use tracing::{debug, warn};

use crate::{app::processor, config::Settings, infra::{ipc, persistence, watch}};

pub struct Daemon {
    state: State,
    #[allow(dead_code)]
    settings: Settings,
    sender: SyncSender<PathBuf>,
}

pub enum State {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
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
                },
                Message::Event(e) => self.handle_event(e)?,
            }
        }

        self.state = State::Stopped;
        Ok(())
    }

    /// dispatches file processing asynchronously (non-blocking) 
    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        if !matches!(self.state, State::Running) {
            debug!("daemon unavalible ignoring");
            return Ok(());
        }

        if let Err(e) = self.sender.send(event.path) {
            warn!(error=%e, "work queue closed unexpectedly");
        }
        Ok(())
    }

    fn handle_command(&mut self, command: Command) -> anyhow::Result<bool> {
        match command {
            Command::Stop => { 
                self.state = State::Stopping;
                return Ok(true);
            },
            Command::Rescan => { 
                rescan();
            },
            Command::Pause => {
                pause();
            },
            Command::Resume => { 
                self.state = State::Running;
            },
        }

        Ok(false)
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
}

pub struct Event {
    pub path: PathBuf 
}

pub fn start() -> anyhow::Result<()> {
    let settings = Settings::load()?;

    if let Some(parent) = settings.db.path.parent() {
        std::fs::create_dir_all(parent)
            .context("failed to create data directory")?;
    }

    let num_workers = settings.processing.worker_threads;
    let dirs_to_watch = settings.watcher.dirs_to_watch.clone();
    let processing = settings.processing.clone();
    let db_path = settings.db.path.clone();

    let (work_tx, work_rx) = mpsc::sync_channel::<PathBuf>(64);

    let shared_rx = Arc::new(Mutex::new(work_rx));
    for _ in 0..num_workers {
        let rx = Arc::clone(&shared_rx);
        let s = settings.clone();
        std::thread::spawn(move || {
            loop {
                let path = match rx.lock().unwrap().recv() {
                    Ok(p) => p,
                    Err(_) => break,
                };

                processor::process_file(path, &s);
            }
        });
    }

    let (tx, rx) = mpsc::channel::<Message>();

    ipc::start(&settings.daemon.socket_path, tx.clone())?;

    let mut daemon = Daemon {
        state: State::Starting,
        settings,
        sender: work_tx,
    };

    persistence::init(&db_path)
        .context("database initialization failed")?;
    debug!("database initialized");
    
    watch::start(
        dirs_to_watch,
        &processing,
        tx.clone(),
    )?;

    daemon.run(rx)
}

fn rescan() {
    warn!("resan not implemented");
}

fn pause() {
    warn!("pause not implemented");
}

