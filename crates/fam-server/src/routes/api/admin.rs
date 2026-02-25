use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::routes::api::auth::require_admin;
use crate::state::AppState;

/// URL of the official BOINC project list.
const BOINC_PROJECT_LIST_URL: &str = "https://boinc.berkeley.edu/project_list.php";

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

/// A project parsed from the BOINC all-projects list.
#[derive(Debug, Default)]
struct BoincListProject {
    name: String,
    url: String,
    web_url: String,
    general_area: String,
    specific_area: String,
    description: String,
    home: String,
    platforms: Vec<String>,
}

fn apply_project_field(proj: &mut BoincListProject, tag: &str, text: String, in_platforms: bool) {
    if text.is_empty() {
        return;
    }
    if in_platforms && tag == "name" {
        proj.platforms.push(text);
    } else {
        match tag {
            "name" => proj.name = text,
            "url" => proj.url = text,
            "web_url" => proj.web_url = text,
            "general_area" => proj.general_area = text,
            "specific_area" => proj.specific_area = text,
            "description" => proj.description = text,
            "home" => proj.home = text,
            _ => {}
        }
    }
}

/// Parse the BOINC project list XML into a vec of projects.
fn parse_boinc_project_list(xml: &str) -> Vec<BoincListProject> {
    let mut reader = Reader::from_str(xml);
    let mut projects = Vec::new();
    let mut current: Option<BoincListProject> = None;
    let mut current_tag = String::new();
    let mut in_platforms = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "project" => {
                        current = Some(BoincListProject::default());
                        in_platforms = false;
                    }
                    "platforms" => in_platforms = true,
                    _ => {}
                }
                current_tag = tag;
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "project" => {
                        if let Some(p) = current.take() {
                            if !p.url.is_empty() && !p.name.is_empty() {
                                projects.push(p);
                            }
                        }
                    }
                    "platforms" => in_platforms = false,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref mut proj) = current {
                    let text = e.unescape().unwrap_or_default().trim().to_string();
                    apply_project_field(proj, &current_tag, text, in_platforms);
                }
            }
            Ok(Event::CData(ref e)) => {
                if let Some(ref mut proj) = current {
                    let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                    apply_project_field(proj, &current_tag, text, in_platforms);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    projects
}

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub total_fetched: usize,
}

/// Import projects from the official BOINC project list.
///
/// `POST /api/admin/projects/import-boinc`
///
/// Fetches `https://boinc.berkeley.edu/project_list.php`, parses the XML,
/// and upserts each project into the database. Existing projects (matched
/// by URL) are updated with the latest metadata; new projects are inserted
/// with a freshly computed URL signature.
pub async fn import_boinc_projects(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ImportResult>, (StatusCode, Json<serde_json::Value>)> {
    require_admin(&state, &headers)
        .await
        .map_err(|s| (s, Json(serde_json::json!({"error": "Forbidden"}))))?;

    // Fetch the BOINC project list
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("HTTP client error: {e}")})),
            )
        })?;

    let xml = client
        .get(BOINC_PROJECT_LIST_URL)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("Failed to fetch BOINC project list: {e}")})),
            )
        })?
        .text()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("Failed to read response: {e}")})),
            )
        })?;

    let projects = parse_boinc_project_list(&xml);
    let total_fetched = projects.len();
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for proj in &projects {
        // Compute URL signature for this project
        let url_signature =
            fam_core::crypto::sign_url(&proj.url, &state.rsa_private_key).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to sign URL: {e}")})),
                )
            })?;

        let home_url = if proj.web_url.is_empty() {
            &proj.url
        } else {
            &proj.web_url
        };

        let result = sqlx::query(
            "INSERT INTO projects (url, name, description, general_area, specific_area, home_url, platforms, url_signature) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (url) DO UPDATE SET \
                name = EXCLUDED.name, \
                description = EXCLUDED.description, \
                general_area = EXCLUDED.general_area, \
                specific_area = EXCLUDED.specific_area, \
                home_url = EXCLUDED.home_url, \
                platforms = EXCLUDED.platforms, \
                url_signature = EXCLUDED.url_signature, \
                updated_at = NOW()",
        )
        .bind(&proj.url)
        .bind(&proj.name)
        .bind(&proj.description)
        .bind(&proj.general_area)
        .bind(&proj.specific_area)
        .bind(home_url)
        .bind(&proj.platforms)
        .bind(&url_signature)
        .execute(&state.db)
        .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::warn!(url = %proj.url, error = %e, "failed to upsert project");
                skipped += 1;
            }
        }
    }

    tracing::info!(
        total_fetched,
        imported,
        skipped,
        "BOINC project list import complete"
    );

    Ok(Json(ImportResult {
        imported,
        skipped,
        total_fetched,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boinc_project_list_basic() {
        let xml = r#"<?xml version="1.0" encoding="ISO-8859-1" ?>
<projects>
<project>
    <name>Einstein@Home</name>
    <url>https://einsteinathome.org/</url>
    <web_url>https://einsteinathome.org/</web_url>
    <general_area>Astronomy/Physics/Chemistry</general_area>
    <specific_area>Gravitational wave detection</specific_area>
    <description>Search for gravitational waves</description>
    <home>UW-Milwaukee</home>
    <platforms>
        <name>windows_x86_64</name>
        <name>x86_64-pc-linux-gnu</name>
    </platforms>
</project>
<project>
    <name>Rosetta@home</name>
    <url>https://boinc.bakerlab.org/rosetta/</url>
    <web_url>https://boinc.bakerlab.org/rosetta/</web_url>
    <general_area>Biology and Medicine</general_area>
    <specific_area>Protein structure prediction</specific_area>
    <description><![CDATA[Determine 3-dimensional shapes of proteins]]></description>
    <home>University of Washington</home>
    <platforms>
        <name>windows_x86_64</name>
    </platforms>
</project>
<account_manager>
    <name>BAM!</name>
    <url>https://bam.boincstats.com/</url>
</account_manager>
</projects>"#;

        let projects = parse_boinc_project_list(xml);
        assert_eq!(projects.len(), 2); // account_managers are not included

        assert_eq!(projects[0].name, "Einstein@Home");
        assert_eq!(projects[0].url, "https://einsteinathome.org/");
        assert_eq!(projects[0].general_area, "Astronomy/Physics/Chemistry");
        assert_eq!(projects[0].specific_area, "Gravitational wave detection");
        assert_eq!(projects[0].description, "Search for gravitational waves");
        assert_eq!(projects[0].home, "UW-Milwaukee");
        assert_eq!(projects[0].platforms, vec!["windows_x86_64", "x86_64-pc-linux-gnu"]);

        assert_eq!(projects[1].name, "Rosetta@home");
        assert_eq!(projects[1].url, "https://boinc.bakerlab.org/rosetta/");
        // CDATA description
        assert_eq!(projects[1].description, "Determine 3-dimensional shapes of proteins");
        assert_eq!(projects[1].platforms, vec!["windows_x86_64"]);
    }

    #[test]
    fn test_parse_boinc_project_list_empty() {
        let xml = r#"<projects></projects>"#;
        let projects = parse_boinc_project_list(xml);
        assert!(projects.is_empty());
    }

    #[test]
    fn test_parse_boinc_project_list_skips_incomplete() {
        // Missing name or url → should be skipped
        let xml = r#"<projects>
<project>
    <url>https://example.com/</url>
</project>
<project>
    <name>Valid Project</name>
    <url>https://valid.com/</url>
</project>
</projects>"#;

        let projects = parse_boinc_project_list(xml);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Valid Project");
    }
}
