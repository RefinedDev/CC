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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LectureRecord {
    pub id: i64,
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCourseRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLectureRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
struct ProgressResponse {
    completed_lectures: Vec<i64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_courses))
        .route("/", post(create_course))
        .route("/analytics", get(course_analytics))
        .route("/{id}", get(get_course))
        .route("/{id}", put(update_course))
        .route("/{id}", delete(delete_course))
        .route("/{id}/lectures", get(list_lectures))
        .route("/{id}/lectures", post(create_lecture))
        .route("/{id}/lectures/{lecture_id}", delete(delete_lecture))
        .route("/{id}/progress", get(get_progress))
        .route("/{id}/progress/{lecture_id}", post(complete_lecture))
        .route("/{id}/enroll", post(enroll_course))
        .route("/{id}/unenroll", post(unenroll_course))
}

async fn course_analytics(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    if !matches!(
        claims.role.to_ascii_lowercase().as_str(),
        "trainer" | "admin"
    ) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "message": "Trainer or administrator access is required." })),
        ));
    }
    let courses = crate::db::course_analytics(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load course analytics." })),
        )
    })?;
    let assessments = crate::db::assessment_analytics(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load assessment analytics." })),
        )
    })?;
    let trainees = crate::db::trainee_progress_analytics(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load trainee progress." })),
        )
    })?;
    let attempts = crate::db::assessment_attempt_details(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load assessment attempts." })),
        )
    })?;
    Ok(Json(
        serde_json::json!({ "courses": courses, "assessments": assessments, "trainees": trainees, "attempts": attempts }),
    ))
}

async fn create_lecture(
    headers: HeaderMap,
    Path(course_id): Path<String>,
    Json(payload): Json<CreateLectureRequest>,
) -> Result<(StatusCode, Json<LectureRecord>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    let course = crate::db::find_course(&course_id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "message": "Course not found."
                })),
            )
        })?;
    if course.created_by != claims.sub {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "message": "Only the course creator can manage its lectures."
            })),
        ));
    }
    let title = payload.title.trim();
    let description = payload.description.trim();
    if title.is_empty() || description.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "message": "Lecture title and description are required."
            })),
        ));
    }
    let lecture = crate::db::insert_lecture(&course_id, title, description).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to create lecture." })),
        )
    })?;
    Ok((StatusCode::CREATED, Json(lecture)))
}

async fn delete_lecture(
    headers: HeaderMap,
    Path((course_id, lecture_id)): Path<(String, i64)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    let course = crate::db::find_course(&course_id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "message": "Course not found."
                })),
            )
        })?;
    if course.created_by != claims.sub {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "message": "Only the course creator can manage its lectures."
            })),
        ));
    }
    if !crate::db::delete_lecture(&course_id, lecture_id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to delete lecture." })),
        )
    })? {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "message": "Lecture not found."
            })),
        ));
    }
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Lecture deleted." })),
    ))
}

async fn get_progress(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ProgressResponse>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    let completed_lectures = crate::db::completed_lectures(&claims.sub, &id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load progress." })),
        )
    })?;
    Ok((
        StatusCode::OK,
        Json(ProgressResponse { completed_lectures }),
    ))
}

async fn complete_lecture(
    headers: HeaderMap,
    Path((id, lecture_id)): Path<(String, i64)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;
    let completed =
        crate::db::mark_lecture_complete(&claims.sub, &id, lecture_id).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to save progress." })),
            )
        })?;
    if !completed {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "message": "Lecture not found for this course."
            })),
        ));
    }
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Lecture completed.",
            "lecture_id": lecture_id
        })),
    ))
}

async fn list_lectures(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Vec<LectureRecord>>), (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Invalid or missing authentication token." })),
        )
    })?;

    if crate::db::find_course(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .is_none()
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "message": format!("Course {} not found.", id)
            })),
        ));
    }

    let lectures = crate::db::list_lectures(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load lectures." })),
        )
    })?;
    Ok((StatusCode::OK, Json(lectures)))
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

    let courses = crate::db::list_courses().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load courses." })),
        )
    })?;
    let mut payload = Vec::with_capacity(courses.len());
    for course in courses {
        let enrolled_users = crate::db::enrolled_users(&course.id).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load enrollments." })),
            )
        })?;
        payload.push(serde_json::json!({
            "id": course.id,
            "title": course.title,
            "description": course.description,
            "created_by": course.created_by,
            "enrolled_users": enrolled_users
        }));
    }

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
    if !matches!(
        claims.role.to_ascii_lowercase().as_str(),
        "trainer" | "admin"
    ) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "message": "Trainer or administrator access is required."
            })),
        ));
    }

    let title = payload.title.trim();
    let description = payload.description.trim();
    if title.is_empty() || description.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "message": "Course title and description are required." })),
        ));
    }

    let id = crate::db::next_course_id().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to create course." })),
        )
    })?;
    let course = CourseRecord {
        id: id.clone(),
        title: title.to_string(),
        description: description.to_string(),
        created_by: claims.sub.clone(),
    };
    crate::db::insert_course(&course).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to create course." })),
        )
    })?;

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

    let course = crate::db::find_course(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
            )
        })?;

    let enrolled_users = crate::db::enrolled_users(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to load enrollments." })),
        )
    })?;

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

    let mut course = crate::db::find_course(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
            )
        })?;

    course.title = title.to_string();
    course.description = description.to_string();
    crate::db::update_course(&course).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to update course." })),
        )
    })?;

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

    let removed = crate::db::delete_course(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to delete course." })),
        )
    })?;
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

    if !crate::db::find_course(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "message": "Failed to load course." })),
            )
        })?
        .is_some()
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": format!("Course {} not found.", id) })),
        ));
    }

    crate::db::enroll(&id, &claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to enroll in course." })),
        )
    })?;

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

    crate::db::unenroll(&id, &claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Failed to unenroll from course." })),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": format!("User {} unenrolled from course {}.", claims.sub, id),
            "course_id": id,
            "user_id": claims.sub,
        })),
    ))
}
