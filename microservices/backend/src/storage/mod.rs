use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;
use std::sync::Arc;

// Trait for converting between write and read models
pub trait ReadModel: Sized {
    type WriteModel;
    fn from_write_model(model: &Self::WriteModel) -> Self;
}

pub trait HasId {
    fn id(&self) -> Uuid;
}

#[async_trait]
pub trait RepositoryImpl<W, R>: Send + Sync + 'static 
where
    W: Serialize + DeserializeOwned + Send + Sync + HasId + 'static,  // Add HasId requirement
    R: ReadModel<WriteModel = W> + Send + Sync + 'static,     // Read model
{
    async fn create(&self, item: W) -> W;
    async fn get(&self, id: Uuid) -> Option<R>;
    async fn list(&self) -> Vec<R>;
    async fn delete(&self, id: Uuid) -> Option<W>;
    async fn update(&self, id: Uuid, item: W) -> Option<W>;
}

// ToDo: Explain facade pattern
#[derive(Clone)]
pub struct Repository<W, R> {
    inner: Arc<dyn RepositoryImpl<W, R>>,
}

impl<W, R> Repository<W, R>
where
    W: Serialize + DeserializeOwned + Send + Sync + HasId + 'static,
    R: ReadModel<WriteModel = W> + Send + Sync + 'static,
{
    pub fn new(impl_: impl RepositoryImpl<W, R> + 'static) -> Self {
        Self {
            inner: Arc::new(impl_),
        }
    }

    pub async fn create(&self, item: W) -> W {
        self.inner.create(item).await
    }

    pub async fn get(&self, id: Uuid) -> Option<R> {
        self.inner.get(id).await
    }

    pub async fn list(&self) -> Vec<R> {
        self.inner.list().await
    }

    pub async fn delete(&self, id: Uuid) -> Option<W> {
        self.inner.delete(id).await
    }

    pub async fn update(&self, id: Uuid, item: W) -> Option<W> {
        self.inner.update(id, item).await
    }
}

pub mod memory;
// pub mod postgres; 