use tower::ServiceExt; // for `.oneshot()`
use lore_sharing::db::init_db;
use lore_sharing::routes::timelines::router;
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
    Extension,
    Json,
    extract::Path
};
use serde_json::{json, Value};
use lore_sharing::handlers::timelines;

#[tokio::test]
async fn test_list_endpoint() {
    let pool = init_db().await.expect("failed init_db");
    let app: Router = Router::new()
        .merge(router())
        .layer(Extension(pool));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/timelines")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.as_array().unwrap().iter().any(|u| {
        println!("{:?}",u.get("description").and_then(Value::as_str));
        u.get("description").and_then(Value::as_str) == Some("Test Timeline")
    }));
}

#[tokio::test]
async fn test_create_endpoint() {
    let pool = init_db().await.expect("failed to init db");

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "author_id":    1,
        "description":  "The story of Man",
        "start":        0,
        "end":          2025,
        "unit":         "years",
        "universe_name":"Lord of the Rings"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/timelines")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create result: {:?}", response);
    println!("{:?}", response.body());
    println!("Create status: {:?}", response.status());

    // assert_eq!(response.status(), StatusCode::CREATED); // or 200 if that's what your handler returns
    assert!(matches!(
        response.status(),
        StatusCode::OK | StatusCode::CREATED
    ));

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let timeline_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", timeline_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/timelines/{}", timeline_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("description").and_then(Value::as_str), Some("The story of Man"));

    assert!(json.get("id").is_some());
}

#[tokio::test]
async fn test_get_by_id() {
    let pool = init_db().await.unwrap();

    let payload = timelines::PostTimeline {
        description: "Story of Fallen Leafs".to_owned(),
        author_id: 1,
        start: 0,
        end: 2025,
        unit: "years".to_owned(),
        universe_name: "Elden Ring".to_owned(),
    };

    let new_timeline = timelines::create(
        Extension(pool.clone()),
        Json(payload)
    )
    .await
    .unwrap()
    .0; // if handler returns Result<Json<User>, _>

    println!("=============== test_get_endpoint =======================");
    println!("new {:?}", new_timeline);

    let htimeline = timelines::get_by_id(
        Path(new_timeline.id),
        Extension(pool.clone())
    )
    .await
    .unwrap()
    .0;
    println!("home value {:?}", htimeline);

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool.clone()));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/timelines/{}", new_timeline.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create result: {:?}", response);

    println!("new.id: {}", new_timeline.id);

    println!("{}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/timelines/{}", new_timeline.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();


    println!("{}", delete_response.status());
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);


    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.get("description").and_then(Value::as_str), Some("Story of Fallen Leafs"));
}


#[tokio::test]
async fn test_delete_by_id() {
    let pool = init_db().await.unwrap();

    let timeline = timelines::create(
        Extension(pool.clone()),
        Json(timelines::PostTimeline {
            description: "Story of Fallen Leafs".to_owned(),
            author_id: 1,
            start: 0,
            end: 2025,
            unit: "years".to_owned(),
            universe_name: "Elden Ring".to_owned(),
        }),
    )
    .await
    .unwrap()
    .0;

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/timelines/{}", timeline.id))
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

    let timeline = timelines::create(
        Extension(pool.clone()),
        Json(timelines::PostTimeline {
            description: "Story of Fallen Leafs".to_owned(),
            author_id: 1,
            start: 0,
            end: 2025,
            unit: "years".to_owned(),
            universe_name: "Elden Ring".to_owned(),
        }),
    )
    .await
    .unwrap()
    .0;

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "description": "changed",
        "start": -100,
        "end": 1000,
        "author_id": 1
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/timelines/{}", timeline.id))
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    println!("put result: {:?}", response);
    println!("body {:?}", response.body());
    println!("status: {:?}", response.status());

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let timeline_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", timeline_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/timelines/{}", timeline_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("description").and_then(Value::as_str), Some("changed"));
    assert_eq!(json.get("start").and_then(Value::as_i64), Some(-100));
    assert_eq!(json.get("end").and_then(Value::as_i64), Some(1000));
}
