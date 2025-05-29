use tower::ServiceExt; // for `.oneshot()`
use lore_sharing::db::init_db;
use lore_sharing::routes::universes::router;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
    Extension,
};
use serde_json::Value;
// use lore_sharing::handlers::universes;
// use lore_sharing::models::

#[tokio::test]
async fn test_list_users_endpoint() {
    let pool = init_db().await.expect("failed init_db");
    let app: Router = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/universes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.as_array().unwrap().iter().any(|u| {
        u.get("name").and_then(Value::as_str) == Some("Elden Ring")
    }));
}
