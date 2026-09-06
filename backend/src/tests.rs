use super::*;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::sync::{LazyLock, Mutex};
use tower::ServiceExt;

static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn call(method: &str, uri: &str, token: Option<&str>, value: Value) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app()
        .oneshot(request.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2_000_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn signup(email: &str) -> (String, String) {
    let (status, body) = call(
        "POST",
        "/api/auth/signup",
        None,
        json!({"email": email, "name": "Test User", "password": "password123"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (
        body["token"].as_str().unwrap().into(),
        body["user"]["id"].as_str().unwrap().into(),
    )
}

async fn trainer(email: &str) -> String {
    let (_, id) = signup(email).await;
    let mut user = db::find_user_by_id(&id).unwrap().unwrap();
    user.role = "trainer".into();
    db::update_user(&user).unwrap();
    let (status, body) = call(
        "POST",
        "/api/auth/login",
        None,
        json!({"email": email, "password": "password123"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["token"].as_str().unwrap().into()
}

#[tokio::test]
async fn auth_and_role_access() {
    let _lock = TEST_LOCK.lock().unwrap();
    db::reset_for_tests().unwrap();
    let (trainee, _) = signup("trainee-auth@test.invalid").await;
    assert_eq!(
        call(
            "POST",
            "/api/auth/login",
            None,
            json!({"email":"trainee-auth@test.invalid","password":"bad"})
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call("GET", "/api/users/me", Some(&trainee), json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            "POST",
            "/api/courses",
            Some(&trainee),
            json!({"title":"No","description":"No"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let staff = trainer("trainer-auth@test.invalid").await;
    assert_eq!(
        call(
            "POST",
            "/api/courses",
            Some(&staff),
            json!({"title":"Allowed","description":"Course"})
        )
        .await
        .0,
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn assessments_support_creation_attempts_and_results() {
    let _lock = TEST_LOCK.lock().unwrap();
    db::reset_for_tests().unwrap();
    let staff = trainer("trainer-assessment@test.invalid").await;
    let (trainee, _) = signup("trainee-assessment@test.invalid").await;
    let assessment = json!({"title":"Rust","subject":"Backend","deadline":"2099-01-01",
        "duration_minutes":10,"questions":[{"prompt":"q","options":["a","b","c","d"],"correct_option":"A"}]});
    assert_eq!(
        call(
            "POST",
            "/api/assessments",
            Some(&trainee),
            assessment.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call("POST", "/api/assessments", Some(&staff), assessment)
            .await
            .0,
        StatusCode::CREATED
    );
    let list = call("GET", "/api/assessments", Some(&trainee), json!({}))
        .await
        .1;
    let id = list[0]["id"].as_i64().unwrap();
    assert_eq!(
        call(
            "POST",
            &format!("/api/assessments/{id}/attempt"),
            Some(&trainee),
            json!({"answers":{"1":"A"}})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            "GET",
            &format!("/api/assessments/{id}/result"),
            Some(&trainee),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn enrollment_and_progress_are_persisted() {
    let _lock = TEST_LOCK.lock().unwrap();
    db::reset_for_tests().unwrap();
    let staff = trainer("trainer-progress@test.invalid").await;
    let (trainee, _) = signup("trainee-progress@test.invalid").await;
    let course = call(
        "POST",
        "/api/courses",
        Some(&staff),
        json!({"title":"Course","description":"Description"}),
    )
    .await
    .1;
    let course_id = course["id"].as_str().unwrap();
    let lecture = call(
        "POST",
        &format!("/api/courses/{course_id}/lectures"),
        Some(&staff),
        json!({"title":"Lesson","description":"Read"}),
    )
    .await
    .1;
    let lecture_id = lecture["id"].as_i64().unwrap();
    assert_eq!(
        call(
            "POST",
            &format!("/api/courses/{course_id}/enroll"),
            Some(&trainee),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            "POST",
            &format!("/api/courses/{course_id}/progress/{lecture_id}"),
            Some(&trainee),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
    let progress = call(
        "GET",
        &format!("/api/courses/{course_id}/progress"),
        Some(&trainee),
        json!({}),
    )
    .await
    .1;
    assert_eq!(progress["completed_lectures"], json!([lecture_id]));
}

#[tokio::test]
async fn resources_enforce_roles_and_round_trip_content() {
    let _lock = TEST_LOCK.lock().unwrap();
    db::reset_for_tests().unwrap();
    let staff = trainer("trainer-resource@test.invalid").await;
    let (trainee, _) = signup("trainee-resource@test.invalid").await;
    let payload =
        json!({"name":"notes.txt","kind":"text/plain","size_bytes":3,"content_base64":"YWJj"});
    assert_eq!(
        call("POST", "/api/resources", Some(&trainee), payload.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let resource = call("POST", "/api/resources", Some(&staff), payload)
        .await
        .1;
    let id = resource["id"].as_i64().unwrap();
    let downloaded = call(
        "GET",
        &format!("/api/resources/{id}/download"),
        Some(&trainee),
        json!({}),
    )
    .await
    .1;
    assert_eq!(downloaded["content_base64"], "YWJj");
    assert_eq!(
        call(
            "DELETE",
            &format!("/api/resources/{id}"),
            Some(&staff),
            json!({})
        )
        .await
        .0,
        StatusCode::OK
    );
}
