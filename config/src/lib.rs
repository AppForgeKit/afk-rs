mod provider;

pub use provider::Provider;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error(transparent)]
    TomlDeError(#[from] toml::de::Error),
}
