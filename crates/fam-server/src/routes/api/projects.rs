use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectResponse {
    pub id: i64,
    pub url: String,
    pub name: String,
    pub description: String,
    pub general_area: String,
    pub specific_area: String,
    pub home_url: String,
    pub is_active: bool,
}

pub async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProjectResponse>>, StatusCode> {
    let projects = sqlx::query_as::<_, ProjectResponse>(
        "SELECT id, url, name, description, general_area, specific_area, home_url, is_active \
         FROM projects WHERE is_active = true ORDER BY name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    let project = sqlx::query_as::<_, ProjectResponse>(
        "SELECT id, url, name, description, general_area, specific_area, home_url, is_active \
         FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}
