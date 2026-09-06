use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;

mod db;
mod routes;

use routes::{admin, assessments, auth, courses, publishing, resources, users};

#[tokio::main]
async fn main() {
    db::init().expect("failed to initialize database");
    let app = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/users", users::router())
        .nest("/api/courses", courses::router())
        .nest("/api/assessments", assessments::router())
        .nest("/api/resources", resources::router())
        .nest("/api/publishing", publishing::router())
        .nest("/api/admin", admin::router())
        .route("/", get(home))
        .layer(CorsLayer::very_permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:6969")
        .await
        .unwrap();

    println!("Server on: http://localhost:6969/");
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> &'static str {
    "Capacity Connect API - OK"
}
