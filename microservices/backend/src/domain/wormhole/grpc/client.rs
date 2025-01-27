use std::time::Duration;
use tonic::{transport::Channel, Request};
use tokio_stream::StreamExt;
use tracing::{info, debug, error};
use base64::{
  Engine,
  engine::general_purpose::STANDARD as BASE64_STANDARD
};
use hex;

use crate::library::errors::Error;

// Generated proto types
use crate::domain::wormhole::grpc::proto::{
    spy::v1::{
        spy_rpc_service_client::SpyRpcServiceClient,
        EmitterFilter,
        SubscribeSignedVaaRequest,
        FilterEntry,
        SubscribeSignedVaaResponse,
    },
    publicrpc::v1::ChainId,
};

use super::vaa::VaaProcessor;

#[derive(Clone)]
pub struct GrpcClient {
    client: SpyRpcServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect(addr: String) -> Result<Self, Error> {
        let addr = if !addr.starts_with("http") {
            format!("http://{}", addr)
        } else {
            addr
        };

        debug!("Connecting to spy at {}", addr);
        
        let channel = Channel::from_shared(addr)
            .map_err(|e| Error::Connection(e.to_string()))?
            .connect_timeout(Duration::from_secs(5))  // ToDo: Make configurable
            .tcp_nodelay(true)  // Disables Nagle's algorithm
            .connect()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;
            
        let client = SpyRpcServiceClient::new(channel);
        info!("Successfully connected to spy service");
        Ok(Self { client })
    }

    pub async fn subscribe_all_vaas(&mut self, limit: usize) 
        -> Result<(usize, Vec<SubscribeSignedVaaResponse>), Error> 
    {
        debug!("Starting VAA subscription for all messages (limit: {})", limit);

        let request = Request::new(SubscribeSignedVaaRequest {
            filters: vec![], // Empty filters to get all VAAs
        });

        debug!("Sending request: {:#?}", request);
        info!("Starting VAA stream...");
        
        let mut stream = self.client
            .subscribe_signed_vaa(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|e| {
                error!("gRPC subscription error: {:?}", e);
                Error::Subscription(e.to_string())
            })?;

        let mut processor = VaaProcessor::new(limit);
        let mut vaas = Vec::new();

        while let Some(response) = stream.next().await {
            match response {
                Ok(vaa) => {
                    if !processor.process_vaa(vaa.clone()) {
                        vaas.push(vaa);
                        break;
                    }
                    vaas.push(vaa);
                }
                Err(e) => {
                    error!("Error receiving VAA: {}", e);
                }
            }
        }

        Ok((processor.processed_count(), vaas))
    }

    pub async fn subscribe_to_emitter(
        &mut self,
        chain_id: u16,
        emitter_address: &str,
    ) -> Result<(), Error> {
        debug!("Creating filter with chain_id: {}, emitter: {}", chain_id, emitter_address);

        // Ensure emitter is valid hex without 0x prefix
        let emitter_address = emitter_address.trim_start_matches("0x");
        hex::decode(emitter_address)
            .map_err(|_| Error::External("Invalid emitter address format".to_string()))?;

        // Map chain_id to the correct enum value
        let chain_id = match chain_id {
            30 => ChainId::Optimism,
            2 => ChainId::Ethereum,
            4 => ChainId::Bsc,
            6 => ChainId::Avalanche,
            _ => return Err(Error::External("Unsupported chain ID".to_string())),
        };

        let filter = EmitterFilter {
            chain_id: chain_id.into(),
            emitter_address: emitter_address.to_string(),
        };

        let filter_entry = FilterEntry {
            filter: Some(crate::domain::wormhole::grpc::proto::spy::v1::filter_entry::Filter::EmitterFilter(filter)),
        };

        let request = Request::new(SubscribeSignedVaaRequest {
            filters: vec![filter_entry],
        });

        self.handle_vaa_stream(request).await
    }

    // Private helper to handle the stream
    async fn handle_vaa_stream(&mut self, request: Request<SubscribeSignedVaaRequest>) -> Result<(), Error> {
        debug!("Sending request: {:#?}", request);
        info!("Starting VAA stream...");
        
        let mut stream = self.client
            .subscribe_signed_vaa(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|e| {
                error!("gRPC subscription error: {:?}", e);
                Error::Subscription(e.to_string())
            })?;

        let mut vaa_count = 0;
        while let Some(response) = stream.next().await {
            match response {
                Ok(vaa) => {
                    vaa_count += 1;
                    info!(
                        "VAA #{}: {} bytes, hash: {}",
                        vaa_count,
                        vaa.vaa_bytes.len(),
                        hex::encode(&vaa.vaa_bytes[0..32].to_vec()),  // First 32 bytes as identifier
                    );
                    debug!("Full VAA: {}", BASE64_STANDARD.encode(&vaa.vaa_bytes));
                    // TODO: Parse VAA bytes to extract more meaningful data
                }
                Err(e) => {
                    error!("Error receiving VAA: {}", e);
                }
            }
        }

        Ok(())
    }
}

// ToDo: Add more meaeningful tests for the gRPC client
#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::config::get_config;

    #[tokio::test]
    async fn test_spy_subscription() -> Result<(), Error> {
        // Known test values
        const CHAIN_ID: u16 = 30;  // Optimism
        const EMITTER: &str = "000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31";

        let config = get_config();
        let spy_addr = config.wormhole_spy_addr.expect("Spy address not configured");
        
        info!("Testing spy subscription...");
        let mut client = GrpcClient::connect(spy_addr).await?;
        
        client.subscribe_all_vaas(50).await?;
        
        Ok(())
    }
}
