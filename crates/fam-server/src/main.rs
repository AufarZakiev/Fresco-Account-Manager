mod background;
mod config;
mod error;
mod routes;
mod state;

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::FamConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = FamConfig::from_env()?;
    tracing::info!(addr = %config.listen_addr, name = %config.server_name, "starting FAM");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&db).await?;
    tracing::info!("database migrations applied");

    let rsa_private_key =
        fam_core::crypto::load_or_generate_private_key(Path::new(&config.private_key_path))?;
    let rsa_public_key = fam_core::crypto::load_or_derive_public_key(
        Path::new(&config.public_key_path),
        &rsa_private_key,
    )?;
    let boinc_public_key_text = fam_core::crypto::public_key_to_boinc_format(&rsa_public_key);
    tracing::info!("RSA keys loaded");

    // Spawn background tasks (session cleanup, stale host cleanup)
    background::spawn_background_tasks(db.clone());

    let state = Arc::new(AppState {
        db,
        config: config.clone(),
        rsa_private_key,
        rsa_public_key,
        boinc_public_key_text,
    });

    // BOINC protocol endpoints
    let boinc_routes = Router::new()
        .route(
            "/get_project_config.php",
            axum::routing::get(routes::project_config::get_project_config),
        )
        .route("/rpc.php", axum::routing::post(routes::rpc::handle_rpc));

    // REST API for web frontend
    let api_routes = Router::new()
        // Auth
        .route(
            "/auth/register",
            axum::routing::post(routes::api::auth::register),
        )
        .route("/auth/login", axum::routing::post(routes::api::auth::login))
        .route(
            "/auth/logout",
            axum::routing::post(routes::api::auth::logout),
        )
        .route("/auth/me", axum::routing::get(routes::api::auth::me))
        // Projects (public)
        .route(
            "/projects",
            axum::routing::get(routes::api::projects::list_projects),
        )
        .route(
            "/projects/{id}",
            axum::routing::get(routes::api::projects::get_project),
        )
        // User projects
        .route(
            "/user/projects",
            axum::routing::get(routes::api::user_projects::list_user_projects)
                .post(routes::api::user_projects::enroll),
        )
        .route(
            "/user/projects/{id}",
            axum::routing::patch(routes::api::user_projects::update_user_project)
                .delete(routes::api::user_projects::leave_project),
        )
        .route(
            "/user/projects/{id}/suspend",
            axum::routing::post(routes::api::user_projects::suspend_project),
        )
        .route(
            "/user/projects/{id}/resume",
            axum::routing::post(routes::api::user_projects::resume_project),
        )
        .route(
            "/user/projects/{id}/detach",
            axum::routing::post(routes::api::user_projects::detach_project),
        )
        // Hosts
        .route(
            "/user/hosts",
            axum::routing::get(routes::api::hosts::list_hosts),
        )
        .route(
            "/user/hosts/{id}",
            axum::routing::get(routes::api::hosts::get_host)
                .patch(routes::api::hosts::update_host),
        )
        // Preferences
        .route(
            "/user/preferences",
            axum::routing::get(routes::api::preferences::get_preferences)
                .put(routes::api::preferences::set_preferences),
        )
        // Stats
        .route(
            "/user/stats",
            axum::routing::get(routes::api::stats::get_user_stats),
        )
        // Password change
        .route(
            "/user/change-password",
            axum::routing::post(routes::api::auth::change_password),
        )
        // Admin
        .route(
            "/admin/users",
            axum::routing::get(routes::api::admin::list_users),
        )
        .route(
            "/admin/stats",
            axum::routing::get(routes::api::admin::get_stats),
        )
        .route(
            "/admin/projects",
            axum::routing::post(routes::api::admin::create_project),
        )
        .route(
            "/admin/projects/{id}",
            axum::routing::put(routes::api::admin::update_project),
        );

    let app = Router::new()
        .route("/health", axum::routing::get(routes::health::health))
        .merge(boinc_routes)
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    tracing::info!("listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
