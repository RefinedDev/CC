use crate::{db, routes::auth};
use axum::{
    Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreateAssessmentRequest {
    title: String,
    subject: String,
    deadline: String,
    duration_minutes: i64,
    questions: Vec<QuestionRequest>,
}
#[derive(Debug, Deserialize)]
struct QuestionRequest {
    prompt: String,
    options: Vec<String>,
    correct_option: String,
}
#[derive(Debug, Deserialize)]
struct AttemptRequest {
    answers: std::collections::HashMap<String, String>,
}
#[derive(Debug, Deserialize)]
struct ResultQuery {
    user_id: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(detail))
        .route("/{id}", delete(remove))
        .route("/{id}/attempt", post(attempt))
        .route("/{id}/result", get(result))
}

fn auth_user(headers: &HeaderMap) -> Result<auth::Claims, (StatusCode, Json<serde_json::Value>)> {
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
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let items = db::list_assessments().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load assessments.",
        )
    })?;
    let payload = items.into_iter().map(|item| {
        let result = db::assessment_result(item.id, &claims.sub).ok().flatten();
        serde_json::json!({"id":item.id,"title":item.title,"subject":item.subject,"deadline":item.deadline,"duration_minutes":item.duration_minutes,"created_by":item.created_by,"result":result.map(|(score,total,submitted_at)| serde_json::json!({"score":score,"total":total,"submitted_at":submitted_at}))})
    }).collect::<Vec<_>>();
    Ok(Json(serde_json::json!(payload)))
}

async fn detail(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    auth_user(&headers)?;
    let item = db::find_assessment(id)
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load assessment.",
            )
        })?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Assessment not found."))?;
    let questions = db::assessment_questions(id, false).map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load questions.",
        )
    })?;
    Ok(Json(
        serde_json::json!({"id":item.id,"title":item.title,"subject":item.subject,"deadline":item.deadline,"duration_minutes":item.duration_minutes,"created_by":item.created_by,"questions":questions.into_iter().map(|(question,_)| question).collect::<Vec<_>>() }),
    ))
}

async fn create(
    headers: HeaderMap,
    Json(payload): Json<CreateAssessmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    if claims.role.to_ascii_lowercase() != "trainer" && claims.role.to_ascii_lowercase() != "admin"
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Trainer or administrator access is required.",
        ));
    }
    if payload.title.trim().is_empty()
        || payload.subject.trim().is_empty()
        || payload.deadline.trim().is_empty()
        || payload.questions.is_empty()
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Title, subject, deadline, and at least one question are required.",
        ));
    }
    for question in &payload.questions {
        if question.prompt.trim().is_empty()
            || question.options.len() != 4
            || !["A", "B", "C", "D"].contains(&question.correct_option.as_str())
            || question
                .options
                .iter()
                .any(|option| option.trim().is_empty())
        {
            return Err(error(
                StatusCode::BAD_REQUEST,
                "Each question needs four options and a valid correct answer.",
            ));
        }
    }

    let assessment = db::AssessmentRecord {
        id: 0,
        title: payload.title.trim().to_string(),
        subject: payload.subject.trim().to_string(),
        deadline: payload.deadline.trim().to_string(),
        duration_minutes: payload.duration_minutes.clamp(1, 240),
        created_by: claims.sub,
    };
    let questions = payload
        .questions
        .into_iter()
        .map(|q| (q.prompt.trim().to_string(), q.options, q.correct_option))
        .collect::<Vec<_>>();
    db::insert_assessment(&assessment, &questions).map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save assessment.",
        )
    })?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"message":"Assessment saved."})),
    ))
}

async fn remove(
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    if claims.role.to_ascii_lowercase() != "trainer" && claims.role.to_ascii_lowercase() != "admin"
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Trainer or administrator access is required.",
        ));
    }
    if !db::delete_assessment(id, &claims.sub).map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete assessment.",
        )
    })? {
        return Err(error(
            StatusCode::NOT_FOUND,
            "Assessment not found or you do not own it.",
        ));
    }
    Ok(Json(serde_json::json!({"message":"Assessment deleted."})))
}

async fn attempt(
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<AttemptRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let result = db::submit_assessment(id, &claims.sub, &payload.answers)
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to submit assessment.",
            )
        })?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Assessment not found."))?;
    Ok(Json(
        serde_json::json!({"score":result.0,"total":result.1,"submitted_at":result.2}),
    ))
}

async fn result(
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(query): Query<ResultQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = auth_user(&headers)?;
    let user_id = query.user_id.as_deref().unwrap_or(&claims.sub);
    if user_id != claims.sub
        && claims.role.to_ascii_lowercase() != "trainer"
        && claims.role.to_ascii_lowercase() != "admin"
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "You can only view your own result.",
        ));
    }
    let result = db::assessment_result(id, user_id)
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load result."))?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "No submitted result found."))?;
    Ok(Json(
        serde_json::json!({"score":result.0,"total":result.1,"submitted_at":result.2}),
    ))
}
