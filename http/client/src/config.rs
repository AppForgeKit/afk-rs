use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize, Clone)]
pub struct AuthBasic {
    #[serde(alias = "user")]
    pub username: String,
    pub password: Option<String>,
}

#[derive(Deserialize, Clone)]
pub enum Auth {
    Basic(AuthBasic),
    Bearer(String),
    Header(HashMap<String, String>),
}

#[derive(Deserialize, Clone)]
pub struct Proxy {
    #[serde(alias = "dst")]
    pub destination: Url,
    pub auth: Option<Auth>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(alias = "dst")]
    pub destination: Option<Url>,
    pub auth: Option<Auth>,
    pub proxy: Option<Proxy>,
    pub headers: Option<HashMap<String, String>>,
}
