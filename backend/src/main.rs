mod db;
mod models;
mod schema;
mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let pool = db::establish_pool();
    db::initialize_db(&pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/books", routes::books::router())
        .nest("/tags", routes::tags::router())
        .nest("/stats", routes::stats::router())
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8008").await.unwrap();
    println!("Server running on http://localhost:8008");
    axum::serve(listener, app).await.unwrap();
}
