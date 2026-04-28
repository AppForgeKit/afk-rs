use crate::config::Config;
use app_forge_kit_service::{Error, Observable, Signal};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::service::Routes;
use tonic::transport::Server;

pub struct Provider<'a> {
    done: Arc<Mutex<Option<tokio::sync::watch::Sender<Option<()>>>>>,
    config: Config,
    routes: Routes,
    reflection_fds: Vec<&'a [u8]>,
}

const DEFAULT_REFLECTION_FDS_CAPACITY: usize = 8;

#[allow(clippy::new_without_default)]
impl<'a> Provider<'a> {
    pub fn new() -> Self {
        Provider {
            done: Arc::new(Mutex::new(None)),
            config: Config::default(),
            routes: Routes::builder().routes(),
            reflection_fds: Vec::with_capacity(DEFAULT_REFLECTION_FDS_CAPACITY),
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        Self { config, ..self }
    }

    pub fn with_routes(self, routes: Routes) -> Self {
        Provider { routes, ..self }
    }

    pub fn with_reflection_fds(mut self, fds: &'a [u8]) -> Self {
        self.reflection_fds.push(fds);

        self
    }
}

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:50051";

#[async_trait]
impl<'a> Observable for Provider<'a> {
    async fn serve(&self) -> Result<(), Error> {
        let config = self.config.clone();

        let tcp_listener = TcpListener::bind(
            config.listen.unwrap_or(
                DEFAULT_LISTEN_ADDR
                    .parse::<SocketAddr>()
                    .map_err(|err| Error::from(crate::Error::AddrParseError(err)))?,
            ),
        )
        .await?;

        let tcp_listener_incoming = TcpListenerStream::new(tcp_listener);

        let (done_tx, mut done_rx) = tokio::sync::watch::channel(None);
        self.done.lock().await.replace(done_tx);

        let mut ref_sb = tonic_reflection::server::Builder::configure();
        let mut ref_sb_alpha = tonic_reflection::server::Builder::configure();

        for fds in self.reflection_fds.clone().into_iter() {
            ref_sb = ref_sb.register_encoded_file_descriptor_set(fds);
            ref_sb_alpha = ref_sb_alpha.register_encoded_file_descriptor_set(fds);
        }

        let ref_server_v1 = ref_sb
            .build_v1()
            .map_err(|err| Error::from(crate::Error::TonicReflectionError(err)))?;

        let ref_server_v1alpha = ref_sb_alpha
            .build_v1alpha()
            .map_err(|err| Error::from(crate::Error::TonicReflectionError(err)))?;

        Server::builder()
            .add_routes(self.routes.clone())
            .add_service(ref_server_v1)
            .add_service(ref_server_v1alpha)
            .serve_with_incoming_shutdown(tcp_listener_incoming, async move {
                let _ = done_rx.changed().await;
            })
            .await
            .map_err(|err| Error::from(crate::Error::TransportError(err)))
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
