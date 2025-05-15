use std::sync::Arc;
use axum::{extract::Extension, Json};
use serde_json::json;
use sqlx::SqlitePool;
use axum::http::StatusCode;

pub async fn list_universes(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT name, description
        FROM universes 
        ORDER BY name 
        "#
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users_json = rows
        .into_iter()
        .map(|r| {
            // println!("id:{} nick:{} desc:{:?}", r.id, r.nickname, r.description);
            json!({
                "name": r.name,
                "description": r.description
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(users_json)))
}
