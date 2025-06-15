use tower::ServiceExt; // for `.oneshot()`
use lore_sharing::db::init_db;
use lore_sharing::routes::events::router;
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
    Extension,
    Json,
    extract::Path,
    // response::IntoResponse
};
use serde_json::{json, Value};
use lore_sharing::handlers::events;

#[tokio::test]
async fn test_list_endpoint() {
    // 1. Init a fresh in-memory SQLite for test isolation
    // let pool = {
    //     // std::env::set_var("DATABASE_URL", "sqlite::memory:");
    //     let p = init_db().await.expect("failed init_db");
    //     p
    // };
    let pool = init_db().await.expect("failed init_db");


    // 2. Build our app with the test pool
    let app: Router = Router::new()
        .merge(router())
        .layer(Extension(pool));

    // 3. Issue a request to GET /events
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. Assert status is 200
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Parse and assert at least one events with name "test_events"
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.as_array().unwrap().iter().any(|u| {
        println!("{:?}",u.get("name").and_then(Value::as_str));
        u.get("name").and_then(Value::as_str) == Some("Test Event")
    }));
}

#[tokio::test]
async fn test_create_endpoint() {
    let pool = init_db().await.expect("failed to init db");

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "name": "test_event_create",
        "description": "test description",
        "reference": "test referance",
        "image": None::<String>,
        "author_id": 1,
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create event result: {:?}", response);
    println!("{:?}", response.body());
    println!("Create event status: {:?}", response.status());

    // assert_eq!(response.status(), StatusCode::CREATED); // or 200 if that's what your handler returns
    assert!(matches!(
        response.status(),
        StatusCode::OK | StatusCode::CREATED
    ));

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let event_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", event_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("name").and_then(Value::as_str), Some("test_event_create"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("test description"));

    assert!(json.get("id").is_some());
}

#[tokio::test]
async fn test_get_by_id() {
    let pool = init_db().await.unwrap();

    let payload = events::PostEvent{
        name: "test_event_get_by_id".to_string(),
        description: "test description".to_owned(),
        reference: "test referance".to_owned(),
        image: None::<String>,
        thumbnail: None,
        author_id: 1,
    };

    // let new_event = events::create(
    //     Extension(pool.clone()),
    //     Json(payload)
    // )
    // .await
    // .unwrap()
    // .0; // if handler returns Result<Json<User>, _>
    
    let (_status, _headers, Json(new_event)) = events::create(
        Extension(pool.clone()),
        Json(payload)
    )
    .await
    .unwrap();

    println!("=============== test_get_endpoint =======================");
    println!("new event {:?}", new_event);

    let hevent = events::get_by_id(
        Path(new_event.id),
        None,
        Extension(pool.clone())
    )
    .await
    .unwrap()
    .0;
    println!("hevent {:?}", hevent);

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool.clone()));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/events/{}", new_event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create event result: {:?}", response);

    println!("new_event.id: {}", new_event.id);

    println!("{}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/events/{}", new_event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();


    println!("{}", delete_response.status());
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);


    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.get("name").and_then(Value::as_str), Some("test_event_get_by_id"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("test description"));
}


#[tokio::test]
async fn test_delete_by_id() {
    let pool = init_db().await.unwrap();


    let (_status, _headers, Json(event)) = events::create(
        Extension(pool.clone()),
        Json(events::PostEvent{
            name: "test_event_delete_by_id".to_string(),
            description: "test description".to_owned(),
            reference: "test referance".to_owned(),
            image: None::<String>,
            thumbnail: None,
            author_id: 1,
        }),
    )
    .await
    .unwrap();

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/events/{}", event.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_put_by_id() {
    let pool = init_db().await.unwrap();

    let (_status, headers, Json(event)) = events::create(
        Extension(pool.clone()),
        Json(events::PostEvent{
            name: "test_event_put".to_string(),
            description: "test description".to_owned(),
            reference: "test referance".to_owned(),
            image: None::<String>,
            thumbnail: None,
            author_id: 1,
        }),
    )
    .await
    .unwrap();

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "name": "changed",
        "description": "changed",
        "author_id": 1
    });

    // Extract the ETag string (should be present)
    let etag_value = headers
        .get(axum::http::header::ETAG)
        .expect("create must return ETag")
        .to_str()
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/events/{}", event.id))
                .header("Content-Type", "application/json")
                .header("If-Match", etag_value)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    println!("put event result: {:?}", response);
    println!("body {:?}", response.body());
    println!("status: {:?}", response.status());

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let event_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", event_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("name").and_then(Value::as_str), Some("changed"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("changed"));
}
