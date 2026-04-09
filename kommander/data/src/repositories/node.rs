use async_trait::async_trait;
use domain::{RepositoryError, node::Node, repositories::NodeRespository};

pub struct PgNodeRepository {
    pool: sqlx::PgPool,
}

impl PgNodeRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NodeRespository for PgNodeRepository {
    async fn get_all(&self) -> Result<Vec<Node>, RepositoryError> {
        Ok(vec![])
    }

    async fn add(&self, node: &mut Node) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn update(&self, node: &mut Node) -> Result<(), RepositoryError> {
        Ok(())
    }
}
