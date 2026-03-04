use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub authenticator: String,
    pub country: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Host {
    pub id: i64,
    pub user_id: i64,
    pub host_cpid: String,
    pub domain_name: String,
    pub client_version: String,
    pub platform_name: String,
    pub venue: String,
    pub host_info_xml: String,
    pub run_mode: String,
    pub opaque_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreferences {
    pub id: i64,
    pub user_id: i64,
    pub prefs_xml: String,
    pub mod_time: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: i64,
    pub url: String,
    pub name: String,
    pub description: String,
    pub general_area: String,
    pub specific_area: String,
    pub home_url: String,
    pub is_active: bool,
    pub url_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProject {
    pub id: i64,
    pub user_id: i64,
    pub project_id: i64,
    pub project_authenticator: String,
    pub resource_share: f32,
    pub suspended: bool,
    pub dont_request_more_work: bool,
    pub pending_detach: bool,
    pub detach_when_done: bool,
    pub is_weak_auth: bool,
    pub last_error: Option<String>,
    pub consecutive_failures: i32,
    pub no_rsc: Vec<String>,
    pub force_update: bool,
    pub total_credit: f32,
    pub recent_credit: f32,
    pub last_info_sync: Option<time::OffsetDateTime>,
}
