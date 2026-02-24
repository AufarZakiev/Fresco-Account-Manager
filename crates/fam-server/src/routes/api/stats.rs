use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::routes::api::auth::require_auth;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectCredit {
    pub project_name: String,
    pub total_credit: f32,
    pub recent_credit: f32,
}

#[derive(Serialize)]
pub struct UserStatsResponse {
    pub total_credit: f64,
    pub recent_credit: f64,
    pub project_count: i64,
    pub host_count: i64,
    pub projects: Vec<ProjectCredit>,
}

pub async fn get_user_stats(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserStatsResponse>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let projects = sqlx::query_as::<_, ProjectCredit>(
        "SELECT p.name as project_name, up.total_credit, up.recent_credit \
         FROM user_projects up \
         JOIN projects p ON p.id = up.project_id \
         WHERE up.user_id = $1 \
         ORDER BY up.total_credit DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_credit: f64 = projects.iter().map(|p| p.total_credit as f64).sum();
    let recent_credit: f64 = projects.iter().map(|p| p.recent_credit as f64).sum();
    let project_count = projects.len() as i64;

    let host_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM hosts WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UserStatsResponse {
        total_credit,
        recent_credit,
        project_count,
        host_count,
        projects,
    }))
}
