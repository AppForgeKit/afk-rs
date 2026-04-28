use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Default, Deserialize, Clone)]
pub struct Config {
    pub listen: Option<SocketAddr>,
}
