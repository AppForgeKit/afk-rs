use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(alias = "dst")]
    pub destination: Url,
    pub metadata: Option<HashMap<String, String>>,
}
