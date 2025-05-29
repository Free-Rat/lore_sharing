use sqlx::{
    migrate::{
        Migrator,
        MigrateDatabase
    },
    // sqlite::SqlitePoolOptions,
    Sqlite,
    SqlitePool,
};
// use std::path::Path;
// use std::sync::Arc;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. sqlite://data/lore.db)");

    // let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/lore.db".to_string());
    //
    // Only for SQLite: ensure dir + file exist
    // `create_database` is a no-op for SQLite, but we still call it for consistency:
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        panic!("No database_exists")
    }

    // let pool = SqlitePoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_url)
    //     .await?;
    let pool = SqlitePool::connect(&db_url).await?;
    MIGRATOR.run(&pool).await?;

    println!("✅ Migrations complete.");

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

    let event_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await?;
    if event_count.0 == 0 {
        sqlx::query!(
            r#"
            INSERT INTO events (name, description, reference, image, thumbnail, author_id)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            "Test Event",
            "test description",
            "test reference",
            "some url string",
            "another url string",
            1
        )
        .execute(&pool)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO events (name, description, reference, image, thumbnail, author_id)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            "Second Test Event",
            "s test description",
            "s test reference",
            "some url string",
            "another url string",
            1
        )
        .execute(&pool)
        .await?;
        println!("Seeded two events");
    }

    let timelines_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM timelines")
        .fetch_one(&pool)
        .await?;
    if timelines_count.0 == 0 {
        sqlx::query!(
            r#"
            INSERT INTO timelines (author_id, description, start, end, unit, universe_name)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            1,
            "Test Timeline",
            0,
            10000,
            "years",
            "Elden Ring"
        )
        .execute(&pool)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO timelines (author_id, description, start, end, unit, universe_name)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            1,
            "Test Timeline for LoR",
            -10000,
            10000,
            "years",
            "Lord of the Rings"
        )
        .execute(&pool)
        .await?;
        println!("Seeded two timelines");
    }

    let timeline_events_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM timeline_events")
        .fetch_one(&pool)
        .await?;
    if timeline_events_count.0 == 0 {
        sqlx::query!(
            r#"
            INSERT INTO timeline_events (timeline_id, event_id, position)
            VALUES (?, ?, ?)
            "#,
            1,
            1,
            1
        )
        .execute(&pool)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO timeline_events (timeline_id, event_id, position)
            VALUES (?, ?, ?)
            "#,
            1,
            2,
            10
        )
        .execute(&pool)
        .await?;
        println!("Seeded two timeline_events");
    }


    Ok(())
}
