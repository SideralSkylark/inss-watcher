use std::{thread::spawn, path::PathBuf, sync::mpsc::{self, Receiver}};
use anyhow::Context;
use tracing::{debug, info};

use crate::{app::processor, config::{Settings}, infra::{persistence, watch}};

pub struct Daemon {
    state: State,
    settings: Settings,
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

        let settings = self.settings.clone();
        spawn(move || {
            processor::process_file(event.path, &settings);
        });
        Ok(())
    }

    fn handle_command(&mut self, command: Command) -> anyhow::Result<bool> {
        match command {
            Command::Stop => { 
                self.state = State::Stopping;
                return Ok(true);
            },
            Command::Resume => { 
                self.state = State::Running;
            },
            Command::Rescan => { 
                rescan();
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
    Resume,
    Rescan,
}

pub struct Event {
    pub path: PathBuf 
}

pub fn start() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<Message>();
    let settings = Settings::load()?;
    let db_path = settings.db.path.canonicalize().unwrap_or(settings.db.path.clone());

    persistence::init(&db_path)
        .context("database initialization failed")?;
    info!("database initialized");
    
    let mut daemon = Daemon { state: State::Starting, settings: settings };

    watch::start(
        daemon.settings.watcher.dirs_to_watch.clone(),
        &daemon.settings.processing,
        tx.clone()
    )?;

    daemon.run(rx)
}

fn rescan() {

}

