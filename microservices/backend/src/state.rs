use std::sync::Arc;
use crate::storage::Repository;
use crate::domain::wormhole::models::{VaaRequest, VaaResponse, VaaRecord, VaaRecordView};
use crate::library::config::get_config;
use crate::library::errors::Error;

#[derive(Clone)]
pub struct Repositories {
    wormhole: Repository<VaaRequest, VaaResponse>,
    vaas: Repository<VaaRecord, VaaRecordView>,
}

impl Repositories {
    pub fn new(
        wormhole: Repository<VaaRequest, VaaResponse>,
        vaas: Repository<VaaRecord, VaaRecordView>,
    ) -> Self {
        Self { wormhole, vaas }
    }

    pub fn wormhole(&self) -> &Repository<VaaRequest, VaaResponse> {
        &self.wormhole
    }

    pub fn vaas(&self) -> &Repository<VaaRecord, VaaRecordView> {
        &self.vaas
    }
}

#[derive(Clone)]
pub struct AppState {
    repositories: Arc<Repositories>,
    // TODO: Refactor to support proper DI pattern.
    // pub rest_handler: Arc<WormholeHandler<RestClient>>,
    // pub grpc_handler: Arc<WormholeHandler<GrpcClient>>,
}

impl AppState {
    pub async fn new(repositories: Repositories) -> Result<Self, Error> {
        /* TODO: Refactor to support proper DI pattern.
        let config = get_config();
        let rest_handler = Arc::new(WormholeHandler::new_rest());
        let grpc_handler = Arc::new(
            WormholeHandler::new_grpc(
                config.wormhole_spy_addr
                    .as_ref()
                    .ok_or_else(|| Error::Connection("Missing wormhole spy address".to_string()))?
                    .clone()
            ).await?
        );
        */

        Ok(Self {
            repositories: Arc::new(repositories),
            // rest_handler,
            // grpc_handler,
        })
    }

    pub fn wormhole_repository(&self) -> &Repository<VaaRequest, VaaResponse> {
        self.repositories.wormhole()
    }

    pub fn vaas_repository(&self) -> &Repository<VaaRecord, VaaRecordView> {
      self.repositories.vaas()
    }
}