use std::sync::Arc;
use axum::{
    extract::{Extension, Path, Query},
    Json
};
use axum_extra::TypedHeader;
// use std::str::FromStr;
use axum_extra::headers::{
    ETag,
    IfMatch,
    IfNoneMatch
};
use serde_json::json;
use serde::{Serialize,Deserialize};
use sqlx::{SqlitePool, QueryBuilder};
use axum::http::StatusCode;
use crate::models::event::Event;
use http::header::{HeaderMap, HeaderValue};
use crc32fast::Hasher;

// Helper to build an ETag from any serializable value:
fn make_etag<T: Serialize>(value: &T) -> String {
    let s = serde_json::to_string(value).unwrap();
    let mut h = Hasher::new();
    h.update(s.as_bytes());
    format!("\"{:08x}\"", h.finalize())
}
// Instead of returning String, return headers::ETag
// fn make_etag(event: &Event) -> ETag {
//     // Build a proper quoted ETag
//     // Example: let raw = format!("\"{:x}\"", hash_of_event(event));
//     raw.parse::<ETag>()
//         .expect("valid ETag generated")
// }

#[derive(Debug, Deserialize)]
pub struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

pub async fn list(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Query(pagination): Query<Pagination>,
// ) -> Result<Json<serde_json::Value>, StatusCode> {
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), StatusCode> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let per_page_i64 = per_page as i64;
    let offset_i64 = offset as i64;

    let rows = sqlx::query!(
        r#"
        SELECT id, name, description, reference, image, thumbnail, author_id
        FROM events 
        ORDER BY id
        LIMIT ? OFFSET ?
        "#,
        per_page_i64,
        offset_i64
    )
        .fetch_all(&*pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let events_json = rows
        .into_iter()
        .map(|r| {
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

    // build Link header
    let mut link_parts = vec![
        format!("<{}?page={}&per_page={}>; rel=\"self\"", "/events", page, per_page),
    ];
    if page > 1 {
        link_parts.push(format!("<{}?page={}&per_page={}>; rel=\"prev\"", "/events", page-1, per_page));
    }
    // you could detect if there's a next page by comparing rows.len() == per_page
    if (events_json.len() as u32) == per_page {
        link_parts.push(format!("<{}?page={}&per_page={}>; rel=\"next\"", "/events", page+1, per_page));
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::LINK,
        HeaderValue::from_str(&link_parts.join(", ")).unwrap(),
    );

    // Ok(Json(json!(events_json)))
    Ok((StatusCode::OK, headers, Json(json!(events_json))))
}

// pub async fn list(
//     Extension(pool): Extension<Arc<SqlitePool>>,
// ) -> Result<Json<serde_json::Value>, StatusCode> {
//     let rows = sqlx::query!(
//         r#"
//         SELECT id, name, description, reference, image, thumbnail, author_id
//         FROM events 
//         ORDER BY id
//         "#
//     )
//     .fetch_all(&*pool)
//     .await
//     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//
//     let events_json = rows
//         .into_iter()
//         .map(|r| {
//             // println!("id:{} nick:{} desc:{:?}", r.id, r.nickname, r.description);
//             json!({
//                 "id": r.id,
//                 "name": r.name,
//                 "description": r.description,
//                 "reference": r.reference,
//                 "image": r.image,
//                 "thumbnail": r.thumbnail,
//                 "author_id": r.author_id
//             })
//         })
//         .collect::<Vec<_>>();
//
//     Ok(Json(json!(events_json)))
// }


#[derive(Deserialize)]
pub struct PostEvent {
    pub name: String,
    pub description: String,
    pub reference: String,
    pub image: Option<String>,     // URL or path
    pub thumbnail: Option<String>, // URL or path
    pub author_id: i64,
}

pub async fn create(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<PostEvent>,
// ) -> Result<Json<Event>, StatusCode> {
) -> Result<(StatusCode, HeaderMap, Json<Event>), StatusCode> {
    let result = sqlx::query_as!(
        Event,
        r#"
        INSERT INTO events (name, description, reference, image, thumbnail, author_id)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING 
            id AS "id!: i64",
            name,
            description,
            reference,
            image,
            thumbnail,
            author_id AS "author_id!: i64"
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

    // Ok(Json(resul

    // build Location
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::LOCATION,
        HeaderValue::from_str(&format!("/events/{}", result.id)).unwrap(),
    );

    //Compute a strong ETag for the newly-created resource
    let etag_str = make_etag(&result);
    headers.insert(
        http::header::ETAG,
        HeaderValue::from_str(&etag_str).unwrap(),
    );

    Ok((StatusCode::CREATED, headers, Json(result)))
}

pub async fn get_by_id(
    Path(id): Path<i64>,
    maybe_if_none: Option<TypedHeader<IfNoneMatch>>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<(StatusCode, HeaderMap, Json<Event>), StatusCode> {
    // 1) Load event
    let event = match sqlx::query_as!(
        Event,
        "SELECT id, name, description, reference, image, thumbnail, author_id FROM events WHERE id = ?",
        id
    )
    .fetch_optional(&*pool)
    .await
    {
        Ok(Some(e)) => e,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 2) Compute a *String* ETag (including quotes and optional `W/`)
    let etag_str = make_etag(&event);

    // 3) Build a typed ETag only for comparison
    let etag = etag_str
        .parse::<ETag>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4) Prepare your headers
    let header_value = HeaderValue::from_str(&etag_str)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::ETAG, header_value.clone());

    // 5) If-None-Match present *and* matches → 304 Not Modified
    if let Some(TypedHeader(if_none)) = maybe_if_none {
        // `precondition_passes()` is false when the client’s ETag *does* match ours
        if !if_none.precondition_passes(&etag) {
            return Ok((StatusCode::NOT_MODIFIED, headers, Json(event)));
        }
    }

    // 6) Otherwise send 200 + body
    Ok((StatusCode::OK, headers, Json(event)))
}

// pub async fn get_by_id(
//     Path(id): Path<i64>,
//     Extension(pool): Extension<Arc<SqlitePool>>,
// ) -> Result<Json<Event>, StatusCode> {
//     let result = sqlx::query_as!(
//         Event,
//         "SELECT id, name, description, reference, image, thumbnail, author_id FROM events WHERE id = ?",
//         id
//     )
//     .fetch_optional(&*pool)
//     .await
//     .map_err(|e| {
//          eprintln!("get events by id DB error: {:?}", e);
//          StatusCode::INTERNAL_SERVER_ERROR
//     })?;
//
//     match result {
//         Some(event) => Ok(Json(event)),
//         None => Err(StatusCode::NOT_FOUND),
//     }
// }

pub async fn delete_by_id(
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

#[derive(Deserialize, Debug)]
pub struct UpdateEvent {
    pub name: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub image: Option<String>,     // URL or path
    pub thumbnail: Option<String>, // URL or path
    pub author_id: i64,
}

pub async fn update(
    Path(event_id): Path<i64>,
    maybe_if_match: Option<TypedHeader<IfMatch>>,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(payload): Json<UpdateEvent>,
// ) -> Result<Json<Event>, StatusCode> {
) -> Result<(StatusCode, HeaderMap, Json<Event>), StatusCode> {

    // Load current event
    let existing: Event = sqlx::query_as!(
        Event,
        "SELECT id, name, description, reference, image, thumbnail, author_id \
         FROM events WHERE id = ?",
        event_id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Compute current ETag
    let current_etag_str = make_etag(&existing);
    let current_etag: ETag = current_etag_str
        .parse()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("{:?}", maybe_if_match);
    println!("{:?}", current_etag);

    // Enforce If-Match: fail if client’s version doesn’t match
    if let Some(TypedHeader(if_match)) = maybe_if_match {
        // precondition_passes() == true → header absent or matches
        // we want to **reject** when it’s present AND doesn’t match
        if !if_match.precondition_passes(&current_etag) {
            return Err(StatusCode::PRECONDITION_FAILED);
        }
    }


    // Start building the query
    let mut qb = QueryBuilder::<sqlx::Sqlite>::new("UPDATE events SET ");
    // println!("{}", event_id);
    // println!("{:?}", payload);

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


    // 5) Re-load the updated event
    let updated: Event = sqlx::query_as!(
        Event,
        "SELECT id, name, description, reference, image, thumbnail, author_id \
         FROM events WHERE id = ?",
        event_id
    )
    .fetch_one(&*pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6) Compute new ETag and build headers
    let new_etag_str = make_etag(&updated);
    let header_value = HeaderValue::from_str(&new_etag_str)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::ETAG, header_value);

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        // 7) Return 200 OK + ETag + updated body
        Ok((StatusCode::OK, headers, Json(updated)))
        // Ok(get_by_id(Path(event_id), None, Extension(pool)).await.unwrap())
    }
}
