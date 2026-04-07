use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init() -> anyhow::Result<WorkerGuard> {
    let log_dir = std::env::current_dir()?.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY, 
        log_dir, 
        "inss_daemon.log"
    );
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .compact()
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking_file)
                .with_ansi(false)  
                .with_target(true)
                .with_line_number(true)
        )
        .init();

    tracing::info!("Logging system started");
    
    Ok(guard)
}
