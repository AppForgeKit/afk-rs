// Internal modules
#[cfg(feature = "default")]
mod config;
#[cfg(feature = "default")]
mod provider;

// Definition
pub mod types {
    pub use reqwest::{IntoUrl, Method, Request, RequestBuilder, Response};

    pub use http::Uri;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

macro_rules! http_methods {
    ($(($name:ident, $method:ident)),* $(,)?) => {
        pub trait RequestBuilderProvider {
            fn request(
                &self,
                method: types::Method,
                uri: &types::Uri,
            ) -> Result<types::RequestBuilder, Error>;

            $(
                fn $name(
                    &self,
                    uri: &types::Uri,
                ) -> Result<types::RequestBuilder, Error> {
                    self.request(types::Method::$method, uri)
                }
            )*
        }
    };
}

http_methods!(
    (get, GET),
    (post, POST),
    (put, PUT),
    (patch, PATCH),
    (delete, DELETE),
    (head, HEAD),
    (options, OPTIONS),
    (connect, CONNECT),
    (trace, TRACE),
);

// Implementation
#[cfg(feature = "default")]
mod default {
    use crate::Error;
    use crate::RequestBuilderProvider;
    use std::collections::HashMap;
    use std::sync::Arc;

    pub use crate::config::Config;
    pub use crate::provider::Provider;

    pub trait RequestBuilderFromConfig {
        type Provider;
        fn request_builder_from_config(self) -> Result<Self::Provider, Error>;
    }

    impl RequestBuilderFromConfig for Config {
        type Provider = Arc<dyn RequestBuilderProvider + Send + Sync>;

        fn request_builder_from_config(self) -> Result<Self::Provider, Error> {
            Ok(Arc::new(Provider::new(&self)?))
        }
    }

    impl RequestBuilderFromConfig for HashMap<String, Config> {
        type Provider = HashMap<String, Arc<dyn RequestBuilderProvider + Send + Sync>>;
        fn request_builder_from_config(self) -> Result<Self::Provider, Error> {
            self.into_iter()
                .map(|(key, config)| Ok((key, config.request_builder_from_config()?)))
                .collect()
        }
    }
}

#[cfg(feature = "default")]
pub use default::*;
