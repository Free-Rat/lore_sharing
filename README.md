⚙️ Lore-Sharing-Project

Rust + Axum + SQLite (sqlx) REST API

File responsibilities

Cargo.toml: project metadata, dependencies (axum, sqlx, serde, dotenv, tokio)

src/main.rs: entry point; load config, initialize DB pool, build and run Axum
server

src/config.rs: config loading (dotenv), database URL, server settings

src/db.rs: initialize and migrate database pool (sqlx)

src/error.rs: custom error types and conversions for HTTP responses

src/lib.rs: re-export modules for easier imports

routes/

mod.rs: aggregate route registration

users.rs: GET /users, POST /users, GET /users/{id}, PUT/PATCH/DELETE /users/{id}

events.rs: endpoints under /events and nested timelines

timelines.rs: /timelines and nested /timelines/{id} and /timelines/{id}/events

universes.rs: /universes and nested /universes/{id}

merges.rs: /timeline_merges

models/

Define struct for each resource, with serde derives and sqlx::FromRow

handlers/

Implement actual business logic: extract path/query/body, call DB layer, return
JSON responses

migrations/

SQL files managed by sqlx migrate for schema creation and versioning
