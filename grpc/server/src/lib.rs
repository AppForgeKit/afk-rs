mod config;
mod provider;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error(transparent)]
    TonicReflectionError(#[from] tonic_reflection::server::Error),
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
}

impl From<Error> for app_forge_kit_service::Error {
    fn from(err: Error) -> app_forge_kit_service::Error {
        app_forge_kit_service::Error::Unknown(Box::new(err))
    }
}

pub use config::Config;
pub use provider::Provider;
pub use tonic::service::RoutesBuilder;
