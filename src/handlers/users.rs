use std::sync::Arc;
use axum::{extract::{Extension, Path}, Json};
use serde_json::json;
use serde::Deserialize;
use sqlx::{SqlitePool, QueryBuilder};
use axum::http::StatusCode;
use crate::models::user::User;

pub async fn list_users(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT id, nickname, description
        FROM users
        ORDER BY id
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
                "id": r.id,
                "nickname": r.nickname,
                "description": r.description
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(users_json)))
}


#[derive(Deserialize)]
pub struct PostUser {
    pub nickname: String,
    pub description: Option<String>,
}

pub async fn create_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<PostUser>,
) -> Result<Json<User>, StatusCode> {
    let result = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (nickname, description)
        VALUES (?, ?)
        RETURNING id, nickname, description
        "#,
        payload.nickname,
        payload.description
        // payload.description.unwrap_or_else(|| "Default description".to_string())
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
         eprintln!("create user DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result))
}

pub async fn get_user_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<User>, StatusCode> {
    // println!("============== get_user_by_id ====================");
    let result = sqlx::query_as!(
        User,
        "SELECT id, nickname, description FROM users WHERE id = ?",
        id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
         eprintln!("get user by id DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;
    // eprintln!("=== GET USER ID === > {:?}", result);
    // println!("=== GET USER ID === > {:?}", result);

    match result {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_user_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(&*pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                Ok(StatusCode::NO_CONTENT)
            }
        }
        Err(e) => {
            eprintln!("Delete error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub nickname: Option<String>,
    pub description: Option<String>,
}

pub async fn update_user(
    Path(user_id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, StatusCode> {
    // Start building the query
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("UPDATE users SET ");

    // Track whether we've added any fields
    let mut first = true;

    if let Some(nick) = payload.nickname {
        if !first {
            qb.push(", ");
        }
        qb.push("nickname = ");
        qb.push_bind(nick);
        first = false;
    }
    if let Some(desc) = payload.description {
        if !first {
            qb.push(", ");
        }
        qb.push("description = ");
        qb.push_bind(desc);
        first = false;
    }

    if first {
        // no fields were bound
        return Err(StatusCode::BAD_REQUEST);
    }

    // Finish with the WHERE clause
    qb.push(" WHERE id = ");
    qb.push_bind(user_id);

    // Build and execute
    let result = qb
        .build()
        .execute(&*pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(get_user_by_id(Path(user_id), Extension(pool)).await.unwrap())
    }
}
