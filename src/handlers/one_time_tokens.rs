use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use axum::http::HeaderMap;
use axum::http::header::{HeaderName, HeaderValue};
use http::StatusCode;
use sqlx::SqlitePool;
use serde_json;
use axum::{
    extract::Extension,
    response::IntoResponse,
    Json
};
use serde_json::json;

/// Try to consume the token, or if already used, replay the stored response.
pub async fn consume_or_replay(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<(StatusCode, HeaderMap, Vec<u8>)>, StatusCode> {
    let row = sqlx::query!(
        r#"
        SELECT used, status_code, response_body, response_headers
          FROM one_time_tokens
         WHERE token = ?
        "#, 
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::UNAUTHORIZED)?;

    if row.used {
        // replay
        let mut headers = HeaderMap::new();
        if let Some(hdr_json) = row.response_headers {
            let map: HashMap<String, String> = serde_json::from_str(&hdr_json)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            for (k, v) in map {
                let name: HeaderName = k.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let value: HeaderValue = HeaderValue::from_str(&v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                headers.insert(name, value);
            }
        }
        let body = row.response_body.unwrap_or_default();
        let status = StatusCode::from_u16(row.status_code.unwrap_or(200) as u16)
            .unwrap_or(StatusCode::OK);
        return Ok(Some((status, headers, body)));
    }

    // mark as used
    let result = sqlx::query!(
        "UPDATE one_time_tokens SET used = TRUE WHERE token = ? AND used = FALSE",
        token
    )
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        // race: somebody else used it first
        return Err(StatusCode::CONFLICT);
    }

    Ok(None)
}

/// After generating the response, save it so future calls can replay.
pub async fn store_response_for_token(
    pool: &SqlitePool,
    token: &str,
    status: StatusCode,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), StatusCode> {
    let hdr_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
        .collect();
    let hdr_json = serde_json::to_string(&hdr_map)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ensure status lives long enough
    let status_i64 = status.as_u16() as i64;
    let response_body = body;
    let response_headers = &hdr_json;

    sqlx::query!(
        r#"
        UPDATE one_time_tokens
           SET status_code      = ?,
               response_body    = ?,
               response_headers = ?
         WHERE token = ?
        "#, 
        status_i64,
        response_body,
        response_headers,
        token
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Issue a new one-time token and store it unused. Returns JSON {"token": "..."} as response.
pub async fn issue_token(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> impl IntoResponse {
    // generate a new UUID token
    let token = Uuid::new_v4().to_string();

    // insert into DB as unused
    let res = sqlx::query!(
        "INSERT INTO one_time_tokens (token, used) VALUES (?, FALSE)",
        token
    )
    .execute(&*pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, Json(json!({"token": token}))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "could not issue token"}))),
    }
}
