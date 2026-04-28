mod service;

use app_forge_kit_grpc_client::{ChannelProvider, ChannelProviderFromConfig};
use app_forge_kit_grpc_server::Provider as GrpcServerProvider;
use app_forge_kit_grpc_server::RoutesBuilder as GrpcServerRoutesBuilder;
use app_forge_kit_http_client::{RequestBuilderFromConfig, RequestBuilderProvider};
use app_forge_kit_http_server::Provider as HttpServerProvider;
use app_forge_kit_http_server::routing::{FromRef, Router as HttpServerRouter};
use app_forge_kit_service::Observer;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct HttpConfig {
    server: app_forge_kit_http_server::Config,
    clients: HashMap<String, app_forge_kit_http_client::Config>,
}

#[derive(Default, Deserialize)]
struct GrpcConfig {
    #[serde(default)]
    server: app_forge_kit_grpc_server::Config,
    clients: HashMap<String, app_forge_kit_grpc_client::Config>,
}

#[derive(Deserialize)]
struct Config {
    http: HttpConfig,
    #[serde(default)]
    grpc: GrpcConfig,
}

struct AppState {
    client: Arc<dyn RequestBuilderProvider + Send + Sync>,
    channel: Arc<dyn ChannelProvider + Send + Sync>,
}

impl AppState {
    fn new(
        client: Arc<dyn RequestBuilderProvider + Send + Sync>,
        channel: Arc<dyn ChannelProvider + Send + Sync>,
    ) -> Self {
        Self { client, channel }
    }
}

impl FromRef<AppState> for service::Service {
    fn from_ref(input: &AppState) -> Self {
        Self::new(input.client.clone(), input.channel.clone())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _tracing_log_guard = app_forge_kit_telemetry_tracing::log::init();

    let config = app_forge_kit_config::Provider::new()
        .with_path("./config.toml")
        .read::<Config>()?;

    let http_clients = config.http.clients.request_builder_from_config()?;

    let grpc_clients = config.grpc.clients.channel_provider_from_config();

    let app_state = AppState::new(
        http_clients
            .get("example")
            .ok_or(anyhow::Error::msg(
                "http client `example` not found".to_string(),
            ))?
            .clone(),
        grpc_clients
            .get("echo")
            .ok_or(anyhow::Error::msg(
                "grpc client `echo` not found".to_string(),
            ))?
            .clone(),
    );

    let http_server_router = HttpServerRouter::new().merge(
        HttpServerRouter::new()
            .merge(service::Service::routes().with_state(service::Service::from_ref(&app_state))),
    );

    let http_server = HttpServerProvider::new()
        .with_config(config.http.server)
        .with_router(http_server_router);

    let mut grpc_server_routes_builder = GrpcServerRoutesBuilder::default();
    grpc_server_routes_builder.add_service(service::GrpcService::new(service::Service::from_ref(
        &app_state,
    )));

    let grpc_server = GrpcServerProvider::new()
        .with_config(config.grpc.server)
        .with_reflection_fds(service::GRPC_SERVICE_FDS)
        .with_routes(grpc_server_routes_builder.routes());

    let observer = Observer::new();

    observer.register(Box::new(http_server))?;
    observer.register(Box::new(grpc_server))?;

    observer.run().await?;

    Ok(())
}
