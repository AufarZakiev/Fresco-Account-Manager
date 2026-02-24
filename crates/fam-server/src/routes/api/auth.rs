use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::api::session_from_cookie;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub country: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = fam_core::auth::register_user(&state.db, &body.email, &body.name, &body.password)
        .await
        .map_err(|e| {
            let msg = match e {
                fam_core::auth::AuthError::Database(ref db_err) => {
                    if db_err.to_string().contains("duplicate key") {
                        "Email already registered"
                    } else {
                        "Registration failed"
                    }
                }
                _ => "Registration failed",
            };
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": msg})),
            )
        })?;

    // Create session
    let session_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '30 days')",
    )
    .bind(&session_id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        )
    })?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        format!("session={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000")
            .parse()
            .unwrap(),
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            country: user.country,
        }),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let password_hash = fam_core::auth::compute_password_hash(&body.password, &body.email);

    let user = fam_core::auth::authenticate_by_password(&state.db, &body.email, &password_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid credentials"})),
            )
        })?;

    let session_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '30 days')",
    )
    .bind(&session_id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        )
    })?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        format!("session={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000")
            .parse()
            .unwrap(),
    );

    Ok((
        headers,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            country: user.country,
        }),
    ))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    if let Some(session_id) = session_from_cookie(&headers) {
        let _ = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(&session_id)
            .execute(&state.db)
            .await;
    }

    let mut resp_headers = axum::http::HeaderMap::new();
    resp_headers.insert(
        "Set-Cookie",
        "session=; Path=/; HttpOnly; Max-Age=0".parse().unwrap(),
    );

    (resp_headers, Json(serde_json::json!({"ok": true})))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = require_auth(&state, &headers).await?;
    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        country: user.country,
    }))
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user = require_auth(&state, &headers)
        .await
        .map_err(|s| (s, Json(serde_json::json!({"error": "Unauthorized"}))))?;

    if body.new_password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Password must be at least 6 characters"})),
        ));
    }

    fam_core::auth::change_password(
        &state.db,
        user.id,
        &user.email,
        &body.old_password,
        &body.new_password,
    )
    .await
    .map_err(|e| match e {
        fam_core::auth::AuthError::InvalidCredentials => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Current password is incorrect"})),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to change password"})),
        ),
    })?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Helper to require authenticated user from session cookie.
pub async fn require_auth(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<fam_core::models::User, StatusCode> {
    let session_id = session_from_cookie(headers).ok_or(StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query_as::<_, fam_core::models::User>(
        "SELECT u.id, u.email, u.name, u.password_hash, u.authenticator, u.country, u.is_admin \
         FROM users u \
         JOIN sessions s ON s.user_id = u.id \
         WHERE s.id = $1 AND s.expires_at > NOW()",
    )
    .bind(&session_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(user)
}

/// Helper to require an admin user from session cookie.
pub async fn require_admin(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<fam_core::models::User, StatusCode> {
    let user = require_auth(state, headers).await?;
    if !user.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(user)
}
