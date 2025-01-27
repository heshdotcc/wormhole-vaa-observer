use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioExecutor,
};
use hyper_tls::HttpsConnector;
use tracing::{debug};

use crate::library::config::get_config;
use crate::library::errors::Error;


#[derive(Clone)]
pub struct RestClient {
    client: Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    base_url: String,
}

impl RestClient {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new())
            .build::<_, Empty<Bytes>>(https);
        let base_url = get_config().wormholescan_base_url.clone();

        Self { client, base_url }
    }

    pub async fn get_vaas(&self, chain_id: u16, emitter_address: &str) -> Result<Bytes, Error> {
        let url = format!("{}/vaas/{}/{}", self.base_url, chain_id, emitter_address);
        let req = Request::builder()
            .method("GET")
            .uri(url.clone())
            .body(Empty::<Bytes>::new())
            .map_err(|e| Error::Request(e.to_string()))?;

        let response = self.client
            .request(req)
            .await
            .map_err(|e| Error::External(format!("Failed to request {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(Error::External(format!(
                "API returned status {}: {}",
                response.status(),
                String::from_utf8_lossy(&response.into_body().collect().await
                    .map_err(|e| Error::External(e.to_string()))?
                    .to_bytes())
            )));
        }

        let bytes = response.into_body().collect().await
            .map_err(|e| Error::External(e.to_string()))?
            .to_bytes();

        let response_str = String::from_utf8_lossy(&bytes);
        debug!("Raw response from {}: {}", url, response_str);

        Ok(bytes)
    }
}