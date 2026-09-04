use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::routes::auth;

pub static COURSES: LazyLock<Mutex<HashMap<String, CourseRecord>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static ENROLLMENTS: LazyLock<Mutex<HashMap<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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

    let mut courses = COURSES.lock().unwrap();
    if courses.len() == 0 {
        courses.insert(
            "1".to_string(),
            CourseRecord {
                id: "course_1".to_string(),
                title: "Introduction to Rust".to_string(),
                description: "Learn the basics of Rust programming language.".to_string(),
                created_by: "admin".to_string(),
            },
        );
    }
    let payload = courses.values().cloned().collect::<Vec<CourseRecord>>();

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

    let mut courses = COURSES.lock().unwrap();
    let id = format!("course_{}", courses.len() + 1);
    let course = CourseRecord {
        id: id.clone(),
        title: title.to_string(),
        description: description.to_string(),
        created_by: claims.sub.clone(),
    };
    courses.insert(id.clone(), course.clone());

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

    let courses = COURSES.lock().unwrap();
    let course = courses.get(&id).cloned().ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        )
    })?;

    let enrollments = ENROLLMENTS.lock().unwrap();
    let enrolled_users = enrollments.get(&id).cloned().unwrap_or_default();

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

    let mut courses = COURSES.lock().unwrap();
    let mut course = courses.get(&id).cloned().ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        )
    })?;

    course.title = title.to_string();
    course.description = description.to_string();
    courses.insert(id.clone(), course.clone());

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

    let mut courses = COURSES.lock().unwrap();
    let removed = courses.remove(&id).is_some();
    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        ));
    }

    let mut enrollments = ENROLLMENTS.lock().unwrap();
    enrollments.remove(&id);

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

    let courses = COURSES.lock().unwrap();
    if !courses.contains_key(&id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        ));
    }

    let mut enrollments = ENROLLMENTS.lock().unwrap();
    let list = enrollments.entry(id.clone()).or_default();
    if !list.contains(&claims.sub) {
        list.push(claims.sub.clone());
    }

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

    let mut enrollments = ENROLLMENTS.lock().unwrap();
    let list = enrollments.get_mut(&id);
    if let Some(list) = list {
        list.retain(|user_id| user_id != &claims.sub);
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("User {} unenrolled from course {}.", claims.sub, id),
            "course_id": id,
            "user_id": claims.sub,
        })),
    ))
}
