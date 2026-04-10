use async_trait::async_trait;

#[async_trait]
pub trait HealthPort: Send + Sync + 'static {
    async fn ping_db(&self) -> bool;
}