mod db;
mod models;

pub use models::{NodeRecord, RegistrationInput};
pub use sqlx::PgPool;

pub struct DataModule {
    
}

pub async fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let db_pool = db::connect(&database_url).await?;
    run_migrations(&db_pool).await?;

    Ok(())
}