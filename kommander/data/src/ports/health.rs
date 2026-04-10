use async_trait::async_trait;
use domain::ports::HealthPort;

pub struct PgHealthPort {
    pool: sqlx::PgPool,
}

impl PgHealthPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthPort for PgHealthPort {
    async fn ping_db(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }
}
