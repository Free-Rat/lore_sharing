use std::sync::Arc;
use axum::{extract::{Extension, Path}, Json};
use serde_json::json;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, QueryBuilder};
use axum::http::StatusCode;
use crate::models::timeline::Timeline;

    // pub id: i64,
    // pub author_id: i64,
    // pub description: String,
    // pub start: u64,
    // pub end: u64,
    // pub unit: String, // e.g., "seconds", "minutes", "hours", "days", "weeks"
    // pub universe_name: String,
//
pub async fn list(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT id, author_id, description, start, end, unit, universe_name 
        FROM timelines 
        ORDER BY id
        "#
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let timeline_json = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "author_id": r.author_id,
                "description": r.description,
                "start": r.start,
                "end": r.end,
                "universe_name": r.universe_name,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(timeline_json)))
}


#[derive(Deserialize, Serialize)]
pub struct PostTimeline {
    pub author_id: i64,
    pub description: String,
    pub start: i64,     
    pub end: i64, 
    pub unit: String,
    pub universe_name: String,
}

// TODO: validation if universe_name exists in universes
pub async fn create(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<PostTimeline>,
) -> Result<Json<Timeline>, StatusCode> {
    let result = sqlx::query_as!(
            // start AS "start!: u64",
            // end AS "end!: u64",
        Timeline,
        r#"
        INSERT INTO timelines (author_id, description, start, end, unit, universe_name)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING 
            id AS "id!: i64",
            author_id AS "author_id!: i64",
            description,
            start, 
            end,
            unit,
            universe_name
        "#,
        payload.author_id,
        payload.description,
        payload.start,
        payload.end,
        payload.unit,
        payload.universe_name,
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
         eprintln!("create timeline DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result))
}

pub async fn get_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<Timeline>, StatusCode> {
    let result = sqlx::query_as!(
        Timeline,
        "SELECT id, author_id, description, start, end, unit, universe_name FROM timelines WHERE id = ?",
        id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
         eprintln!("get by id DB error: {:?}", e);
         StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(timeline) => Ok(Json(timeline)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM timelines WHERE id = ?", id)
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

#[derive(Deserialize, Debug)]
pub struct UpdateTimeline {
    pub description: Option<String>,
    pub author_id: i64,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub unit: Option<String>,
    pub universe_name: Option<String>,
}

pub async fn update(
    Path(timeline_id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<UpdateTimeline>,
) -> Result<Json<Timeline>, StatusCode> {
    // Start building the query
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("UPDATE timelines SET ");
    // println!("{:?}", payload);

    // Track whether we've added any fields
    let mut first = true;

    // if let Some(name) = payload.name {
    //     if !first {
    //         qb.push(", ");
    //     }
    //     qb.push("name = ");
    //     qb.push_bind(name);
    //     first = false;
    // }
    if let Some(desc) = payload.description {
        if !first {
            qb.push(", ");
        }
        qb.push("description = ");
        qb.push_bind(desc);
        first = false;
    }
// author_id, description, start, end, unit, universe_name
    if let Some(start) = payload.start {
        if !first {
            qb.push(", ");
        }
        qb.push("start = ");
        qb.push_bind(start);
        first = false;
    }
    if let Some(end) = payload.end {
        if !first {
            qb.push(", ");
        }
        qb.push("end = ");
        qb.push_bind(end);
        first = false;
    }
    if let Some(unit) = payload.unit {
        if !first {
            qb.push(", ");
        }
        qb.push("unit = ");
        qb.push_bind(unit);
        first = false;
    }
    if let Some(universe_name) = payload.universe_name {
        if !first {
            qb.push(", ");
        }
        qb.push("universe_name = ");
        qb.push_bind(universe_name);
        first = false;
    }

    if first {
        // no fields were bound
        return Err(StatusCode::BAD_REQUEST);
    }

    // Finish with the WHERE clause
    qb.push(" WHERE id = ");
    qb.push_bind(timeline_id);
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
        Ok(get_by_id(Path(timeline_id), Extension(pool)).await.unwrap())
    }
}
