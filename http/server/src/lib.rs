#[cfg(feature = "default")]
mod config;
#[cfg(feature = "default")]
mod provider;

#[cfg(feature = "default")]
pub use config::Config;
#[cfg(feature = "default")]
pub use provider::Provider;

#[cfg(feature = "routing")]
pub mod routing {
    pub use axum::Router;
    pub use axum::extract::{FromRef, State};
    pub use axum::routing::{any, connect, delete, get, head, options, patch, post, put, trace};
}

pub mod types {
    pub mod http {
        pub use http::StatusCode;
    }

    pub use axum::body::Body;
    pub use axum::response::{Response, Result};
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> types::Response<types::Body> {
        let (status_code, message) = match self {
            Self::Unknown(err) => (http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            _ => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Unknown error".into(),
            ),
        };

        (status_code, message).into_response()
    }
}

impl From<Error> for app_forge_kit_service::Error {
    fn from(err: Error) -> app_forge_kit_service::Error {
        app_forge_kit_service::Error::Unknown(Box::new(err))
    }
}
