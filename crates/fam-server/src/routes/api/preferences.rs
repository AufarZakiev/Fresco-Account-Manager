use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::routes::api::auth::require_auth;
use crate::state::AppState;

#[derive(Serialize)]
pub struct PreferencesResponse {
    pub prefs_xml: String,
    pub mod_time: String,
}

#[derive(Deserialize)]
pub struct SetPreferencesRequest {
    pub prefs_xml: String,
}

pub async fn get_preferences(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<PreferencesResponse>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    let prefs = fam_core::auth::get_user_preferences(&state.db, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match prefs {
        Some(p) => Ok(Json(PreferencesResponse {
            prefs_xml: p.prefs_xml,
            mod_time: p.mod_time.to_string(),
        })),
        None => Ok(Json(PreferencesResponse {
            prefs_xml: String::new(),
            mod_time: String::new(),
        })),
    }
}

pub async fn set_preferences(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SetPreferencesRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&state, &headers).await?;

    fam_core::auth::set_user_preferences(&state.db, user.id, &body.prefs_xml)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"ok": true})))
}
