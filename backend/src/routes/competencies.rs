use crate::{db, routes::auth};
use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CompetencyPayload {
    skills: String,
    interests: String,
}
#[derive(Deserialize)]
struct RequirementPayload {
    skills: String,
}
fn auth_user(headers: &HeaderMap) -> Result<auth::Claims, (StatusCode, Json<serde_json::Value>)> {
    auth::auth_from_headers(headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"message":"Invalid or missing authentication token."})),
        )
    })
}
pub fn router() -> Router {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/recommendations", get(recommendations))
        .route(
            "/courses/{id}/requirements",
            get(get_requirements).put(update_requirements),
        )
}
async fn get_me(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let (skills, interests) = db::get_competencies(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to load competencies."})),
        )
    })?;
    Ok(Json(
        serde_json::json!({"skills":skills,"interests":interests}),
    ))
}
async fn update_me(
    headers: HeaderMap,
    Json(payload): Json<CompetencyPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    db::update_competencies(&claims.sub, payload.skills.trim(), payload.interests.trim()).map_err(
        |_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message":"Failed to save competencies."})),
            )
        },
    )?;
    Ok(Json(serde_json::json!({"message":"Competencies saved."})))
}
async fn recommendations(
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let (skills, _) = db::get_competencies(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to load competencies."})),
        )
    })?;
    Ok(Json(db::course_recommendations(&skills).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to load recommendations."})),
        )
    })?))
}

async fn get_requirements(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _claims = auth_user(&headers)?;
    let skills = db::get_course_requirements(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to load course requirements."})),
        )
    })?;
    Ok(Json(serde_json::json!({"skills": skills})))
}
async fn update_requirements(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RequirementPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let course = db::find_course(&id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"message":"Failed to load course."})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"message":"Course not found."})),
            )
        })?;
    if course.created_by != claims.sub && claims.role.to_ascii_lowercase() != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"message":"Course ownership is required."})),
        ));
    }
    db::update_course_requirements(&id, payload.skills.trim()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"message":"Failed to save requirements."})),
        )
    })?;
    Ok(Json(
        serde_json::json!({"message":"Course requirements saved."}),
    ))
}
