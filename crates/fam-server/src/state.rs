use rsa::{RsaPrivateKey, RsaPublicKey};
use sqlx::PgPool;

use fam_core::project_rpc::ProjectRpcClient;

use crate::config::FamConfig;

#[allow(dead_code)]
pub struct AppState {
    pub db: PgPool,
    pub config: FamConfig,
    pub rsa_private_key: RsaPrivateKey,
    pub rsa_public_key: RsaPublicKey,
    /// Pre-computed BOINC-format hex string of the public key.
    pub boinc_public_key_text: String,
    /// HTTP client for BOINC project Web RPCs. In mock mode, intercepts all calls.
    pub project_rpc: ProjectRpcClient,
}
