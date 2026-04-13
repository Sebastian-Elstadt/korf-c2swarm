use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    RepositoryError,
    node::{Node, NodeCommandEntry, NodeLogEntry},
};

#[async_trait]
pub trait NodeRespository: Send + Sync + 'static {
    async fn get_all(&self) -> Result<Vec<Node>, RepositoryError>;
    async fn get_by_nodus_id(&self, nodus_id: [u8; 32]) -> Result<Option<Node>, RepositoryError>;

    async fn add(&self, node: &mut Node) -> Result<(), RepositoryError>;
    async fn update(&self, node: &Node) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait NodeLogRespository: Send + Sync + 'static {
    async fn get_by_node_id(&self, node_id: Uuid) -> Result<Vec<NodeLogEntry>, RepositoryError>;

    async fn add(&self, entry: &mut NodeLogEntry) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait NodeCommandRepository: Send + Sync + 'static {
    async fn get_by_node_id(&self, node_id: Uuid) -> Result<Vec<NodeCommandEntry>, RepositoryError>;
    async fn get_queued(&self) -> Result<Vec<NodeCommandEntry>, RepositoryError>;

    async fn add(&self, entry: &mut NodeCommandEntry) -> Result<(), RepositoryError>;
    async fn update(&self, entry: &NodeCommandEntry) -> Result<(), RepositoryError>;
}
