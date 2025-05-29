use tower::ServiceExt; // for `.oneshot()`
use lore_sharing::db::init_db;
use lore_sharing::routes::users::router;
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
    Extension,
    Json,
    extract::Path
};
use serde_json::{json, Value};
use lore_sharing::handlers::users;

#[tokio::test]
async fn test_list_users_endpoint() {
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

    // 3. Issue a request to GET /users
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. Assert status is 200
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Parse and assert at least one user with nickname "test_user"
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json.as_array().unwrap().iter().any(|u| {
        u.get("nickname").and_then(Value::as_str) == Some("test_user")
    }));
}

#[tokio::test]
async fn test_create_user_endpoint() {
    let pool = init_db().await.expect("failed to init db");

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "nickname": "test_user_create",
        "description": "test description"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create user result: {:?}", response);
    println!("{:?}", response.body());
    println!("Create user status: {:?}", response.status());

    // assert_eq!(response.status(), StatusCode::CREATED); // or 200 if that's what your handler returns
    assert!(matches!(
        response.status(),
        StatusCode::OK | StatusCode::CREATED
    ));

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let user_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", user_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("nickname").and_then(Value::as_str), Some("test_user_create"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("test description"));

    assert!(json.get("id").is_some());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let pool = init_db().await.unwrap();

    let payload = users::PostUser{
        nickname: "test_user_get_by_id".to_string(),
        description: Some("test description".to_owned())
    };

    let new_user = users::create_user(
        Extension(pool.clone()),
        Json(payload)
    )
    .await
    .unwrap()
    .0; // if handler returns Result<Json<User>, _>

    println!("=============== test_create_user_endpoint =======================");
    println!("new user {:?}", new_user);

    let huser = users::get_user_by_id(
        Path(new_user.id),
        Extension(pool.clone())
    )
    .await
    .unwrap()
    .0;
    println!("huser {:?}", huser);

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool.clone()));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/users/{}", new_user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("Create user result: {:?}", response);

    println!("new_user.id: {}", new_user.id);

    println!("{}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", new_user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();


    println!("{}", delete_response.status());
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);


    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.get("nickname").and_then(Value::as_str), Some("test_user_get_by_id"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("test description"));

    // users::delete_user_by_id(Path(new_user.id), Extension(pool.clone()));
}


#[tokio::test]
async fn test_delete_user_by_id() {
    let pool = init_db().await.unwrap();

    let user = users::create_user(
        Extension(pool.clone()),
        Json(users::PostUser {
            nickname: "user_test".to_owned(),
            description: Some("to delete".to_string()),
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
                .uri(format!("/users/{}", user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_put_user_by_id() {
    let pool = init_db().await.unwrap();

    let test_nick_name: String = "user_test_put".to_owned();
    let user = users::create_user(
        Extension(pool.clone()),
        Json(users::PostUser {
            nickname: test_nick_name,
            description: Some("test desc".to_string()),
        }),
    )
    .await
    .unwrap()
    .0;

    let app = Router::new()
        .merge(router())
        .layer(Extension(pool));

    let payload = json!({
        "nickname": "changed",
        "description": "changed"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/users/{}", user.id))
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    println!("put user result: {:?}", response);
    println!("body {:?}", response.body());
    println!("status: {:?}", response.status());

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let user_id = json.get("id").and_then(|v| v.as_i64()).unwrap();
    println!("{}", user_id);
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    assert_eq!(json.get("nickname").and_then(Value::as_str), Some("changed"));
    assert_eq!(json.get("description").and_then(Value::as_str), Some("changed"));
}
