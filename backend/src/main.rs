use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;

mod db;
mod routes;

use routes::{admin, assessments, auth, competencies, courses, publishing, resources, users};

fn app() -> Router {
    Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/users", users::router())
        .nest("/api/courses", courses::router())
        .nest("/api/competencies", competencies::router())
        .nest("/api/assessments", assessments::router())
        .nest("/api/resources", resources::router())
        .nest("/api/publishing", publishing::router())
        .nest("/api/admin", admin::router())
        .route("/", get(home))
        .layer(CorsLayer::very_permissive())
}

#[tokio::main]
async fn main() {
    db::init().expect("failed to initialize database");

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "6969".to_string());
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap();

    println!("Server on: http://{address}/");
    axum::serve(listener, app()).await.unwrap();
}

async fn home() -> &'static str {
    "Capacity Connect API - OK"
}

#[cfg(test)]
mod tests;
