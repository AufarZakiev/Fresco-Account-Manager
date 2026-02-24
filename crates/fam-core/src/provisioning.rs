use sqlx::PgPool;

use crate::project_rpc::{ProjectRpcClient, ProjectRpcError, ERR_NOT_FOUND};

#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("project RPC error: {0}")]
    ProjectRpc(#[from] ProjectRpcError),
}

/// Provision a user's account on a BOINC project.
///
/// Tries `lookup_account` first. If not found, calls `create_account`.
/// Stores the project authenticator in `user_projects`.
#[allow(clippy::too_many_arguments)]
pub async fn provision_project_account(
    db: &PgPool,
    rpc_client: &ProjectRpcClient,
    user_id: i64,
    project_id: i64,
    project_url: &str,
    email: &str,
    password_hash: &str,
    user_name: &str,
) -> Result<String, ProvisionError> {
    // Try lookup first
    let authenticator = match rpc_client
        .lookup_account(project_url, email, password_hash)
        .await
    {
        Ok(auth) => auth,
        Err(ProjectRpcError::ProjectError { code, .. }) if code == ERR_NOT_FOUND => {
            // Account doesn't exist — create it
            rpc_client
                .create_account(project_url, email, password_hash, user_name)
                .await?
        }
        Err(e) => return Err(e.into()),
    };

    // Store the authenticator
    sqlx::query(
        "UPDATE user_projects SET project_authenticator = $1 \
         WHERE user_id = $2 AND project_id = $3",
    )
    .bind(&authenticator)
    .bind(user_id)
    .bind(project_id)
    .execute(db)
    .await?;

    Ok(authenticator)
}
