use crate::handlers::events::{create, delete_by_id, get_by_id, list, update};
use crate::handlers::timeline_events;
use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new()
        .route("/events", get(list).post(create))
        .route(
            "/events/{id}",
            get(get_by_id).delete(delete_by_id).put(update),
        )
        .route(
            "/events/{id}/timelines/{id}",
            get(timeline_events::get_by_id)
                .put(timeline_events::update)
                .delete(timeline_events::delete_by_id),
        )
        .route(
            "/events/{id}/timelines",
            get(timeline_events::list_timelines_for_event),
        )
}
