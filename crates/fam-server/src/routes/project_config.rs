use std::sync::Arc;

use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;

use crate::state::AppState;

pub async fn get_project_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let xml = fam_core::xml::project_config::build_project_config(&state.config.server_name, 6);
    ([(header::CONTENT_TYPE, "application/xml")], xml)
}
