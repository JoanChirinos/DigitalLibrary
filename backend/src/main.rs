mod auth;
mod db;
mod models;
mod schema;
mod routes;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use std::path::PathBuf;

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

    // Check if frontend dist exists
    let frontend_path = PathBuf::from("../frontend/dist");
    let serve_frontend = frontend_path.exists();

    let mut app = Router::new()
        .nest("/api/auth", routes::auth::router())
        .nest("/api/books", routes::books::router())
        .nest("/api/tags", routes::tags::router())
        .nest("/api/stats", routes::stats::router())
        .layer(cors)
        .with_state((pool, token_store));

    if serve_frontend {
        println!("Serving frontend from ../frontend/dist");
        app = app.fallback_service(ServeDir::new(frontend_path));
    } else {
        println!("Frontend not built. Run 'npm run build' in frontend/");
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8008").await.unwrap();
    println!("Server running on http://localhost:8008");
    axum::serve(listener, app).await.unwrap();
}
