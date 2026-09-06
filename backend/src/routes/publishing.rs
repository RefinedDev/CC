use crate::{db, routes::auth};
use axum::{
    Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct KindQuery {
    kind: Option<String>,
}
pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/manage", get(manage))
        .route("/{id}/read", post(mark_read))
        .route("/{id}", put(edit).delete(remove))
        .route("/read-all", post(mark_all_read))
}

async fn manage(
    headers: HeaderMap,
) -> Result<Json<Vec<db::PublicationRecord>>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"message":"Administrator access is required."})),
        ));
    }
    Ok(Json(db::list_all_publications().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to load publications."})),
        )
    })?))
}

async fn edit(
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<db::PublicationRecord>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"message":"Administrator access is required."})),
        ));
    }
    let kind = payload
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("announcement");
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let target = payload
        .get("target_user_id")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty());
    if title.is_empty() || body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message":"Title and body are required."})),
        ));
    }
    if !db::update_publication(id, kind, title, body, target).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to update publication."})),
        )
    })? {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message":"Publication not found."})),
        ));
    }
    let item = db::list_all_publications()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message":"Failed to load publication."})),
            )
        })?
        .into_iter()
        .find(|item| item.id == id)
        .unwrap();
    Ok(Json(item))
}

async fn remove(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"message":"Administrator access is required."})),
        ));
    }
    if !db::delete_publication(id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to delete publication."})),
        )
    })? {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message":"Publication not found."})),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn list(
    headers: HeaderMap,
    Query(query): Query<KindQuery>,
) -> Result<Json<Vec<db::PublicationRecord>>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    Ok(Json(
        db::list_publications(query.kind.as_deref(), &claims.sub).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message":"Failed to load published content."})),
            )
        })?,
    ))
}
async fn create(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<db::PublicationRecord>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    if claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"message":"Administrator access is required."})),
        ));
    }
    let kind = payload
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("announcement");
    if !matches!(
        kind,
        "notification" | "announcement" | "achievement" | "content"
    ) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message":"Invalid publication type."})),
        ));
    }
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if title.is_empty() || body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message":"Title and body are required."})),
        ));
    }
    let target_user_id = payload
        .get("target_user_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(user_id) = target_user_id {
        if db::find_user_by_id(user_id)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"message":"Failed to validate target user."})),
                )
            })?
            .is_none()
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"message":"Target user was not found."})),
            ));
        }
    }
    let item =
        db::insert_publication(kind, title, body, &claims.sub, target_user_id).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message":"Failed to publish content."})),
            )
        })?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn mark_read(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    db::mark_publication_read(id, &claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to mark notification as read."})),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mark_all_read(
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })?;
    db::mark_all_publications_read(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to mark notifications as read."})),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}
