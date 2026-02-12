use std::{path::PathBuf, sync::mpsc::Receiver};

use crate::{config::Settings, infra::watch};

pub struct Daemon {
    state: State,
    config: Settings,
}

impl Daemon {
    fn run(&mut self, rx: Receiver<Message>) -> anyhow::Result<()> {
        self.state = State::Starting;
    }
}

pub enum Message {
    Command(Command),
    Event(Event),
}

pub enum Command {
    Start,
    Stop,
    Resume,
    Rescan,
}

pub struct Event {
    pub path: PathBuf 
}

pub enum State {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

pub fn start() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel<Message>();
    let settings = Settings::default();

    watch::start(settings.watcher.dirs_to_watch, &settings.processing, tx.clone());

}

fn handle_event(event: &Event) -> anyhow::Result<()> {

}

fn handle_command(command: &Command) -> anyhow::Result<()> {

}
