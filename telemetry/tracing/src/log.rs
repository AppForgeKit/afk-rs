use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt, prelude::*};

pub fn init() -> WorkerGuard {
    let (writer, guard) = non_blocking(std::io::stdout());

    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_writer(writer)
            .json()
            .log_internal_errors(true)
            .with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env_lossy(),
            ),
    );

    #[cfg(feature = "sentry")]
    // Подключает интеграцию tracing с Sentry.
    let subscriber = subscriber.with(sentry_tracing::layer());

    subscriber.init();

    guard
}
