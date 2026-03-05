mod auth;
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

    let token_store = auth::create_token_store();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let app = Router::new()
        .nest("/auth", routes::auth::router())
        .nest("/books", routes::books::router())
        .nest("/tags", routes::tags::router())
        .nest("/stats", routes::stats::router())
        .layer(cors)
        .with_state((pool, token_store));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8008").await.unwrap();
    println!("Server running on http://localhost:8008");
    axum::serve(listener, app).await.unwrap();
}
