use async_trait::async_trait;

use crate::{RepositoryError, node::Node};

#[async_trait]
pub trait NodeRespository: Send + Sync + 'static {
    async fn get_all(&self) -> Result<Vec<Node>, RepositoryError>;

    async fn add(&self, node: &mut Node) -> Result<(), RepositoryError>;
    async fn update(&self, node: &Node) -> Result<(), RepositoryError>;
}
