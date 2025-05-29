use sqlx::{
    sqlite::SqlitePoolOptions,
     SqlitePool,
};
// use std::path::Path;
use std::sync::Arc;

pub async fn init_db() -> Result<Arc<SqlitePool>, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/lore.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    Ok(Arc::new(pool))
}
