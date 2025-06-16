use tower::ServiceExt; // for `.oneshot()`
use lore_sharing::db::init_db;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    Router,
    Extension,
    Json,
    // extract::Path
};
use serde_json::{json, Value};

use lore_sharing::routes::{
    events::router     as events_router,
    timelines::router  as timelines_router,
    one_time_tokens::router as token_router,
};
use lore_sharing::handlers::{
    events::{
        // create as create_event,
        PostEvent
    },
    timelines::{create as create_timeline, PostTimeline},
    timeline_events::{PostTimelineEvent, UpdateTimelineEvent},
};
use lore_sharing::models::event::Event;

#[tokio::test]
async fn timeline_events_crud_flow() {
    // —— 1. Init DB & app
    let pool = init_db().await.expect("DB init failed");
    let app = Router::new()
        .merge(events_router())
        .merge(timelines_router())
        .merge(token_router()) // add router for issuing tokens
        .layer(Extension(pool.clone()));

    let token_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/one_time_tokens")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(token_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(token_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let token_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = token_json
        .get("token")
        .and_then(Value::as_str)
        .expect("token must be a string");
    let auth_header = format!("Bearer {}", token);

    // —— 2. Create a Universe
    let uni_test = "Elden Ring".to_owned();

    // —— 3. Create an Event
    let evt_payload = PostEvent {
        name:        "evt1".into(),
        description: "an event".into(),
        reference:   "ref1".into(),
        image:       None,
        thumbnail:   None,
        author_id:   1,  // seeded test_user
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("Authorization", auth_header.clone())
                .body(Body::from(json!(evt_payload).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let evt: Event = serde_json::from_slice(&body).unwrap();

    // —— 4. Create a Timeline
    let tl_payload = PostTimeline {
        author_id:     1,
        description:   "a timeline".into(),
        start:         0,
        end:           100,
        unit:          "hours".into(),
        universe_name: uni_test.clone(),
    };
    let Json(tl) = create_timeline(Extension(pool.clone()), Json(tl_payload))
        .await
        .expect("create_timeline failed");

    // —— 5. POST /timelines/{id}/events  (create link)
    let te_payload = PostTimelineEvent {
        event_id: evt.id,
        position: 5,
    };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/timelines/{}/events", tl.id))
                .header("Content-Type", "application/json")
                .body(Body::from(json!(te_payload).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(matches!(response.status(), StatusCode::OK | StatusCode::CREATED), "got {}",response.status());
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let te_created: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(te_created["timeline_id"].as_i64(), Some(tl.id));
    assert_eq!(te_created["event_id"].as_i64(), Some(evt.id));
    assert_eq!(te_created["position"].as_i64(), Some(5));

    // —— 6. GET /timelines/{id}/events  (list events)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/timelines/{}/events", tl.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("6. GET /timelines/id/events  (list events) {:?}", response.body());
    assert_eq!(response.status(), StatusCode::OK, "get timelines got {}",response.status());
    let arr: Vec<Value> =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(arr.iter().any(|e| e["event_id"].as_i64() == Some(evt.id)));

    // —— 7. GET /timelines/{tid}/events/{eid}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/timelines/{}/events/{}", tl.id, evt.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("7. GET /timelines/tid/events/eid {:?}", response);
    assert_eq!(response.status(), StatusCode::OK, "7. got {}", response.status());
    let single: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(single["position"].as_i64(), Some(5));

    // —— 8. PUT /timelines/{tid}/events/{eid}  (update position)
    let upd_payload = UpdateTimelineEvent { position: Some(10) };
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/timelines/{}/events/{}", tl.id, evt.id))
                .header("Content-Type", "application/json")
                .body(Body::from(json!(upd_payload).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    println!("8. PUT /timelines/tid/events/eid  (update position) {:?}", response);
    assert_eq!(response.status(), StatusCode::OK);
    let updated: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(updated["position"].as_i64(), Some(10));

    // —— 9. DELETE /timelines/{tid}/events/{eid}
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/timelines/{}/events/{}", tl.id, evt.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("9. DELETE /timelines/tid/events/eid {:?}", response);
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // —— 10. Ensure it’s gone
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/timelines/{}/events/{}", tl.id, evt.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    println!("10. Ensure it’s gone {:?}", response);
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
