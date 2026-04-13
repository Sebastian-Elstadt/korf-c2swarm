use async_trait::async_trait;
use domain::{RepositoryError, node::NodeLogEntry, repositories::NodeLogRespository};
use sqlx::types::Uuid;

pub struct PgNodeLogRepository {
    pool: sqlx::PgPool,
}

impl PgNodeLogRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NodeLogRespository for PgNodeLogRepository {
    async fn get_by_node_id(&self, node_id: Uuid) -> Result<Vec<NodeLogEntry>, RepositoryError> {
        sqlx::query("SELECT * FROM node_logs WHERE node_id = $1 ORDER BY created_at ASC")
            .bind(node_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?
            .into_iter()
            .map(|row| NodeLogEntry::from_pg_row(row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn add(&self, entry: &mut NodeLogEntry) -> Result<(), RepositoryError> {
        let uuid = sqlx::query_scalar(
            r#"
            INSERT INTO node_logs (
                node_id,
                event_type,
                text_content,
                ipv4_addr,
                network_port,
                network_protocol
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6
            ) RETURNING id;
            "#,
        )
        .bind(&entry.node_id)
        .bind(entry.event_type.clone() as i16)
        .bind(&entry.text_content)
        .bind(entry.ipv4_addr.map(|v| v.to_string()))
        .bind(entry.network_port.map(|v| v as i32))
        .bind(entry.network_protocol.clone().map(|v| v as i16))
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        entry.id = uuid;

        Ok(())
    }
}
