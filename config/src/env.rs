pub struct EnvConfig {
    separator: Option<String>,
    prefix: Option<String>,
    ignore_empty: bool,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn separator(mut self, separator: &str) -> Self {
        self.separator = Some(separator.to_string());
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn ignore_empty(mut self, ignore_empty: bool) -> Self {
        self.ignore_empty = ignore_empty;
        self
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            separator: Some("_".to_string()),
            prefix: None,
            ignore_empty: true,
        }
    }
}

impl From<&EnvConfig> for config::Environment {
    fn from(config: &EnvConfig) -> Self {
        let mut env = config::Environment::default();

        if let Some(prefix) = &config.prefix {
            env = env.prefix(prefix);
        }

        if let Some(separator) = &config.separator {
            env = env.separator(separator);
        }

        if config.ignore_empty {
            env = env.ignore_empty(true);
        }

        env
    }
}
