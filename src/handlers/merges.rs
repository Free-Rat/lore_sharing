use std::sync::Arc;
use axum::{
    extract::{ Extension, Path},
    Json,
    http::StatusCode
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use sqlx::{
    SqlitePool,
    Transaction,
    sqlite::SqliteQueryResult,
};
use crate::models::merges::TimelineMerge;

//     pub id: i64,
//     pub source_timeline_id: i64,
//     pub target_timeline_id: i64,
//     pub merged_at: String,
//
// merge mają na celu mergowanie dwóch timelinów 
//
// list merge wypisuje liste mergów
//
// create merge: 
// 1. dodaje eventy z source_timeline do target_timeline
// w przypadku konfliktu:
//    jeśli istnieje już event w target któru jest w source,
//    evetny w target mają pierwszeństwo
//
// 2. usuwa source_timeline
//
// 3. zapisuje merged_at jako czas wykonania requesta
//
// 

pub async fn list(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("debug: list_events_for_timeline");

    let rows = sqlx::query!(
        r#"
        SELECT
            id, 
            source_timeline_id,
            target_timeline_id,
            merged_at AS "merged_at!: String"
        FROM timeline_merges 
        ORDER BY merged_at 
        "#
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during ___: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let merged = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "source_timeline_id": r.source_timeline_id,
                "target_timeline_id": r.target_timeline_id,
                "merged_at": r.merged_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!(merged)))
}

#[derive(Deserialize, Serialize)]
pub struct PostTimelineMerge {
    pub source_timeline_id: i64,
    pub target_timeline_id: i64,
}

pub async fn create(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<PostTimelineMerge>,
) -> Result<(StatusCode, Json<TimelineMerge>), StatusCode> {
    // 1. Start a transaction
    let mut tx: Transaction<'_, _> = pool
        .begin()
        .await
        .map_err(|e| {
            eprintln!("SQL Error during ___: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 2. Copy events from source → target, skipping conflicts
    //    (target events take priority)
    sqlx::query!(
        r#"
        INSERT OR IGNORE INTO timeline_events (timeline_id, event_id, position)
          SELECT ?, event_id, position
            FROM timeline_events
           WHERE timeline_id = ?
        "#,
        payload.target_timeline_id,
        payload.source_timeline_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during ___: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("2. Copy events from source → target, skipping conflicts ");

    // 3. Delete the source timeline’s event links
    sqlx::query!(
        "DELETE FROM timeline_events WHERE timeline_id = ?",
        payload.source_timeline_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during DELETE FROM timeline_events: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("3. Delete the source timeline’s event links");

    // 4. Delete the source timeline itself
    let res: SqliteQueryResult = sqlx::query!(
        "DELETE FROM timelines WHERE id = ?",
        payload.source_timeline_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during DELETE FROM timelines: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    if res.rows_affected() == 0 {
        // nothing to delete → bad source ID
        return Err(StatusCode::NOT_FOUND);
    }
    println!("Delete the source timeline itself");

    // 5. Record the merge
    let merged: TimelineMerge = sqlx::query_as!(
        TimelineMerge,
        r#"
        INSERT INTO timeline_merges (source_timeline_id, target_timeline_id)
        VALUES (?, ?)
        RETURNING
            id AS "id! : i64", 
            source_timeline_id,
            target_timeline_id,
            merged_at AS "merged_at!: String"
        "#,
        payload.source_timeline_id,
        payload.target_timeline_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during INSERT INTO timeline_merges: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("record the merge");

    // 6. Commit
    tx.commit()
        .await
        .map_err(|e| {
            eprintln!("SQL Error during Commit: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    println!("commit");

    Ok((StatusCode::CREATED, Json(merged)))
}

pub async fn get_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<TimelineMerge>, StatusCode> {
    let m = sqlx::query_as!(
        TimelineMerge,
        r#"
        SELECT
            id AS "id! : i64", 
            source_timeline_id,
            target_timeline_id,
            merged_at AS "merged_at!: String"
        FROM timeline_merges
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during ___: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    m.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete(
    Path(id): Path<i64>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, StatusCode> {
    let res = sqlx::query!(
        "DELETE FROM timeline_merges WHERE id = ?",
        id
    )
    .execute(&*pool)
    .await
    .map_err(|e| {
        eprintln!("SQL Error during ___: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if res.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
