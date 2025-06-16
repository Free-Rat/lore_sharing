use crate::handlers::one_time_tokens::issue_token;
use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new().route("/one_time_tokens", get(issue_token))
}
