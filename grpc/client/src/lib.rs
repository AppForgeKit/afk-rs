// Internal modules
#[cfg(feature = "default")]
mod config;
mod interceptors;
#[cfg(feature = "default")]
mod provider;

// Definitions
pub mod types {
    use crate::interceptors::Metadata;
    use http::uri::InvalidUri;
    use tonic::service::interceptor::InterceptedService;
    use tonic::transport::Error as TransportError;
    pub use tonic::{Request, Response, Status};

    pub type Channel = InterceptedService<tonic::transport::Channel, Metadata>;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error(transparent)]
        Transport(#[from] TransportError),
        #[error(transparent)]
        InvalidUri(#[from] InvalidUri),
    }
}

use tonic::async_trait;

#[async_trait]
pub trait ChannelProvider {
    async fn provide(&self) -> Result<types::Channel, types::Error>;
}

// Implementation
#[cfg(feature = "default")]
mod default {
    use crate::ChannelProvider;
    pub use crate::config::Config;
    pub use crate::provider::Provider;
    use std::collections::HashMap;
    use std::sync::Arc;

    pub trait ChannelProviderFromConfig {
        type ChannelProvider;

        fn channel_provider_from_config(self) -> Self::ChannelProvider;
    }

    impl ChannelProviderFromConfig for Config {
        type ChannelProvider = Arc<dyn ChannelProvider + Send + Sync>;

        fn channel_provider_from_config(self) -> Self::ChannelProvider {
            Arc::new(Provider::new(&self))
        }
    }

    impl ChannelProviderFromConfig for HashMap<String, Config> {
        type ChannelProvider = HashMap<String, Arc<dyn ChannelProvider + Send + Sync>>;

        fn channel_provider_from_config(self) -> Self::ChannelProvider {
            self.into_iter()
                .map(|(key, config)| (key, config.channel_provider_from_config()))
                .collect()
        }
    }
}

#[cfg(feature = "default")]
pub use default::*;
