use crate::Error;
use app_forge_kit_telemetry_tracing::debug;
use std::path::{Path, PathBuf};

pub struct Provider {
    path: PathBuf,
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
        let path = Self::resolve_path(&self.path)?;
        debug!("read config from {}", path.display());

        let content = std::fs::read(path)?;

        toml::from_slice::<T>(&content).map_err(|err| err.into())
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            path: "./config.toml".into(),
        }
    }
}
