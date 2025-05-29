use crate::handlers::merges::{create, get_by_id, list};
use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new()
        .route("/timelines_merges", get(list).post(create))
        .route("/timelines_merges/{id}", get(get_by_id))
}
