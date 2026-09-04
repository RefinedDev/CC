use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ApprovalRequest {
    pub user_id: String,
    pub action: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/users", get(list_all_users))
        .route("/users/{id}/approve", post(approve_user))
        .route("/users/{id}/reject", post(reject_user))
        .route("/dashboard", get(get_dashboard))
        .route("/courses/{id}/approve", post(approve_course))
        .route("/notifications", post(send_notification))
}

async fn list_all_users() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": "List all users endpoint - TODO" })),
    )
}

async fn approve_user(Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("Approve user {} endpoint - TODO", id) })),
    )
}

async fn reject_user(Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("Reject user {} endpoint - TODO", id) })),
    )
}

async fn get_dashboard() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Get admin dashboard endpoint - TODO" })),
    )
}

async fn approve_course(Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("Approve course {} endpoint - TODO", id) })),
    )
}

async fn send_notification(
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Send notification endpoint - TODO" })),
    )
}
