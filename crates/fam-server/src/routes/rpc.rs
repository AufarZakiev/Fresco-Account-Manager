use std::sync::Arc;

use axum::extract::State;
use sqlx::PgPool;
use axum::http::header;
use axum::response::IntoResponse;

use fam_core::auth::{
    authenticate_by_password, authenticate_by_token, get_host_venue, get_user_preferences,
    set_user_preferences, upsert_host, AuthError, UpsertHostParams,
};
use fam_core::xml::reply::{build_acct_mgr_reply, build_error_reply, AccountEntry, AcctMgrReply};
use fam_core::xml::request::{parse_acct_mgr_request, RequestProject};

use crate::state::AppState;

/// BOINC error codes
const ERR_BAD_PASSWD: i32 = -206;
const ERR_NOT_FOUND: i32 = -136;
const ERR_XML_PARSE: i32 = -112;

pub async fn handle_rpc(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    let xml = handle_rpc_inner(&state, &body).await;
    ([(header::CONTENT_TYPE, "application/xml")], xml)
}

async fn handle_rpc_inner(state: &AppState, body: &str) -> String {
    // 1. Parse the XML request
    let request = match parse_acct_mgr_request(body) {
        Ok(req) => req,
        Err(e) => {
            tracing::warn!(error = %e, "failed to parse AM request");
            return build_error_reply(ERR_XML_PARSE, "Invalid XML request");
        }
    };

    // 2. Authenticate the user
    let user = if let Some(ref auth) = request.authenticator {
        match authenticate_by_token(&state.db, auth).await {
            Ok(user) => user,
            Err(AuthError::InvalidCredentials) => {
                return build_error_reply(ERR_BAD_PASSWD, "Invalid authenticator");
            }
            Err(e) => {
                tracing::error!(error = %e, "auth error");
                return build_error_reply(-1, "Internal error");
            }
        }
    } else if let (Some(ref name), Some(ref password_hash)) =
        (&request.name, &request.password_hash)
    {
        match authenticate_by_password(&state.db, name, password_hash).await {
            Ok(user) => user,
            Err(AuthError::UserNotFound) => {
                return build_error_reply(
                    ERR_NOT_FOUND,
                    "Account not found. Please register at the FAM website first.",
                );
            }
            Err(AuthError::InvalidCredentials) => {
                return build_error_reply(ERR_BAD_PASSWD, "Invalid password");
            }
            Err(e) => {
                tracing::error!(error = %e, "auth error");
                return build_error_reply(-1, "Internal error");
            }
        }
    } else {
        return build_error_reply(ERR_BAD_PASSWD, "No credentials provided");
    };

    tracing::info!(user_id = user.id, email = %user.email, host_cpid = %request.host_cpid, "AM RPC");

    // 3. Upsert host (with expanded fields)
    let host_id = match upsert_host(
        &state.db,
        &UpsertHostParams {
            user_id: user.id,
            host_cpid: &request.host_cpid,
            domain_name: request.domain_name.as_deref().unwrap_or(""),
            client_version: request.client_version.as_deref().unwrap_or(""),
            platform_name: request.platform_name.as_deref().unwrap_or(""),
            host_info_xml: request.host_info.as_deref().unwrap_or(""),
            run_mode: request.run_mode.as_deref().unwrap_or(""),
            opaque_data: request.opaque.as_deref().unwrap_or(""),
        },
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "host upsert error");
            return build_error_reply(-1, "Internal error");
        }
    };

    // 4. Sync project authenticators from client's project list
    sync_project_authenticators(&state.db, user.id, &request.projects).await;

    // 5. Fetch user's enrolled projects
    let accounts = match fetch_user_accounts(state, user.id).await {
        Ok(accounts) => accounts,
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch accounts");
            vec![]
        }
    };

    // 6. Handle global preferences
    // If client sends newer prefs, store them. If server has newer prefs, include in reply.
    let global_preferences = handle_preferences(state, user.id, &request).await;

    // 7. Get host venue
    let host_venue = match get_host_venue(&state.db, host_id).await {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    };

    // 8. Build reply
    let reply = AcctMgrReply {
        name: state.config.server_name.clone(),
        authenticator: Some(user.authenticator.clone()),
        user_name: Some(user.name.clone()),
        signing_key: state.boinc_public_key_text.clone(),
        repeat_sec: state.config.repeat_sec,
        accounts,
        global_preferences,
        host_venue,
        opaque: Some(format!("<fam_host_id>{host_id}</fam_host_id>")),
        ..Default::default()
    };

    build_acct_mgr_reply(&reply)
}

/// Handle global preferences sync between client and server.
///
/// The BOINC protocol says:
/// - If client sends `<global_preferences>` with a `<mod_time>` newer than what we have, store it.
/// - If server has preferences with a `<mod_time>` newer than the client's working prefs, return them.
async fn handle_preferences(
    state: &AppState,
    user_id: i64,
    request: &fam_core::xml::request::AcctMgrRequest,
) -> Option<String> {
    let stored = get_user_preferences(&state.db, user_id).await.ok().flatten();

    // If client sends global_preferences, store them (the client's prefs are authoritative when sent)
    if let Some(ref client_prefs) = request.global_preferences {
        if !client_prefs.trim().is_empty() {
            if let Err(e) = set_user_preferences(&state.db, user_id, client_prefs).await {
                tracing::warn!(error = %e, "failed to store user preferences");
            }
            // Don't echo back prefs we just received from the client
            return None;
        }
    }

    // If we have stored preferences, send them to the client
    if let Some(prefs) = stored {
        if !prefs.prefs_xml.is_empty() {
            return Some(prefs.prefs_xml);
        }
    }

    None
}

