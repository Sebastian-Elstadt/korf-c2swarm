use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod repositories;
pub mod ports;

pub async fn create_database_pool(url: &str) -> PgPool {
    PgPoolOptions::new().connect(url).await.unwrap()
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

// pub async fn upsert_registration(
//     pool: &PgPool,
//     input: &RegistrationInput,
// ) -> Result<(), sqlx::Error> {
//     sqlx::query(
//         r#"
//         INSERT INTO nodes (
//             nodus_id, mac_addr, asym_sec_algo, asym_sec_pubkey,
//             cpu_arch, hostname, username, device_name, account_name,
//             first_seen_at, last_seen_at
//         )
//         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
//         ON CONFLICT (nodus_id) DO UPDATE SET
//             mac_addr = EXCLUDED.mac_addr,
//             asym_sec_algo = EXCLUDED.asym_sec_algo,
//             asym_sec_pubkey = EXCLUDED.asym_sec_pubkey,
//             cpu_arch = EXCLUDED.cpu_arch,
//             hostname = EXCLUDED.hostname,
//             username = EXCLUDED.username,
//             device_name = EXCLUDED.device_name,
//             account_name = EXCLUDED.account_name,
//             last_seen_at = NOW()
//         "#,
//     )
//     .bind(&input.nodus_id[..])
//     .bind(&input.mac_addr)
//     .bind(input.asym_sec_algo)
//     .bind(&input.asym_sec_pubkey[..])
//     .bind(&input.cpu_arch)
//     .bind(input.hostname.as_deref())
//     .bind(input.username.as_deref())
//     .bind(input.device_name.as_deref())
//     .bind(input.account_name.as_deref())
//     .execute(pool)
//     .await?;
//     Ok(())
// }

// pub async fn list_nodes(pool: &PgPool) -> Result<Vec<NodeRecord>, sqlx::Error> {
//     sqlx::query_as::<_, NodeRecord>(
//         r#"
//         SELECT
//             id,
//             nodus_id,
//             mac_addr,
//             asym_sec_algo,
//             asym_sec_pubkey,
//             cpu_arch,
//             hostname,
//             username,
//             device_name,
//             account_name,
//             first_seen_at,
//             last_seen_at
//         FROM nodes
//         ORDER BY last_seen_at DESC
//         "#,
//     )
//     .fetch_all(pool)
//     .await
// }

// pub async fn ping_db(pool: &PgPool) -> Result<(), sqlx::Error> {
//     sqlx::query_scalar::<_, i32>("SELECT 1")
//         .fetch_one(pool)
//         .await?;
//     Ok(())
// }
