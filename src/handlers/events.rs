use std::sync::Arc;
use axum::{extract::{Extension, Path}, Json};
use serde_json::json;
use serde::Deserialize;
use sqlx::{SqlitePool, QueryBuilder};
use axum::http::StatusCode;
use crate::models::event::Event;

pub async fn list_events(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, description, reference, image, thumbnail, author_id
        FROM events 
        ORDER BY id
        "#
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let events_json = rows
        .into_iter()
        .map(|r| {
            // println!("id:{} nick:{} desc:{:?}", r.id, r.nickname, r.description);
            json!({
                "id": r.id,
                "name": r.name,
                "description": r.description,
                "reference": r.reference,
                "image": r.image,
                "thumbnail": r.thumbnail,
                "author_id": r.author_id
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(events_json)))
}


#[derive(Deserialize)]
pub struct PostEvent {
    pub name: String,
    pub description: String,
    pub reference: String,
    pub image: Option<String>,     // URL or path
    pub thumbnail: Option<String>, // URL or path
    pub author_id: i64,
}

pub async fn create_event(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<PostEvent>,
) -> Result<Json<Event>, StatusCode> {
    let result = sqlx::query_as!(
        Event,
        r#"
        INSERT INTO events (name, description, reference, image, thumbnail, author_id)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id, name, description, reference, image, thumbnail, author_id
        "#,
        payload.name,
        payload.description,
        payload.reference,
        payload.image,
        payload.thumbnail,
        payload.author_id
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
         eprintln!("create event DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result))
}

pub async fn get_event_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<Event>, StatusCode> {
    let result = sqlx::query_as!(
        Event,
        "SELECT id, name, description, reference, image, thumbnail, author_id FROM events WHERE id = ?",
        id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
         eprintln!("get events by id DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(event) => Ok(Json(event)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_event_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM events WHERE id = ?", id)
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
pub struct UpdateEvent {
    pub name: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub image: Option<String>,     // URL or path
    pub thumbnail: Option<String>, // URL or path
    pub author_id: i64,
}

pub async fn update_event(
    Path(event_id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<Event>, StatusCode> {
    // Start building the query
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("UPDATE events SET ");

    // Track whether we've added any fields
    let mut first = true;

    if let Some(name) = payload.name {
        if !first {
            qb.push(", ");
        }
        qb.push("name = ");
        qb.push_bind(name);
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
    if let Some(refer) = payload.reference {
        if !first {
            qb.push(", ");
        }
        qb.push("reference = ");
        qb.push_bind(refer);
        first = false;
    }
    if let Some(img) = payload.image {
        if !first {
            qb.push(", ");
        }
        qb.push("image = ");
        qb.push_bind(img);
        first = false;
    }
    if let Some(thumb) = payload.thumbnail {
        if !first {
            qb.push(", ");
        }
        qb.push("thumbnail = ");
        qb.push_bind(thumb);
        first = false;
    }

    if first {
        // no fields were bound
        return Err(StatusCode::BAD_REQUEST);
    }

    // Finish with the WHERE clause
    qb.push(" WHERE id = ");
    qb.push_bind(event_id);
    qb.push(" AND author_id = ");
    qb.push_bind(payload.author_id);

    // Build and execute
    let result = qb
        .build()
        .execute(&*pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(get_event_by_id(Path(event_id), Extension(pool)).await.unwrap())
    }
}
