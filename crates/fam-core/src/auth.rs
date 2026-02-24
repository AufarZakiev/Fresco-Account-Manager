use md5::{Digest, Md5};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("user not found")]
    UserNotFound,
}

/// Authenticate a user via authenticator token.
pub async fn authenticate_by_token(db: &PgPool, authenticator: &str) -> Result<User, AuthError> {
    sqlx::query_as::<_, User>("SELECT id, email, name, password_hash, authenticator, country, is_admin FROM users WHERE authenticator = $1")
        .bind(authenticator)
        .fetch_optional(db)
        .await?
        .ok_or(AuthError::InvalidCredentials)
}

/// Authenticate a user via name (email) + password hash.
pub async fn authenticate_by_password(
    db: &PgPool,
    login_name: &str,
    password_hash: &str,
) -> Result<User, AuthError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, password_hash, authenticator, country, is_admin FROM users WHERE email = $1",
    )
    .bind(login_name)
    .fetch_optional(db)
    .await?
    .ok_or(AuthError::UserNotFound)?;

    if user.password_hash != password_hash {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}

/// Register a new user. Returns the created user.
pub async fn register_user(
    db: &PgPool,
    email: &str,
    name: &str,
    password: &str,
) -> Result<User, AuthError> {
    let password_hash = compute_password_hash(password, email);
    let authenticator = Uuid::new_v4().to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, name, password_hash, authenticator) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, email, name, password_hash, authenticator, country, is_admin",
    )
    .bind(email)
    .bind(name)
    .bind(&password_hash)
    .bind(&authenticator)
    .fetch_one(db)
    .await?;

    Ok(user)
}

/// Compute the BOINC password hash: MD5(password + lowercase(email)).
pub fn compute_password_hash(password: &str, email: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    hasher.update(email.to_lowercase().as_bytes());
    format!("{:032x}", hasher.finalize())
}

/// Parameters for upserting a host.
pub struct UpsertHostParams<'a> {
    pub user_id: i64,
    pub host_cpid: &'a str,
    pub domain_name: &'a str,
    pub client_version: &'a str,
    pub platform_name: &'a str,
    pub host_info_xml: &'a str,
    pub run_mode: &'a str,
    pub opaque_data: &'a str,
}

/// Upsert a host record, returning the host ID.
pub async fn upsert_host(db: &PgPool, params: &UpsertHostParams<'_>) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "INSERT INTO hosts (user_id, host_cpid, domain_name, client_version, platform_name, \
         host_info_xml, run_mode, opaque_data, last_rpc_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW()) \
         ON CONFLICT (user_id, host_cpid) \
         DO UPDATE SET domain_name = $3, client_version = $4, platform_name = $5, \
         host_info_xml = $6, run_mode = $7, opaque_data = $8, last_rpc_at = NOW() \
         RETURNING id",
    )
    .bind(params.user_id)
    .bind(params.host_cpid)
    .bind(params.domain_name)
    .bind(params.client_version)
    .bind(params.platform_name)
    .bind(params.host_info_xml)
    .bind(params.run_mode)
    .bind(params.opaque_data)
    .fetch_one(db)
    .await?;

    Ok(row)
}

/// Get the venue for a specific host.
pub async fn get_host_venue(db: &PgPool, host_id: i64) -> Result<String, sqlx::Error> {
    let venue = sqlx::query_scalar::<_, String>("SELECT venue FROM hosts WHERE id = $1")
        .bind(host_id)
        .fetch_optional(db)
        .await?
        .unwrap_or_default();
    Ok(venue)
}

/// Update the venue for a host.
pub async fn update_host_venue(
    db: &PgPool,
    host_id: i64,
    user_id: i64,
    venue: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("UPDATE hosts SET venue = $1 WHERE id = $2 AND user_id = $3")
            .bind(venue)
            .bind(host_id)
            .bind(user_id)
            .execute(db)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Get user preferences.
pub async fn get_user_preferences(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<crate::models::UserPreferences>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::UserPreferences>(
        "SELECT id, user_id, prefs_xml, mod_time FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

/// Set user preferences (upsert).
pub async fn set_user_preferences(
    db: &PgPool,
    user_id: i64,
    prefs_xml: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_preferences (user_id, prefs_xml, mod_time) \
         VALUES ($1, $2, NOW()) \
         ON CONFLICT (user_id) \
         DO UPDATE SET prefs_xml = $2, mod_time = NOW()",
    )
    .bind(user_id)
    .bind(prefs_xml)
    .execute(db)
    .await?;
    Ok(())
}

/// Change user password. Returns new password_hash.
pub async fn change_password(
    db: &PgPool,
    user_id: i64,
    email: &str,
    old_password: &str,
    new_password: &str,
) -> Result<String, AuthError> {
    let old_hash = compute_password_hash(old_password, email);
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, password_hash, authenticator, country, is_admin FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or(AuthError::UserNotFound)?;

    if user.password_hash != old_hash {
        return Err(AuthError::InvalidCredentials);
    }

    let new_hash = compute_password_hash(new_password, email);

    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_hash)
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(new_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash() {
        // BOINC convention: MD5(password + lowercase(email))
        let hash = compute_password_hash("mypassword", "User@Example.COM");
        // Should be MD5("mypassword" + "user@example.com")
        let mut hasher = Md5::new();
        hasher.update(b"mypassworduser@example.com");
        let expected = format!("{:032x}", hasher.finalize());
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_password_hash_is_32_hex_chars() {
        let hash = compute_password_hash("test", "test@example.com");
        assert_eq!(hash.len(), 32);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_password_hash_case_insensitive_email() {
        let hash1 = compute_password_hash("pass", "USER@EXAMPLE.COM");
        let hash2 = compute_password_hash("pass", "user@example.com");
        let hash3 = compute_password_hash("pass", "User@Example.Com");
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_password_hash_different_passwords() {
        let hash1 = compute_password_hash("password1", "user@example.com");
        let hash2 = compute_password_hash("password2", "user@example.com");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_password_hash_different_emails() {
        let hash1 = compute_password_hash("password", "user1@example.com");
        let hash2 = compute_password_hash("password", "user2@example.com");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_password_hash_empty_password() {
        let hash = compute_password_hash("", "user@example.com");
        // MD5("" + "user@example.com") = MD5("user@example.com")
        let mut hasher = Md5::new();
        hasher.update(b"user@example.com");
        let expected = format!("{:032x}", hasher.finalize());
        assert_eq!(hash, expected);
    }
}
