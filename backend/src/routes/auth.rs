use axum::{
    Json,
    Router,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const JWT_SECRET: &[u8] = b"capacity-connect-dev-secret-key";

pub static USERS: LazyLock<Mutex<HashMap<String, UserRecord>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: Option<String>,
    pub user: Option<UserSummary>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
}

async fn signup(Json(payload): Json<SignupRequest>) -> (StatusCode, Json<AuthResponse>) {
    let email = payload.email.trim().to_ascii_lowercase();
    let name = payload.name.trim().to_string();

    if email.is_empty() || name.is_empty() || payload.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                token: None,
                user: None,
                message: "Email, name, and password are required. Password must be at least 8 characters.".to_string(),
            }),
        );
    }

    let mut users = USERS.lock().unwrap();
    if users.values().any(|user| user.email == email) {
        return (
            StatusCode::CONFLICT,
            Json(AuthResponse {
                token: None,
                user: None,
                message: "An account with this email already exists.".to_string(),
            }),
        );
    }

    let id = format!("user_{}", users.len() + 1);
    let password_hash = hash_password(&payload.password);
    let user = UserRecord {
        id: id.clone(),
        name: name.clone(),
        email: email.clone(),
        password_hash,
        role: "trainee".to_string(),
    };

    let token = match create_token(&user.id, &user.role) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "Failed to create authentication token.".to_string(),
                }),
            );
        }
    };

    users.insert(id.clone(), user.clone());

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            token: Some(token),
            user: Some(UserSummary {
                id: user.id,
                name: user.name,
                email: user.email,
                role: user.role,
            }),
            message: "Signup successful.".to_string(),
        }),
    )
}

async fn login(Json(payload): Json<LoginRequest>) -> (StatusCode, Json<AuthResponse>) {
    let email = payload.email.trim().to_ascii_lowercase();
    let password = payload.password.trim();

    if email.is_empty() || password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                token: None,
                user: None,
                message: "Email and password are required.".to_string(),
            }),
        );
    }

    let users = USERS.lock().unwrap();
    let user = match users.values().find(|user| user.email == email) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "Invalid email or password.".to_string(),
                }),
            );
        }
    };

    if !verify_password(password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                token: None,
                user: None,
                message: "Invalid email or password.".to_string(),
            }),
        );
    }

    let token = match create_token(&user.id, &user.role) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "Failed to create token.".to_string(),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            token: Some(token),
            user: Some(UserSummary {
                id: user.id.clone(),
                name: user.name.clone(),
                email: user.email.clone(),
                role: user.role.clone(),
            }),
            message: "Login successful.".to_string(),
        }),
    )
}

async fn logout() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Logout successful. Client should discard the token."
        })),
    )
}

async fn refresh_token(
    Json(payload): Json<RefreshTokenRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let token = payload.token.trim();
    if token.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                token: None,
                user: None,
                message: "Refresh token is required.".to_string(),
            }),
        );
    }

    let claims = match decode_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "Refresh token is invalid or expired.".to_string(),
                }),
            );
        }
    };

    let users = USERS.lock().unwrap();
    let user = match users.get(&claims.sub) {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "User no longer exists.".to_string(),
                }),
            );
        }
    };

    let new_token = match create_token(&user.id, &user.role) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    token: None,
                    user: None,
                    message: "Failed to refresh token.".to_string(),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            token: Some(new_token),
            user: Some(UserSummary {
                id: user.id.clone(),
                name: user.name.clone(),
                email: user.email.clone(),
                role: user.role.clone(),
            }),
            message: "Token refreshed successfully.".to_string(),
        }),
    )
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    hash_password(password) == password_hash
}

fn create_token(user_id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(24)).timestamp() as usize,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET), &validation)?;
    Ok(token_data.claims)
}

pub fn get_user_by_id(user_id: &str) -> Option<UserRecord> {
    let users = USERS.lock().unwrap();
    users.get(user_id).cloned()
}

pub fn get_user_by_email(email: &str) -> Option<UserRecord> {
    let users = USERS.lock().unwrap();
    users.values().find(|user| user.email == email).cloned()
}

pub fn auth_from_headers(headers: &HeaderMap) -> Result<Claims, &'static str> {
    let auth_header = headers
        .get("authorization")
        .ok_or("Missing authorization header")?
        .to_str()
        .map_err(|_| "Invalid authorization header")?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or("Authorization header must use Bearer scheme")?;

    decode_token(token).map_err(|_| "Invalid or expired JWT")
}
