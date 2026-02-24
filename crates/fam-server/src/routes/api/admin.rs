use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::routes::api::auth::require_admin;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminUserResponse {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub country: String,
    pub is_admin: bool,
    pub created_at: time::OffsetDateTime,
}

#[derive(Serialize)]
pub struct AdminStatsResponse {
    pub total_users: i64,
    pub total_hosts: i64,
    pub total_projects: i64,
    pub total_enrollments: i64,
    pub active_sessions: i64,
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub general_area: Option<String>,
    pub specific_area: Option<String>,
    pub home_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub general_area: Option<String>,
    pub specific_area: Option<String>,
    pub home_url: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<AdminUserResponse>>, StatusCode> {
    require_admin(&state, &headers).await?;

    let users = sqlx::query_as::<_, AdminUserResponse>(
        "SELECT id, email, name, country, is_admin, created_at FROM users ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<AdminStatsResponse>, StatusCode> {
    require_admin(&state, &headers).await?;

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_hosts = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM hosts")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_projects =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE is_active = true")
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_enrollments = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_projects")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_sessions =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE expires_at > NOW()")
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStatsResponse {
        total_users,
        total_hosts,
        total_projects,
        total_enrollments,
        active_sessions,
    }))
}

pub async fn create_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    require_admin(&state, &headers)
        .await
        .map_err(|s| (s, Json(serde_json::json!({"error": "Forbidden"}))))?;

    // Compute URL signature
    let url_signature = fam_core::crypto::sign_url(&body.url, &state.rsa_private_key)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to sign URL: {e}")})),
            )
        })?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO projects (url, name, description, general_area, specific_area, home_url, url_signature) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(&body.url)
    .bind(&body.name)
    .bind(body.description.as_deref().unwrap_or(""))
    .bind(body.general_area.as_deref().unwrap_or(""))
    .bind(body.specific_area.as_deref().unwrap_or(""))
    .bind(body.home_url.as_deref().unwrap_or(""))
    .bind(&url_signature)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        let msg = if e.to_string().contains("duplicate key") {
            "Project URL already exists"
        } else {
            "Failed to create project"
        };
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"id": id, "ok": true})),
    ))
}

pub async fn update_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    require_admin(&state, &headers).await?;

    if let Some(ref name) = body.name {
        sqlx::query("UPDATE projects SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ref desc) = body.description {
        sqlx::query("UPDATE projects SET description = $1 WHERE id = $2")
            .bind(desc)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ref area) = body.general_area {
        sqlx::query("UPDATE projects SET general_area = $1 WHERE id = $2")
            .bind(area)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ref area) = body.specific_area {
        sqlx::query("UPDATE projects SET specific_area = $1 WHERE id = $2")
            .bind(area)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ref url) = body.home_url {
        sqlx::query("UPDATE projects SET home_url = $1 WHERE id = $2")
            .bind(url)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(active) = body.is_active {
        sqlx::query("UPDATE projects SET is_active = $1 WHERE id = $2")
            .bind(active)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}
