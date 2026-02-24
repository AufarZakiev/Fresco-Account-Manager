use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::routes::api::auth::require_auth;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct HostResponse {
    pub id: i64,
    pub host_cpid: String,
    pub domain_name: String,
    pub client_version: String,
    pub platform_name: String,
    pub venue: String,
    pub run_mode: String,
    pub last_rpc_at: time::OffsetDateTime,
}

#[derive(Serialize)]
pub struct HostDetailResponse {
    pub id: i64,
    pub host_cpid: String,
    pub domain_name: String,
    pub client_version: String,
    pub platform_name: String,
    pub venue: String,
    pub run_mode: String,
    pub host_info_xml: String,
    pub last_rpc_at: time::OffsetDateTime,
}

#[derive(Deserialize)]
pub struct UpdateHostRequest {
    pub venue: Option<String>,
}

pub async fn list_hosts(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<HostResponse>>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let rows = sqlx::query_as::<_, HostResponse>(
        "SELECT id, host_cpid, domain_name, client_version, platform_name, \
         venue, run_mode, last_rpc_at \
         FROM hosts WHERE user_id = $1 ORDER BY last_rpc_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

pub async fn get_host(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<HostDetailResponse>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let row = sqlx::query_as::<_, HostRow>(
        "SELECT id, host_cpid, domain_name, client_version, platform_name, \
         venue, run_mode, host_info_xml, last_rpc_at \
         FROM hosts WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(HostDetailResponse {
        id: row.id,
        host_cpid: row.host_cpid,
        domain_name: row.domain_name,
        client_version: row.client_version,
        platform_name: row.platform_name,
        venue: row.venue,
        run_mode: row.run_mode,
        host_info_xml: row.host_info_xml,
        last_rpc_at: row.last_rpc_at,
    }))
}

pub async fn update_host(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<UpdateHostRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    if let Some(ref venue) = body.venue {
        fam_core::auth::update_host_venue(&state.db, id, user.id, venue)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(sqlx::FromRow)]
struct HostRow {
    id: i64,
    host_cpid: String,
    domain_name: String,
    client_version: String,
    platform_name: String,
    venue: String,
    run_mode: String,
    host_info_xml: String,
    last_rpc_at: time::OffsetDateTime,
}
