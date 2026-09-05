use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use crate::routes::auth;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ApprovalRequest {
    pub user_id: String,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/users", get(list_all_users))
        .route("/users/{id}/role", post(update_role))
        .route("/stats", get(get_stats))
        .route("/users/{id}/approve", post(approve_user))
        .route("/users/{id}/reject", post(reject_user))
        .route("/dashboard", get(get_dashboard))
        .route("/courses/{id}/approve", post(approve_course))
        .route("/notifications", post(send_notification))
}

async fn get_stats(headers: HeaderMap) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&headers)?;
    let (users, courses, enrollments, trainees, trainers, admins) = crate::db::dashboard_counts().map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load dashboard statistics." })),
    ))?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "users": users, "courses": courses, "enrollments": enrollments,
        "certificates": 0, "trainees": trainees, "trainers": trainers, "admins": admins
    }))))
}

async fn list_all_users(headers: HeaderMap) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&headers)?;
    let users = crate::db::list_users().map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load users." })),
    ))?;
    let users = users.into_iter().map(|user| serde_json::json!({
        "id": user.id, "name": user.name, "email": user.email, "role": user.role
    })).collect::<Vec<_>>();
    Ok((StatusCode::OK, Json(serde_json::json!(users))))
}

async fn update_role(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&headers)?;
    let role = payload.role.trim().to_ascii_lowercase();
    if !matches!(role.as_str(), "trainee" | "trainer" | "admin") {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "message": "Role must be trainee, trainer, or admin."
        }))));
    }
    let mut user = crate::db::find_user_by_id(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load user." })),
    ))?.ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({
        "message": "User not found."
    }))))?;
    user.role = role;
    crate::db::update_user(&user).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to update role." })),
    ))?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "id": user.id, "name": user.name, "email": user.email, "role": user.role
    }))))
}

fn require_admin(headers: &HeaderMap) -> Result<auth::Claims, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(headers).map_err(|_| (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
    ))?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({
            "message": "Administrator access is required."
        }))));
    }
    Ok(claims)
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
