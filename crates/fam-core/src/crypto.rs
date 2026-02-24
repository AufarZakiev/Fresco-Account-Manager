use std::path::Path;

use md5::{Digest, Md5};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};

/// BOINC uses 1024-bit RSA keys.
const RSA_BITS: usize = 1024;
/// Modulus length in bytes (1024 / 8).
const MODULUS_LEN: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("RSA error: {0}")]
    Rsa(#[from] rsa::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PKCS8 error: {0}")]
    Pkcs8Encode(String),
    #[error("key decode error: {0}")]
    KeyDecode(String),
}

/// Load an RSA private key from a PEM file, or generate and save a new one.
pub fn load_or_generate_private_key(path: &Path) -> Result<RsaPrivateKey, CryptoError> {
    if path.exists() {
        let pem = std::fs::read_to_string(path)?;
        RsaPrivateKey::from_pkcs8_pem(&pem).map_err(|e| CryptoError::KeyDecode(e.to_string()))
    } else {
        tracing::info!("generating new {}-bit RSA key pair", RSA_BITS);
        let mut rng = rand::thread_rng();
        let key = RsaPrivateKey::new(&mut rng, RSA_BITS)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pem = key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| CryptoError::Pkcs8Encode(e.to_string()))?;
        std::fs::write(path, pem.as_bytes())?;
        tracing::info!("saved private key to {}", path.display());
        Ok(key)
    }
}

/// Load an RSA public key from PEM, or derive from private key and save.
pub fn load_or_derive_public_key(
    path: &Path,
    private_key: &RsaPrivateKey,
) -> Result<RsaPublicKey, CryptoError> {
    if path.exists() {
        let pem = std::fs::read_to_string(path)?;
        RsaPublicKey::from_public_key_pem(&pem).map_err(|e| CryptoError::KeyDecode(e.to_string()))
    } else {
        let pub_key = private_key.to_public_key();
        let pem = pub_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| CryptoError::Pkcs8Encode(e.to_string()))?;
        std::fs::write(path, pem.as_bytes())?;
        tracing::info!("saved public key to {}", path.display());
        Ok(pub_key)
    }
}

/// Serialize an RSA public key to BOINC's hex format.
///
/// Format:
/// ```text
/// 1024\n
/// <hex encoded R_RSA_PUBLIC_KEY struct>\n
/// .\n
/// ```
///
/// The struct layout is: 2-byte `bits` (LE) + 128-byte modulus (BE, zero-padded right-justified)
/// + 128-byte exponent (BE, zero-padded right-justified) = 258 bytes total.
pub fn public_key_to_boinc_format(key: &RsaPublicKey) -> String {
    let mut key_bytes = Vec::with_capacity(258);

    // 2-byte bits field (little-endian)
    let bits = key.n().bits() as u16;
    key_bytes.extend_from_slice(&bits.to_le_bytes());

    // 128-byte modulus, right-justified (big-endian, zero-padded)
    let n_bytes = key.n().to_bytes_be();
    let mut modulus = [0u8; MODULUS_LEN];
    let offset = MODULUS_LEN.saturating_sub(n_bytes.len());
    modulus[offset..].copy_from_slice(&n_bytes[..MODULUS_LEN.min(n_bytes.len())]);
    key_bytes.extend_from_slice(&modulus);

    // 128-byte exponent, right-justified (big-endian, zero-padded)
    let e_bytes = key.e().to_bytes_be();
    let mut exponent = [0u8; MODULUS_LEN];
    let offset = MODULUS_LEN.saturating_sub(e_bytes.len());
    exponent[offset..].copy_from_slice(&e_bytes[..MODULUS_LEN.min(e_bytes.len())]);
    key_bytes.extend_from_slice(&exponent);

    // Format: bits decimal, then hex data with 64-char lines, then ".\n"
    let mut out = format!("{}\n", bits);
    for (i, byte) in key_bytes.iter().enumerate() {
        out.push_str(&format!("{:02x}", byte));
        if i % 32 == 31 {
            out.push('\n');
        }
    }
    // 258 bytes: 258 % 32 = 2, so last line has remainder
    if !key_bytes.len().is_multiple_of(32) {
        out.push('\n');
    }
    out.push_str(".\n");
    out
}

