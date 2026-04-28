use crate::config::Config;
use crate::{ChannelProvider, interceptors, types};
use tonic::async_trait;
use tonic::service::InterceptorLayer;
use tonic::transport::Channel;

pub struct Provider {
    config: Config,
}

impl Provider {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl ChannelProvider for Provider {
    async fn provide(&self) -> Result<types::Channel, types::Error> {
        let channel = Channel::from_shared(self.config.destination.to_string())?
            .connect()
            .await?;

        Ok(tower::ServiceBuilder::new()
            .layer(InterceptorLayer::new(interceptors::Metadata(
                self.config.metadata.clone(),
            )))
            .service(channel))
    }
}
