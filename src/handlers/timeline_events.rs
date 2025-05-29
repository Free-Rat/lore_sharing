use std::sync::Arc;
use axum::{extract::{Extension, Path}, Json};
use serde_json::json;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, QueryBuilder};
use axum::http::StatusCode;
use crate::models::timeline_event::TimelineEvent;

    // pub timeline_id: i64,
    // pub event_id: i64,
    // pub position: u64,
//
pub async fn list_events_for_timeline(
    Path(timeline_id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("debug: list_events_for_timeline");

    let rows = sqlx::query!(
        r#"
        SELECT timeline_id, event_id, position 
        FROM timeline_events 
        WHERE timeline_id = ?
        ORDER BY position
        "#,
        timeline_id
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let timeline_json = rows
        .into_iter()
        .map(|r| {
            json!({
                "timeline_id": r.timeline_id,
                "event_id": r.event_id,
                "position": r.position,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(timeline_json)))
}

pub async fn list_timelines_for_event(
    Path(event_id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query!(
        r#"
        SELECT timeline_id, event_id, position 
        FROM timeline_events 
        WHERE event_id = ?
        ORDER BY position
        "#,
        event_id
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let timeline_json = rows
        .into_iter()
        .map(|r| {
            json!({
                "timeline_id": r.timeline_id,
                "event_id": r.event_id,
                "position": r.position,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(timeline_json)))
}

#[derive(Deserialize, Serialize)]
pub struct PostTimelineEvent {
    pub event_id: i64,     
    pub position: i64,     
}

pub async fn create(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Path(timeline_id): Path<i64>,
    Json(payload): Json<PostTimelineEvent>,
) -> Result<Json<TimelineEvent>, StatusCode> {
    println!("create timeline event");

    // First: Try to fetch the existing row
    if let Ok(existing) = sqlx::query_as!(
        TimelineEvent,
        r#"
        SELECT timeline_id, event_id, position
        FROM timeline_events
        WHERE timeline_id = ? AND event_id = ?
        "#,
        timeline_id,
        payload.event_id
    )
    .fetch_one(&*pool)
    .await
    {
        println!("Already exists: {:?}", existing);
        return Ok(Json(existing));
    }

    // Not found: Insert the new row
    let result = sqlx::query_as!(
        TimelineEvent,
        r#"
        INSERT INTO timeline_events (timeline_id, event_id, position)
        VALUES (?, ?, ?)
        RETURNING timeline_id, event_id, position
        "#,
        timeline_id,
        payload.event_id,
        payload.position
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        eprintln!("create timeline DB error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    println!("Inserted: {:?}", result);
    Ok(Json(result))
}

pub async fn get_by_id(
    Path((timeline_id, event_id)): Path<(i64, i64)>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<TimelineEvent>, StatusCode> {
    let result = sqlx::query_as!(
        TimelineEvent,
        "SELECT timeline_id, event_id, position FROM timeline_events WHERE timeline_id = ? AND event_id = ? ",
        timeline_id,
        event_id
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
    Path((timeline_id, event_id)): Path<(i64, i64)>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM timeline_events WHERE timeline_id = ? AND event_id = ?", 
        timeline_id,
        event_id
    )
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

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateTimelineEvent {
    pub position: Option<i64>,     
}

pub async fn update(
    Path((timeline_id, event_id)): Path<(i64, i64)>,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<UpdateTimelineEvent>,
) -> Result<Json<TimelineEvent>, StatusCode> {

    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("UPDATE timeline_events SET ");

    let mut first = true;

    if let Some(position) = payload.position {
        if !first {
            qb.push(", ");
        }
        qb.push("position = ");
        qb.push_bind(position);
        first = false;
    }

    if first {
        // no fields were bound
        return Err(StatusCode::BAD_REQUEST);
    }

    // Finish with the WHERE clause
    qb.push(" WHERE timeline_id = ");
    qb.push_bind(timeline_id);
    qb.push(" AND event_id = ");
    qb.push_bind(event_id);

    // Build and execute
    let result = qb
        .build()
        .execute(&*pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(get_by_id(Path((timeline_id,event_id)),  Extension(pool)).await.unwrap())
    }
}
