use app_forge_kit_grpc_client::{ChannelProvider, types};
use app_forge_kit_http_client::RequestBuilderProvider;
use app_forge_kit_http_server::routing::{Router as HttpServerRouter, State, get, post};
use echo::{DoRequest, DoResponse};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tonic::{Request, Response, Status, async_trait};

mod echo {
    app_forge_kit_grpc_common::proto::include_proto!("echo");

    pub const FDS: &[u8] = app_forge_kit_grpc_common::proto::include_file_descriptor_set!("echo");
}

#[derive(Clone)]
pub struct Service {
    client: Arc<dyn RequestBuilderProvider + Send + Sync>,
    channel_provider: Arc<dyn ChannelProvider + Send + Sync>,
    channel: OnceCell<Arc<types::Channel>>,
}

impl Service {
    pub fn new(
        client: Arc<dyn RequestBuilderProvider + Send + Sync>,
        channel: Arc<dyn ChannelProvider + Send + Sync>,
    ) -> Self {
        Self {
            client,
            channel_provider: channel,
            channel: OnceCell::new(),
        }
    }

    pub fn routes() -> HttpServerRouter<Self> {
        HttpServerRouter::new().route(
            "/",
            get(Self::handle_request).merge(post(Self::handle_request)),
        )
    }

    async fn handle_request(State(state): State<Self>) -> String {
        let channel = state
            .channel
            .get_or_try_init::<types::Error, _, _>(async || {
                Ok(Arc::new(state.channel_provider.provide().await?))
            })
            .await
            .unwrap();

        let mut client = echo::echo_client::EchoClient::new(channel.clone().as_ref().clone());

        let result = client
            .r#do(app_forge_kit_grpc_client::types::Request::new(echo::DoRequest {
                param: "/".to_string(),
            }))
            .await
            .unwrap();

        result.into_inner().param
    }
}

#[async_trait]
impl echo::echo_server::Echo for Service {
    async fn r#do(&self, _request: Request<DoRequest>) -> Result<Response<DoResponse>, Status> {
        let result = self
            .client
            .get(&("/".parse().unwrap()))
            .unwrap()
            .send()
            .await
            .unwrap();

        Ok(Response::new(DoResponse {
            param: result.text().await.unwrap(),
        }))
    }
}

pub type GrpcService = echo::echo_server::EchoServer<Service>;
pub const GRPC_SERVICE_FDS: &[u8] = echo::FDS;
