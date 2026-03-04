use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use fam_core::project_rpc::SetInfoParams;

use crate::routes::api::auth::require_auth;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct UserProjectResponse {
    pub id: i64,
    pub project_id: i64,
    pub project_name: String,
    pub project_url: String,
    pub resource_share: f32,
    pub suspended: bool,
    pub dont_request_more_work: bool,
    pub has_authenticator: bool,
    pub pending_detach: bool,
    pub detach_when_done: bool,
    pub last_error: Option<String>,
    pub consecutive_failures: i32,
    pub no_rsc: Vec<String>,
}

#[derive(Deserialize)]
pub struct EnrollRequest {
    pub project_id: i64,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub resource_share: Option<f32>,
    pub suspended: Option<bool>,
    pub dont_request_more_work: Option<bool>,
    pub no_rsc: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct ProjectPrefsResponse {
    pub project_prefs: Option<String>,
}

#[derive(Deserialize)]
pub struct SetProjectPrefsRequest {
    pub project_prefs: String,
}

pub async fn list_user_projects(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<UserProjectResponse>>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let rows = sqlx::query_as::<_, UserProjectResponse>(
        "SELECT up.id, up.project_id, p.name as project_name, p.url as project_url, \
         up.resource_share, up.suspended, up.dont_request_more_work, \
         (up.project_authenticator != '') as has_authenticator, \
         up.pending_detach, up.detach_when_done, up.last_error, up.consecutive_failures, \
         up.no_rsc \
         FROM user_projects up \
         JOIN projects p ON p.id = up.project_id \
         WHERE up.user_id = $1 \
         ORDER BY p.name",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn enroll(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<EnrollRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let user = require_auth(&state, &headers)
        .await
        .map_err(|s| (s, Json(serde_json::json!({"error": "Unauthorized"}))))?;

    // Check project exists
    let project_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND is_active = true)",
    )
    .bind(body.project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        )
    })?;

    if !project_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        ));
    }

    sqlx::query(
        "INSERT INTO user_projects (user_id, project_id) VALUES ($1, $2) \
         ON CONFLICT (user_id, project_id) DO NOTHING",
    )
    .bind(user.id)
    .bind(body.project_id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"ok": true}))))
}

pub async fn update_user_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    if let Some(rs) = body.resource_share {
        sqlx::query("UPDATE user_projects SET resource_share = $1 WHERE id = $2 AND user_id = $3")
            .bind(rs)
            .bind(id)
            .bind(user.id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(s) = body.suspended {
        sqlx::query("UPDATE user_projects SET suspended = $1 WHERE id = $2 AND user_id = $3")
            .bind(s)
            .bind(id)
            .bind(user.id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(d) = body.dont_request_more_work {
        sqlx::query(
            "UPDATE user_projects SET dont_request_more_work = $1 WHERE id = $2 AND user_id = $3",
        )
        .bind(d)
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ref no_rsc) = body.no_rsc {
        sqlx::query("UPDATE user_projects SET no_rsc = $1 WHERE id = $2 AND user_id = $3")
            .bind(no_rsc)
            .bind(id)
            .bind(user.id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Suspend a user project (set suspended = true).
pub async fn suspend_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    sqlx::query("UPDATE user_projects SET suspended = true WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Resume a user project (set suspended = false).
pub async fn resume_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    sqlx::query("UPDATE user_projects SET suspended = false WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Initiate graceful detach: sets pending_detach flag.
/// The next RPC will tell the BOINC client to detach, then the row is deleted.
pub async fn detach_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    sqlx::query("UPDATE user_projects SET pending_detach = true WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// Immediate leave: deletes the user_project row directly (same as before).
pub async fn leave_project(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    sqlx::query("DELETE FROM user_projects WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /api/user/projects/:id/prefs — fetch project preferences from the project server.
pub async fn get_project_prefs(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ProjectPrefsResponse>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let row = sqlx::query_as::<_, ProjectAuthRow>(
        "SELECT up.project_authenticator, p.url \
         FROM user_projects up \
         JOIN projects p ON p.id = up.project_id \
         WHERE up.id = $1 AND up.user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;

    if row.project_authenticator.is_empty() {
        return Ok(Json(ProjectPrefsResponse { project_prefs: None }));
    }

    let info = state.project_rpc
        .get_info(&row.url, &row.project_authenticator)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to get project info");
            StatusCode::BAD_GATEWAY
        })?;

    Ok(Json(ProjectPrefsResponse {
        project_prefs: info.project_prefs,
    }))
}

/// PUT /api/user/projects/:id/prefs — set project preferences on the project server.
pub async fn set_project_prefs(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SetProjectPrefsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let row = sqlx::query_as::<_, ProjectAuthRow>(
        "SELECT up.project_authenticator, p.url \
         FROM user_projects up \
         JOIN projects p ON p.id = up.project_id \
         WHERE up.id = $1 AND up.user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = row.ok_or(StatusCode::NOT_FOUND)?;

    if row.project_authenticator.is_empty() {
        return Err(StatusCode::PRECONDITION_FAILED);
    }

    state.project_rpc
        .set_info(
            &row.url,
            &row.project_authenticator,
            &SetInfoParams {
                project_prefs: Some(body.project_prefs),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to set project info");
            StatusCode::BAD_GATEWAY
        })?;

    // Set force_update so the next AM reply tells the client to re-fetch
    sqlx::query("UPDATE user_projects SET force_update = true WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /api/projects/:id/apps — get project config (apps, platforms) from the project server.
pub async fn get_project_apps(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<fam_core::project_rpc::ProjectConfig>, StatusCode> {
    let _user = require_auth(&state, &headers).await?;

    let project_url = sqlx::query_scalar::<_, String>(
        "SELECT url FROM projects WHERE id = $1 AND is_active = true",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let config = state.project_rpc
        .get_project_config(&project_url)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to get project config");
            StatusCode::BAD_GATEWAY
        })?;

    Ok(Json(config))
}

#[derive(sqlx::FromRow)]
struct ProjectAuthRow {
    project_authenticator: String,
    url: String,
}