fn normalize_project_url(url: &str) -> String {
    url.trim_end_matches('/').to_lowercase()
}

/// Capture project authenticators reported by the BOINC client.
///
/// The client sends `account_key` for each project it's attached to. We store it in
/// `user_projects.project_authenticator` so FAM can later call project server APIs and
/// include the project in AM reply `<account>` entries.
///
/// Safety: `WHERE project_authenticator = ''` ensures we never overwrite a valid authenticator.
async fn sync_project_authenticators(
    db: &PgPool,
    user_id: i64,
    client_projects: &[RequestProject],
) {
    // Load all active projects once to avoid N+1 queries
    let db_projects = match sqlx::query_as::<_, ProjectUrlRow>(
        "SELECT id, url FROM projects WHERE is_active = true",
    )
    .fetch_all(db)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "failed to load projects for auth sync");
            return;
        }
    };

    // Build a lookup: normalized_url → project_id
    let url_to_id: std::collections::HashMap<String, i64> = db_projects
        .iter()
        .map(|p| (normalize_project_url(&p.url), p.id))
        .collect();

    for cp in client_projects {
        let account_key = match cp.account_key.as_deref() {
            Some(key) if !key.is_empty() => key,
            _ => continue,
        };

        let normalized = normalize_project_url(&cp.url);
        let project_id = match url_to_id.get(&normalized) {
            Some(&id) => id,
            None => continue,
        };

        let result = if cp.attached_via_acct_mgr {
            // Managed project: upsert (create row if needed)
            sqlx::query(
                "INSERT INTO user_projects (user_id, project_id, project_authenticator) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (user_id, project_id) \
                 DO UPDATE SET project_authenticator = EXCLUDED.project_authenticator \
                 WHERE user_projects.project_authenticator = ''",
            )
            .bind(user_id)
            .bind(project_id)
            .bind(account_key)
            .execute(db)
            .await
        } else {
            // Manually attached: only update existing row, don't auto-enroll
            sqlx::query(
                "UPDATE user_projects \
                 SET project_authenticator = $1 \
                 WHERE user_id = $2 AND project_id = $3 AND project_authenticator = ''",
            )
            .bind(account_key)
            .bind(user_id)
            .bind(project_id)
            .execute(db)
            .await
        };

        if let Err(e) = result {
            tracing::warn!(
                error = %e,
                project_id,
                "failed to sync project authenticator"
            );
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectUrlRow {
    id: i64,
    url: String,
}

async fn fetch_user_accounts(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<AccountEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserProjectRow>(
        "SELECT up.id, up.project_authenticator, up.resource_share, up.suspended, \
         up.dont_request_more_work, up.pending_detach, up.detach_when_done, up.no_rsc, \
         up.force_update, \
         p.url, p.url_signature \
         FROM user_projects up \
         JOIN projects p ON p.id = up.project_id \
         WHERE up.user_id = $1 AND p.is_active = true AND up.project_authenticator != ''",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let mut accounts = Vec::new();
    let mut force_update_ids = Vec::new();

    for row in rows {
        // If pending_detach, send detach signal and delete the row after this RPC
        if row.pending_detach {
            accounts.push(AccountEntry {
                url: row.url.clone(),
                url_signature: row.url_signature.clone(),
                authenticator: row.project_authenticator.clone(),
                resource_share: row.resource_share,
                suspend: false,
                dont_request_more_work: false,
                detach_when_done: false,
                detach: true,
                update: false,
                no_rsc: vec![],
                abort_not_started: false,
            });

            // Delete the user_project row after sending detach
            let _ = sqlx::query("DELETE FROM user_projects WHERE id = $1")
                .bind(row.id)
                .execute(&state.db)
                .await;

            continue;
        }

        if row.force_update {
            force_update_ids.push(row.id);
        }

        accounts.push(AccountEntry {
            url: row.url,
            url_signature: row.url_signature,
            authenticator: row.project_authenticator,
            resource_share: row.resource_share,
            suspend: row.suspended,
            dont_request_more_work: row.dont_request_more_work,
            detach_when_done: row.detach_when_done,
            detach: false,
            update: row.force_update,
            no_rsc: row.no_rsc,
            abort_not_started: false,
        });
    }

    // Reset force_update flags after building the reply
    if !force_update_ids.is_empty() {
        let _ = sqlx::query(
            "UPDATE user_projects SET force_update = false WHERE id = ANY($1)",
        )
        .bind(&force_update_ids)
        .execute(&state.db)
        .await;
    }

    Ok(accounts)
}

/// Clean up detached projects that the client has confirmed.
/// Called when we see that a project previously marked as pending_detach
/// is no longer reported by the client (meaning it detached successfully).
async fn _cleanup_confirmed_detaches(
    state: &AppState,
    user_id: i64,
    client_project_urls: &[String],
) {
    // Any user_project with pending_detach=true whose URL is NOT in the client's list
    // has been successfully detached — delete it.
    let result = sqlx::query(
        "DELETE FROM user_projects up \
         USING projects p \
         WHERE up.project_id = p.id \
         AND up.user_id = $1 \
         AND up.pending_detach = true \
         AND p.url != ALL($2)",
    )
    .bind(user_id)
    .bind(client_project_urls)
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "failed to cleanup detached projects");
    }
}

#[derive(sqlx::FromRow)]
struct UserProjectRow {
    id: i64,
    project_authenticator: String,
    resource_share: f32,
    suspended: bool,
    dont_request_more_work: bool,
    pending_detach: bool,
    detach_when_done: bool,
    no_rsc: Vec<String>,
    force_update: bool,
    url: String,
    url_signature: String,
}
