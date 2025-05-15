use sqlx::{
    migrate::{Migrator, MigrateDatabase},
    sqlite::SqlitePoolOptions,
    Sqlite, SqlitePool,
};
// use std::path::Path;
use std::sync::Arc;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_db() -> Result<Arc<SqlitePool>, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/lore.db".to_string());

    // If the file doesn't exist yet, create it
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    MIGRATOR.run(&pool).await?;

    // --- seed a test user if table is empty ---
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;
    if user_count.0 == 0 {
        sqlx::query!(
            r#"
            INSERT INTO users (nickname, description)
            VALUES (?, ?)
            "#,
            "test_user",
            "This is a seeded test user"
        )
        .execute(&pool)
        .await?;
        // tracing::info!("Seeded one test user");
        println!("Seeded ont test user");
    }

    // 
    let universe_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM universes")
        .fetch_one(&pool)
        .await?;
    if universe_count.0 == 0 {
        sqlx::query!(
            r#"
            INSERT INTO universes (name, description)
            VALUES (?, ?)
            "#,
            "Elden Ring",
            "The universe of The Lands Between"
        )
        .execute(&pool)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO universes (name, description)
            VALUES (?, ?)
            "#,
            "League of Leagends",
            "The universe of Runeterra"
        )
        .execute(&pool)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO universes (name, description)
            VALUES (?, ?)
            "#,
            "Lord of the Rings",
            "The universe of Middle-earth"
        )
        .execute(&pool)
        .await?;
        println!("Seeded three universe");
    }

    Ok(Arc::new(pool))
}
