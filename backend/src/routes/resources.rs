use crate::{db, routes::auth};
use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{delete, get},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreateResourceRequest {
    name: String,
    kind: String,
    size_bytes: i64,
    course_id: Option<String>,
    content_base64: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}/download", get(download))
        .route("/{id}", delete(remove))
}

fn user(headers: &HeaderMap) -> Result<auth::Claims, (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })
}
fn error(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"message":message})))
}

async fn list(
    headers: HeaderMap,
) -> Result<Json<Vec<db::ResourceRecord>>, (StatusCode, Json<serde_json::Value>)> {
    user(&headers)?;
    Ok(Json(db::list_resources().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load resources.",
        )
    })?))
}

async fn create(
    headers: HeaderMap,
    Json(payload): Json<CreateResourceRequest>,
) -> Result<(StatusCode, Json<db::ResourceRecord>), (StatusCode, Json<serde_json::Value>)> {
    let claims = user(&headers)?;
    if !matches!(
        claims.role.to_ascii_lowercase().as_str(),
        "trainer" | "admin"
    ) {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Trainer or administrator access is required.",
        ));
    }
    if payload.name.trim().is_empty()
        || payload.content_base64.trim().is_empty()
        || payload.size_bytes < 0
        || payload.size_bytes > 25_000_000
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "A resource file up to 25 MB is required.",
        ));
    }
    let id = db::insert_resource(
        &payload.name.trim(),
        &payload.kind,
        payload.size_bytes,
        payload.course_id.as_deref(),
        &claims.sub,
        &payload.content_base64,
    )
    .map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save resource.",
        )
    })?;
    let resource = db::list_resources()
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load resource.",
            )
        })?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Saved resource could not be loaded.",
            )
        })?;
    Ok((StatusCode::CREATED, Json(resource)))
}

async fn download(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    user(&headers)?;
    let (resource, content) = db::resource_content(id)
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load resource.",
            )
        })?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Resource not found."))?;
    Ok(Json(
        serde_json::json!({"name":resource.name,"kind":resource.kind,"content_base64":content}),
    ))
}

async fn remove(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = user(&headers)?;
    if !matches!(
        claims.role.to_ascii_lowercase().as_str(),
        "trainer" | "admin"
    ) {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Trainer or administrator access is required.",
        ));
    }
    if !db::delete_resource(id, &claims.sub).map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete resource.",
        )
    })? {
        return Err(error(
            StatusCode::NOT_FOUND,
            "Resource not found or you do not own it.",
        ));
    }
    Ok(Json(serde_json::json!({"message":"Resource deleted."})))
}
