use crate::handlers::users::{
    create_user, delete_user_by_id, get_user_by_id, list_users, update_user,
};
use axum::routing::get;
use axum::Router;

pub fn users_router() -> Router {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user_by_id)
                .delete(delete_user_by_id)
                .put(update_user),
        )
}
