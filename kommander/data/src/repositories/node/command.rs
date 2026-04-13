use async_trait::async_trait;
use domain::{RepositoryError, node::NodeCommandEntry, repositories::NodeCommandRepository};
use sqlx::types::Uuid;

pub struct PgNodeCommandRepository {
    pool: sqlx::PgPool,
}

impl PgNodeCommandRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NodeCommandRepository for PgNodeCommandRepository {
    async fn get_by_node_id(
        &self,
        node_id: Uuid,
    ) -> Result<Vec<NodeCommandEntry>, RepositoryError> {
        sqlx::query("SELECT * FROM node_commands WHERE node_id = $1")
            .bind(node_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?
            .into_iter()
            .map(|row| NodeCommandEntry::from_pg_row(row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn get_queued(&self) -> Result<Vec<NodeCommandEntry>, RepositoryError> {
        sqlx::query("SELECT * FROM node_commands WHERE \"status\" = 0 ORDER BY created_at ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?
            .into_iter()
            .map(|row| NodeCommandEntry::from_pg_row(row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))
    }

    async fn add(&self, entry: &mut NodeCommandEntry) -> Result<(), RepositoryError> {
        let uuid = sqlx::query_scalar(
            r#"
            INSERT INTO node_commands (
                node_id,
                "status",
                command_type,
                last_attempted_at,
                completed_at,
                text_content
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6
            ) RETURNING id;
            "#,
        )
        .bind(&entry.node_id)
        .bind(entry.status.clone() as i16)
        .bind(entry.command_type.clone() as i16)
        .bind(&entry.last_attempted_at)
        .bind(&entry.completed_at)
        .bind(&entry.text_content)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        entry.id = uuid;

        Ok(())
    }

    async fn update(&self, entry: &NodeCommandEntry) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE node_commands SET
                "status" = $2,
                command_type = $3,
                last_attempted_at = $4,
                completed_at = $5,
                text_content = $6
            WHERE id = $1;
            "#,
        )
        .bind(&entry.id)
        .bind(entry.status.clone() as i16)
        .bind(entry.command_type.clone() as i16)
        .bind(&entry.last_attempted_at)
        .bind(&entry.completed_at)
        .bind(&entry.text_content)
        .execute(&self.pool)
        .await
        .map_err(|err| RepositoryError::DbQueryFailure(err.to_string()))?;

        Ok(())
    }
}
