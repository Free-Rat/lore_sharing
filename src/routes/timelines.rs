use crate::handlers::timeline_events;
use crate::handlers::timelines::{create, delete_by_id, get_by_id, list, update};
use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new()
        .route("/timelines", get(list).post(create))
        .route(
            "/timelines/{id}",
            get(get_by_id).delete(delete_by_id).put(update),
        )
        .route(
            "/timelines/{timeline_id}/events",
            get(timeline_events::list_events_for_timeline).post(timeline_events::create),
        )
        .route(
            "/timelines/{id}/events/{id}",
            get(timeline_events::get_by_id)
                .put(timeline_events::update)
                .delete(timeline_events::delete_by_id),
        )
}
