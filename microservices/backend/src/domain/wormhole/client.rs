use std::error::Error;
use hyper::{body::Bytes, Request};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioExecutor,
};
use hyper_tls::HttpsConnector;
use http_body_util::{BodyExt, Empty};
use crate::library::env::get_config;

#[derive(Clone)]
pub struct WormholeClient {
    client: Client<HttpsConnector<HttpConnector>, Empty<Bytes>>,
    base_url: String,
}

impl WormholeClient {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new())
            .build::<_, Empty<Bytes>>(https);
        let base_url = get_config().wormholescan_base_url.clone();

        Self { client, base_url }
    }

    pub async fn get_vaas(&self, chain_id: u16, emitter: &str) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/vaas/{}/{}", self.base_url, chain_id, emitter);
        let req = Request::builder()
            .method("GET")
            .uri(url)
            .body(Empty::<Bytes>::new())?;

        let response = self.client
            .request(req)
            .await?;

        let body = response.into_body().collect().await?.to_bytes();
        Ok(body)
    }
}