use crate::Error;
use crate::env::EnvConfig;
use app_forge_kit_telemetry_tracing::debug;
use config::{Config, Environment, File};
use std::path::{Path, PathBuf};

pub struct Provider {
    path: PathBuf,
    env_config: Option<EnvConfig>,
}

impl Provider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..self
        }
    }

    pub fn with_env_config(self, env_config: EnvConfig) -> Self {
        Self {
            env_config: Some(env_config),
            ..self
        }
    }

    pub fn without_env_config(self) -> Self {
        Self {
            env_config: None,
            ..self
        }
    }

    fn resolve_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
        let path_ref = path.as_ref();

        if path_ref.is_relative() {
            std::fs::canonicalize(path_ref).map_err(|err| err.into())
        } else {
            Ok(path_ref.to_path_buf())
        }
    }

    pub fn read<T>(&self) -> Result<T, Error>
    where
        T: for<'de> serde::de::Deserialize<'de>,
    {
        let mut builder = Config::builder();

        let path = Self::resolve_path(&self.path)?;
        debug!("read config from {}", path.display());

        builder = builder.add_source(File::from(path));

        if let Some(env_config) = &self.env_config {
            builder = builder.add_source(Environment::from(env_config));
        }

        builder.build()?.try_deserialize().map_err(|err| err.into())
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            path: "./config.toml".into(),
            env_config: Some(EnvConfig::default()),
        }
    }
}
