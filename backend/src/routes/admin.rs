use crate::routes::auth;
use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/users", get(list_all_users))
        .route("/users/{id}/role", post(update_role))
        .route("/stats", get(get_stats))
        .route("/notifications", post(send_notification))
}

fn require_admin(
    headers: &HeaderMap,
) -> Result<auth::Claims, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "message": "Administrator access is required." })),
        ));
    }
    Ok(claims)
}

async fn get_stats(
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&headers)?;
    let (users, courses, enrollments, trainees, trainers, admins) = crate::db::dashboard_counts()
        .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load dashboard statistics." })),
        )
    })?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "users": users, "courses": courses, "enrollments": enrollments,
            "certificates": 0, "trainees": trainees, "trainers": trainers, "admins": admins
        })),
    ))
}

async fn list_all_users(
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&headers)?;
    let users = crate::db::list_users()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load users." })),
            )
        })?
        .into_iter()
        .map(|user| {
            serde_json::json!({
                "id": user.id, "name": user.name, "email": user.email, "role": user.role
            })
        })
        .collect::<Vec<_>>();
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
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Role must be trainee, trainer, or admin." })),
        ));
    }
    let mut user = crate::db::find_user_by_id(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load user." })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "message": "User not found." })),
            )
        })?;
    user.role = role;
    crate::db::update_user(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to update role." })),
        )
    })?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": user.id, "name": user.name, "email": user.email, "role": user.role
        })),
    ))
}

async fn send_notification(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = require_admin(&headers)?;
    let kind = payload
        .get("kind")
        .and_then(|value| value.as_str())
        .unwrap_or("notification");
    let title = payload
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();
    let body = payload
        .get("body")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();
    if title.is_empty() || body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Title and body are required." })),
        ));
    }
    let target_user_id = payload.get("target_user_id").and_then(|value| value.as_str()).map(str::trim).filter(|value| !value.is_empty());
    if let Some(user_id) = target_user_id {
        if crate::db::find_user_by_id(user_id).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "message": "Failed to validate target user." }))))?.is_none() {
            return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "message": "Target user was not found." }))));
        }
    }
    let item = crate::db::insert_publication(kind, title, body, &claims.sub, target_user_id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to publish." })),
        )
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::json!(item))))
}
