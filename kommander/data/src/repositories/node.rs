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
        sqlx::query_as("SELECT * FROM nodes")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn get_by_nodus_id(&self, nodus_id: [u8; 32]) -> Result<Option<Node>, RepositoryError> {
        sqlx::query_as("SELECT * FROM nodes WHERE nodus_id = $1")
            .bind(nodus_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn add(&self, node: &mut Node) -> Result<(), RepositoryError> {
        let uuid = sqlx::query_scalar(
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
                last_seen_at,
                host_local_time
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, NOW(), NOW(),
                $10
            ) RETURNING id;
            "#,
        )
        .bind(&node.nodus_id)
        .bind(&node.mac_addr)
        .bind(&node.asym_sec_algo)
        .bind(&node.asym_sec_pubkey)
        .bind(&node.cpu_arch)
        .bind(&node.hostname)
        .bind(&node.username)
        .bind(&node.device_name)
        .bind(&node.account_name)
        .bind(&node.host_local_time)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        node.id = uuid;

        Ok(())
    }

    async fn update(&self, node: &Node) -> Result<(), RepositoryError> {
        sqlx::query(
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
                last_seen_at = $12,
                host_local_time = $13
            WHERE id = $1
            "#,
        )
        .bind(&node.id)
        .bind(&node.nodus_id)
        .bind(&node.mac_addr)
        .bind(&node.asym_sec_algo)
        .bind(&node.asym_sec_pubkey)
        .bind(&node.cpu_arch)
        .bind(&node.hostname)
        .bind(&node.username)
        .bind(&node.device_name)
        .bind(&node.account_name)
        .bind(&node.first_seen_at)
        .bind(&node.last_seen_at)
        .bind(&node.host_local_time)
        .execute(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        Ok(())
    }
}