/// Sign a URL string using BOINC's signing algorithm.
///
/// Algorithm:
/// 1. Compute MD5 of the raw URL bytes → 32-char lowercase hex string
/// 2. RSA-private-encrypt the hex string bytes using PKCS#1 v1.5 → ~128 bytes
/// 3. Hex-encode the signature with newlines every 64 chars, terminated by ".\n"
pub fn sign_url(url: &str, private_key: &RsaPrivateKey) -> Result<String, CryptoError> {
    // Step 1: MD5 hash of the URL → 32-char hex string
    let mut hasher = Md5::new();
    hasher.update(url.as_bytes());
    let md5_hex = format!("{:032x}", hasher.finalize());

    // Step 2: RSA-private-encrypt the MD5 hex string (not the binary digest!)
    // BOINC's sign_block does: encrypt_private(key, md5_hex_bytes) using PKCS1 padding.
    // Pkcs1v15Sign::new_unprefixed() gives us raw PKCS#1 v1.5 without digest algorithm prefix.
    let signature = private_key.sign(Pkcs1v15Sign::new_unprefixed(), md5_hex.as_bytes())?;

    // Step 3: Hex-encode with line breaks every 64 chars
    Ok(format_boinc_signature(&signature))
}

/// Format binary signature data as BOINC hex string.
///
/// Every 32 bytes (64 hex chars) gets a newline. Terminated with ".\n".
fn format_boinc_signature(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, byte) in data.iter().enumerate() {
        out.push_str(&format!("{:02x}", byte));
        if i % 32 == 31 {
            out.push('\n');
        }
    }
    if !data.len().is_multiple_of(32) {
        out.push('\n');
    }
    out.push_str(".\n");
    out
}

