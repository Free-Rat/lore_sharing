#!/usr/bin/env rust-script
//! This is a regular crate doc comment, but it also contains a partial
//! Cargo manifest.  Note the use of a *fenced* code block, and the
//! `cargo` "language".
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1.45.0", features = ["full"] }
//! sqlx = { version = "0.8.5", features = ["sqlite", "runtime-tokio-rustls", "macros"] }
//! dotenv = "0.15"
//! ```

use sqlx::{
    migrate::{Migrator, MigrateDatabase},
    // sqlite::SqlitePoolOptions,
    Sqlite, SqlitePool,
};
use std::path::Path;
// use std::sync::Arc;

// static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
// static MIGRATOR: Migrator = sqlx::migrate!("/home/freerat/project/lore_sharing/migrations");

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    
    let path = std::env::var("MIGRATIONS_PATH")
        .unwrap_or_else(|_| "./migrations".to_string());

    let migrator = Migrator::new(Path::new(&path)).await?;

    println!("Loaded migrations from: {}", path);

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. sqlite://data/lore.db)");

    // let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/lore.db".to_string());
    //
    // Only for SQLite: ensure dir + file exist
        let path_str = db_url
            .strip_prefix("sqlite://")
            .expect("DATABASE_URL must start with sqlite://");
        let db_path = Path::new(path_str);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("failed to create DB dir {:?}: {}", parent, e));
        }
        if !db_path.exists() {
            std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
            // std::fs::File::create(db_path).unwrap();
            if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
                Sqlite::create_database(&db_url).await?;
            }
       }
        // `create_database` is a no-op for SQLite, but we still call it for consistency:
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await?;
        }

    let pool = SqlitePool::connect(&db_url).await?;
    migrator.run(&pool).await?;

    println!("✅ Migrations complete.");

    Ok(())
}
