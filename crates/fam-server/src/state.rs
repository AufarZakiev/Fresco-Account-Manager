use rsa::{RsaPrivateKey, RsaPublicKey};
use sqlx::PgPool;

use crate::config::FamConfig;

#[allow(dead_code)]
pub struct AppState {
    pub db: PgPool,
    pub config: FamConfig,
    pub rsa_private_key: RsaPrivateKey,
    pub rsa_public_key: RsaPublicKey,
    /// Pre-computed BOINC-format hex string of the public key.
    pub boinc_public_key_text: String,
}
