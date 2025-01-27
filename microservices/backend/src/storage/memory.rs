use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tracing::info;

use super::{RepositoryImpl, ReadModel, HasId};

pub struct MemoryRepository<W>
where
    W: HasId,
{
    items: Arc<Mutex<HashMap<Uuid, W>>>,
}

impl<W> MemoryRepository<W>
where
    W: HasId,
{
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<W, R> RepositoryImpl<W, R> for MemoryRepository<W>
where
    W: Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + HasId + 'static,
    R: ReadModel<WriteModel = W> + Send + Sync + 'static,
{
    async fn create(&self, item: W) -> W {
        let mut items = self.items.lock().await;
        items.insert(item.id(), item.clone());
        item
    }

    async fn get(&self, id: Uuid) -> Option<R> {
        let items = self.items.lock().await;
        info!("Looking for item with id: {}", id);
        info!("Available items: {:?}", items);
        
        items.get(&id).map(|item| R::from_write_model(item))
    }

    async fn list(&self) -> Vec<R> {
        self.items.lock().await
            .values()
            .map(R::from_write_model)
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Option<W> {
        self.items.lock().await.remove(&id)
    }

    async fn update(&self, id: Uuid, item: W) -> Option<W> {
        let mut items = self.items.lock().await;
        if items.contains_key(&id) {
            items.insert(id, item.clone());
            Some(item)
        } else {
            None
        }
    }
} 