use crate::config::Settings;

pub struct Daemon {
    state: State,
    config: Settings,
}

enum Message {
    Command(Command),
}

pub enum Command {
    Start,
    Stop,
    Resume,
    Rescan,
}

pub enum State {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}
