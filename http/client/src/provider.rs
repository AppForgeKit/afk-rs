use crate::{Error, RequestBuilderProvider, config, types};
use http::{HeaderMap, HeaderName, HeaderValue, header};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

#[derive(Clone)]
pub struct Provider {
    client: reqwest::Client,
    destination: Option<Url>,
    auth: Option<config::Auth>,
    headers: Option<HashMap<String, String>>,
}

impl Provider {
    fn compose_headers(headers_hash_map: &HashMap<String, String>) -> HeaderMap {
        let mut headers = HeaderMap::with_capacity(headers_hash_map.len());

        for (key, value) in headers_hash_map {
            if let (Ok(name), Ok(h_value)) =
                (HeaderName::from_str(key), HeaderValue::from_str(value))
            {
                headers.insert(name, h_value);
            }
        }

        headers
    }
    fn client_builder_proxy_config(
        client_builder: reqwest::ClientBuilder,
        config: &config::Config,
    ) -> Result<reqwest::ClientBuilder, Error> {
        if let Some(config_proxy) = &config.proxy {
            let mut client_proxy = match config_proxy.destination.scheme() {
                "http" => reqwest::Proxy::http(config_proxy.destination.as_str()),
                "https" => reqwest::Proxy::https(config_proxy.destination.as_str()),
                _ => reqwest::Proxy::all(config_proxy.destination.as_str()),
            }?;

            if let Some(auth) = &config_proxy.auth {
                match auth {
                    config::Auth::Basic(auth_basic) => {
                        client_proxy = client_proxy.basic_auth(
                            auth_basic.clone().username.as_str(),
                            auth_basic.clone().password.unwrap_or_default().as_str(),
                        )
                    }
                    config::Auth::Bearer(auth_bearer) => {
                        client_proxy = client_proxy.custom_http_auth(HeaderValue::from_str(
                            format!("Bearer {}", auth_bearer).as_str(),
                        )?)
                    }
                    config::Auth::Header(auth_header) => {
                        let header_map = Self::compose_headers(auth_header);

                        if let Some(value) = header_map.get(header::PROXY_AUTHORIZATION) {
                            client_proxy = client_proxy.custom_http_auth(value.clone());
                        }
                    }
                };
            }

            return Ok(client_builder.proxy(client_proxy));
        }

        Ok(client_builder)
    }

    pub fn new(config: &config::Config) -> Result<Self, Error> {
        let mut client_builder = reqwest::ClientBuilder::new();

        client_builder = Self::client_builder_proxy_config(client_builder, config)?;

        Ok(Self {
            client: client_builder.build()?,
            destination: config.destination.clone(),
            auth: config.auth.clone(),
            headers: config.headers.clone(),
        })
    }

    fn compose_request_url(&self, uri: &types::Uri) -> Result<Url, Error> {
        if let Some(destination) = &self.destination {
            let mut base_url = destination.clone();

            let mut base_url_path = base_url.path();
            base_url_path = base_url_path.strip_suffix("/").unwrap_or(base_url_path);

            let mut uri_path = uri.path();
            uri_path = uri_path.strip_prefix("/").unwrap_or(uri_path);

            let url_path = [base_url_path, uri_path].join("/");

            base_url.set_path(url_path.as_str());
            base_url.set_query(uri.query());

            return Ok(base_url);
        }

        let url = Url::parse(uri.to_string().as_str())?;

        Ok(url)
    }
}

impl RequestBuilderProvider for Provider {
    fn request(
        &self,
        method: types::Method,
        uri: &types::Uri,
    ) -> Result<types::RequestBuilder, Error> {
        let request_url = self.compose_request_url(uri)?;

        let mut request_builder = self.client.request(method, request_url);

        if let Some(auth) = &self.auth {
            match auth {
                config::Auth::Basic(auth_basic) => {
                    request_builder = request_builder
                        .basic_auth(auth_basic.username.clone(), auth_basic.password.clone())
                }
                config::Auth::Bearer(auth_bearer) => {
                    request_builder = request_builder.bearer_auth(auth_bearer.clone())
                }
                config::Auth::Header(auth_header) => {
                    request_builder = request_builder.headers(Self::compose_headers(auth_header))
                }
            }
        }

        if let Some(headers) = &self.headers {
            request_builder = request_builder.headers(Self::compose_headers(headers))
        }

        Ok(request_builder)
    }
}
