use axum::{
    routing::get,
    Router,
    // Extension,
    extract::Extension
};
use lore_sharing::db::init_db;
use lore_sharing::routes::{
    users,
    universes,
    events,
    timelines,
    merges,
    one_time_tokens,
};

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();
    let pool = init_db().await.expect("DB init failed");
    println!("Database connected and migrations applied.");

    // build our application with a single route
    //
    // API:
    //  /users
    //  /universes
    //  /events
    //  /timelines
    //  /timelines/{}/branches/
    //  /timeline_merges
    //
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/status", get(unit_handler))
        .merge(users::router())
        .merge(universes::router())
        .merge(events::router())
        .merge(timelines::router())
        .merge(merges::router())
        .merge(one_time_tokens::router())
        .layer(Extension(pool))
        ;

    async fn unit_handler() {}

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
