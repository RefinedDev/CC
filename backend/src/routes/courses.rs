use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};

use crate::routes::auth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCourseRequest {
    pub title: String,
    pub description: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_courses))
        .route("/", post(create_course))
        .route("/{id}", get(get_course))
        .route("/{id}", put(update_course))
        .route("/{id}", delete(delete_course))
        .route("/{id}/enroll", post(enroll_course))
        .route("/{id}/unenroll", post(unenroll_course))
}

async fn list_courses(
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let payload = crate::db::list_courses().map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load courses." })),
    ))?;

    Ok((StatusCode::OK, Json(serde_json::json!(payload))))
}

async fn create_course(
    headers: HeaderMap,
    Json(payload): Json<CreateCourseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let title = payload.title.trim();
    let description = payload.description.trim();
    if title.is_empty() || description.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Course title and description are required." })),
        ));
    }

    let id = crate::db::next_course_id().map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to create course." })),
    ))?;
    let course = CourseRecord {
        id: id.clone(),
        title: title.to_string(),
        description: description.to_string(),
        created_by: claims.sub.clone(),
    };
    crate::db::insert_course(&course).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to create course." })),
    ))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": course.id,
            "title": course.title,
            "description": course.description,
            "created_by": course.created_by,
        })),
    ))
}

async fn get_course(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let course = crate::db::find_course(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load course." })),
    ))?.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        )
    })?;

    let enrolled_users = crate::db::enrolled_users(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load enrollments." })),
    ))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": course.id,
            "title": course.title,
            "description": course.description,
            "created_by": course.created_by,
            "enrolled_users": enrolled_users,
        })),
    ))
}

async fn update_course(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<CreateCourseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let title = payload.title.trim();
    let description = payload.description.trim();
    if title.is_empty() || description.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Course title and description are required." })),
        ));
    }

    let mut course = crate::db::find_course(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load course." })),
    ))?.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        )
    })?;

    course.title = title.to_string();
    course.description = description.to_string();
    crate::db::update_course(&course).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to update course." })),
    ))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": course.id,
            "title": course.title,
            "description": course.description,
            "created_by": course.created_by,
        })),
    ))
}

async fn delete_course(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    let removed = crate::db::delete_course(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to delete course." })),
    ))?;
    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": format!("Course {} deleted.", id) })),
    ))
}

async fn enroll_course(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    if !crate::db::find_course(&id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to load course." })),
    ))?.is_some() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        ));
    }

    crate::db::enroll(&id, &claims.sub).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to enroll in course." })),
    ))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("User {} enrolled in course {}.", claims.sub, id),
            "course_id": id,
            "user_id": claims.sub,
        })),
    ))
}

async fn unenroll_course(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    crate::db::unenroll(&id, &claims.sub).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "message": "Failed to unenroll from course." })),
    ))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("User {} unenrolled from course {}.", claims.sub, id),
            "course_id": id,
            "user_id": claims.sub,
        })),
    ))
}
