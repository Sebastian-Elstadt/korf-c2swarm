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
        sqlx::query_as!(domain::node::Node, "SELECT * FROM nodes")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn add(&self, node: &mut Node) -> Result<(), RepositoryError> {
        let uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO nodes (
                nodus_id, 
                mac_addr, 
                asym_sec_algo, 
                asym_sec_pubkey, 
                cpu_arch, 
                hostname, 
                username, 
                device_name, 
                account_name, 
                first_seen_at, 
                last_seen_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, NOW(), NOW()
            ) RETURNING id;
            "#,
            node.nodus_id,
            node.mac_addr,
            node.asym_sec_algo,
            node.asym_sec_pubkey,
            node.cpu_arch,
            node.hostname,
            node.username,
            node.device_name,
            node.account_name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        node.id = uuid;

        Ok(())
    }

    async fn update(&self, node: &Node) -> Result<(), RepositoryError> {
        sqlx::query_scalar!(
            r#"
            UPDATE nodes
            SET 
                nodus_id = $2,
                mac_addr = $3,
                asym_sec_algo = $4,
                asym_sec_pubkey = $5,
                cpu_arch = $6,
                hostname = $7,
                username = $8,
                device_name = $9,
                account_name = $10,
                first_seen_at = $11,
                last_seen_at = $12
            WHERE id = $1
            "#,
            node.id,
            node.nodus_id,
            node.mac_addr,
            node.asym_sec_algo,
            node.asym_sec_pubkey,
            node.cpu_arch,
            node.hostname,
            node.username,
            node.device_name,
            node.account_name,
            node.first_seen_at,
            node.last_seen_at
        )
        .execute(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        Ok(())
    }
}
