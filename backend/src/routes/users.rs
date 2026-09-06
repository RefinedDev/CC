use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};

use crate::routes::auth;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me", put(update_profile))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}", delete(delete_user))
}

async fn get_profile(
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let user = crate::db::find_user_by_id(&claims.sub)
        .ok()
        .flatten()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "message": "User could not be found for this token." })),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
        })),
    ))
}

async fn update_profile(
    headers: HeaderMap,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let mut user = crate::db::find_user_by_id(&claims.sub)
        .ok()
        .flatten()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "message": "User could not be found for this token." })),
            )
        })?;

    if let Some(name) = payload.name.filter(|value| !value.trim().is_empty()) {
        user.name = name.trim().to_string();
    }

    if let Some(email) = payload.email.filter(|value| !value.trim().is_empty()) {
        user.email = email.trim().to_lowercase();
    }

    if crate::db::find_user_by_id(&user.id)
        .ok()
        .flatten()
        .is_none()
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": "User could not be found for this token." })),
        ));
    }
    crate::db::update_user(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to update profile." })),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
        })),
    ))
}

async fn get_user_by_id(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let user = crate::db::find_user_by_id(&id)
        .ok()
        .flatten()
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "message": format!("User {} not found.", id) })),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
        })),
    ))
}

async fn delete_user(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let removed = crate::db::delete_user(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to delete user." })),
        )
    })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("User {} not found.", id) })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("User {} deleted.", id) })),
    ))
}
