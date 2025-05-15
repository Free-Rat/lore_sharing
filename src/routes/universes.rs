use crate::handlers::universes::list_universes;
use axum::routing::get;
use axum::Router;

pub fn universes_router() -> Router {
    Router::new().route("/universes", get(list_universes))
}
