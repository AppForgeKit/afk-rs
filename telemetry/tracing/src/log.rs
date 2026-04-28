use crate::types::Level;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{EnvFilter, fmt};

pub fn init() -> WorkerGuard {
    let (writer, guard) = non_blocking(std::io::stdout());

    fmt()
        .with_writer(writer)
        .with_max_level(Level::DEBUG)
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    guard
}