/// Verify a URL signature against a public key (for testing).
pub fn verify_url_signature(
    url: &str,
    signature_hex: &str,
    public_key: &RsaPublicKey,
) -> Result<bool, CryptoError> {
    // Parse hex signature (skip whitespace and dot terminator)
    let hex_clean: String = signature_hex
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    let sig_bytes =
        hex::decode(&hex_clean).map_err(|_| CryptoError::Rsa(rsa::Error::Verification))?;

    // Compute MD5 of URL
    let mut hasher = Md5::new();
    hasher.update(url.as_bytes());
    let md5_hex = format!("{:032x}", hasher.finalize());

    // Verify
    match public_key.verify(
        Pkcs1v15Sign::new_unprefixed(),
        md5_hex.as_bytes(),
        &sig_bytes,
    ) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();
        let public_key = private_key.to_public_key();

        let url = "https://einsteinathome.org/";
        let signature = sign_url(url, &private_key).unwrap();

        // Signature should be hex with line breaks
        assert!(signature.ends_with(".\n"));
        let hex_chars: usize = signature.chars().filter(|c| c.is_ascii_hexdigit()).count();
        assert!(hex_chars > 0);

        // Verify
        assert!(verify_url_signature(url, &signature, &public_key).unwrap());

        // Wrong URL should fail
        assert!(!verify_url_signature("https://wrong.com/", &signature, &public_key).unwrap());
    }

    #[test]
    fn test_boinc_key_format() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();
        let public_key = private_key.to_public_key();

        let formatted = public_key_to_boinc_format(&public_key);

        // First line should be the bit count
        let first_line = formatted.lines().next().unwrap();
        let bits: u16 = first_line.parse().unwrap();
        assert!(bits >= 1023 && bits <= 1024); // RSA-1024 keys are 1023 or 1024 bits

        // Should end with dot-newline
        assert!(formatted.ends_with(".\n"));

        // Hex data: 258 bytes = 516 hex chars
        let hex_chars: usize = formatted
            .lines()
            .skip(1) // skip bits line
            .take_while(|l| *l != ".")
            .flat_map(|l| l.chars())
            .filter(|c| c.is_ascii_hexdigit())
            .count();
        assert_eq!(hex_chars, 258 * 2);
    }

    #[test]
    fn test_key_gen_and_load() {
        let dir = std::env::temp_dir().join("fam_test_keys");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let priv_path = dir.join("test_private.pem");
        let pub_path = dir.join("test_public.pem");

        // Generate
        let priv_key = load_or_generate_private_key(&priv_path).unwrap();
        let pub_key = load_or_derive_public_key(&pub_path, &priv_key).unwrap();

        // Reload
        let priv_key2 = load_or_generate_private_key(&priv_path).unwrap();
        let pub_key2 = load_or_derive_public_key(&pub_path, &priv_key2).unwrap();

        assert_eq!(pub_key.n(), pub_key2.n());
        assert_eq!(pub_key.e(), pub_key2.e());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sign_different_urls_produce_different_sigs() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();

        let sig1 = sign_url("https://example.com/", &private_key).unwrap();
        let sig2 = sign_url("https://other.com/", &private_key).unwrap();
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_same_url_same_sig() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();

        let sig1 = sign_url("https://example.com/", &private_key).unwrap();
        let sig2 = sign_url("https://example.com/", &private_key).unwrap();
        // PKCS#1 v1.5 is deterministic
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_format_dot_terminated() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();

        let sig = sign_url("https://test.com/", &private_key).unwrap();
        assert!(sig.ends_with(".\n"));
        // 128-byte RSA-1024 signature = 256 hex chars = 4 lines of 64
        let hex_chars: usize = sig.chars().filter(|c| c.is_ascii_hexdigit()).count();
        assert_eq!(hex_chars, 256); // 128 bytes * 2
    }

    #[test]
    fn test_boinc_key_format_struct_size() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();
        let public_key = private_key.to_public_key();

        let formatted = public_key_to_boinc_format(&public_key);

        // Decode the hex data (skip first line = bits, skip last line = ".")
        let hex_data: String = formatted
            .lines()
            .skip(1)
            .take_while(|l| *l != ".")
            .collect();
        let hex_chars: usize = hex_data.chars().filter(|c| c.is_ascii_hexdigit()).count();

        // 2 (bits) + 128 (modulus) + 128 (exponent) = 258 bytes = 516 hex chars
        assert_eq!(hex_chars, 516);

        // Parse back the bits field (first 2 bytes, little-endian)
        let bytes = hex::decode(&hex_data.chars().filter(|c| c.is_ascii_hexdigit()).take(4).collect::<String>()).unwrap();
        let bits = u16::from_le_bytes([bytes[0], bytes[1]]);
        assert!(bits >= 1023 && bits <= 1024);
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let mut rng = rand::thread_rng();
        let key1 = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();
        let key2 = RsaPrivateKey::new(&mut rng, RSA_BITS).unwrap();
        let pub2 = key2.to_public_key();

        let url = "https://example.com/";
        let sig = sign_url(url, &key1).unwrap();

        // Signature from key1 should NOT verify with key2's public key
        assert!(!verify_url_signature(url, &sig, &pub2).unwrap());
    }

    #[test]
    fn test_format_boinc_signature() {
        // Small test: 32 bytes = exactly one line
        let data = vec![0xABu8; 32];
        let formatted = format_boinc_signature(&data);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 2); // one data line + "."
        assert_eq!(lines[0].len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(lines[1], ".");

        // 33 bytes = one full line + partial line
        let data = vec![0xCDu8; 33];
        let formatted = format_boinc_signature(&data);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 3); // one full line + partial + "."
    }
}
