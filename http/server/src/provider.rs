use crate::config::Config;
use crate::routing::Router;
use app_forge_kit_service::{Error, Observable, Signal};
use app_forge_kit_telemetry_tracing::types::Level;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};

pub struct Provider {
    done: Arc<Mutex<Option<tokio::sync::watch::Sender<Option<()>>>>>,
    config: Option<Config>,
    router: Router,
}

#[allow(clippy::new_without_default)]
impl Provider {
    pub fn new() -> Self {
        Provider {
            done: Arc::new(Mutex::new(None)),
            config: None,
            router: Router::new(),
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        Self {
            config: Some(config),
            ..self
        }
    }

    pub fn with_router(self, router: Router) -> Self {
        Self { router, ..self }
    }
}

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:8080";

#[async_trait]
impl Observable for Provider {
    async fn serve(&self) -> Result<(), Error> {
        let config = self.config.clone().unwrap_or_default();

        let tcp_listener = TcpListener::bind(
            config.listen.unwrap_or(
                DEFAULT_LISTEN_ADDR
                    .parse::<SocketAddr>()
                    .map_err(|err| Error::from(crate::Error::from(err)))?,
            ),
        )
        .await?;

        let (done_tx, mut done_rx) = tokio::sync::watch::channel(None);
        self.done.lock().await.replace(done_tx);

        let router = self.router.clone().layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async move {
                let _ = done_rx.changed().await;
            })
            .await?;

        Ok(())
    }

    async fn signal(&self, signal: &Signal) -> Result<(), Error> {
        if signal.is_terminate()
            && let Some(done) = self.done.lock().await.take()
        {
            let _ = done.send(None);
        }

        Ok(())
    }
}
