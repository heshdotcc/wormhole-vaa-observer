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
}

impl AppState {
    pub async fn new(repositories: Repositories) -> Result<Self, Error> {
        Ok(Self {
            repositories: Arc::new(repositories),
        })
    }

    pub fn wormhole_repository(&self) -> &Repository<VaaRequest, VaaResponse> {
        self.repositories.wormhole()
    }

    pub fn vaas_repository(&self) -> &Repository<VaaRecord, VaaRecordView> {
      self.repositories.vaas()
    }
}