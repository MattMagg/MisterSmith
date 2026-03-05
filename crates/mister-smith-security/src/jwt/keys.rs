//! Key loading for JWT signing and verification.

use crate::config::KeySource;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use mister_smith_core::SecurityError;

/// Load an encoding (signing) key from the configured key source.
pub(crate) fn load_encoding_key(
    source: &KeySource,
    algorithm: Algorithm,
) -> Result<EncodingKey, SecurityError> {
    match source {
        KeySource::Hmac { secret } => {
            validate_hmac_algorithm(algorithm)?;
            Ok(EncodingKey::from_secret(secret))
        }
        KeySource::RsaPem { private_pem, .. } => {
            validate_rsa_algorithm(algorithm)?;
            let pem = std::fs::read(private_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read RSA private key {}: {e}",
                    private_pem.display()
                ))
            })?;
            EncodingKey::from_rsa_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid RSA private key: {e}"))
            })
        }
        KeySource::EcPem { private_pem, .. } => {
            validate_ec_algorithm(algorithm)?;
            let pem = std::fs::read(private_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read EC private key {}: {e}",
                    private_pem.display()
                ))
            })?;
            EncodingKey::from_ec_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid EC private key: {e}"))
            })
        }
        KeySource::EdPem { private_pem, .. } => {
            validate_ed_algorithm(algorithm)?;
            let pem = std::fs::read(private_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read Ed private key {}: {e}",
                    private_pem.display()
                ))
            })?;
            EncodingKey::from_ed_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid Ed private key: {e}"))
            })
        }
    }
}

/// Load a decoding (verification) key from the configured key source.
pub(crate) fn load_decoding_key(
    source: &KeySource,
    algorithm: Algorithm,
) -> Result<DecodingKey, SecurityError> {
    match source {
        KeySource::Hmac { secret } => {
            validate_hmac_algorithm(algorithm)?;
            Ok(DecodingKey::from_secret(secret))
        }
        KeySource::RsaPem { public_pem, .. } => {
            validate_rsa_algorithm(algorithm)?;
            let pem = std::fs::read(public_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read RSA public key {}: {e}",
                    public_pem.display()
                ))
            })?;
            DecodingKey::from_rsa_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid RSA public key: {e}"))
            })
        }
        KeySource::EcPem { public_pem, .. } => {
            validate_ec_algorithm(algorithm)?;
            let pem = std::fs::read(public_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read EC public key {}: {e}",
                    public_pem.display()
                ))
            })?;
            DecodingKey::from_ec_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid EC public key: {e}"))
            })
        }
        KeySource::EdPem { public_pem, .. } => {
            validate_ed_algorithm(algorithm)?;
            let pem = std::fs::read(public_pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!(
                    "failed to read Ed public key {}: {e}",
                    public_pem.display()
                ))
            })?;
            DecodingKey::from_ed_pem(&pem).map_err(|e| {
                SecurityError::KeyLoadFailed(format!("invalid Ed public key: {e}"))
            })
        }
    }
}

fn validate_hmac_algorithm(alg: Algorithm) -> Result<(), SecurityError> {
    match alg {
        Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => Ok(()),
        _ => Err(SecurityError::KeyLoadFailed(format!(
            "HMAC key source requires HS* algorithm, got {alg:?}"
        ))),
    }
}

fn validate_rsa_algorithm(alg: Algorithm) -> Result<(), SecurityError> {
    match alg {
        Algorithm::RS256
        | Algorithm::RS384
        | Algorithm::RS512
        | Algorithm::PS256
        | Algorithm::PS384
        | Algorithm::PS512 => Ok(()),
        _ => Err(SecurityError::KeyLoadFailed(format!(
            "RSA key source requires RS*/PS* algorithm, got {alg:?}"
        ))),
    }
}

fn validate_ec_algorithm(alg: Algorithm) -> Result<(), SecurityError> {
    match alg {
        Algorithm::ES256 | Algorithm::ES384 => Ok(()),
        _ => Err(SecurityError::KeyLoadFailed(format!(
            "EC key source requires ES* algorithm, got {alg:?}"
        ))),
    }
}

fn validate_ed_algorithm(alg: Algorithm) -> Result<(), SecurityError> {
    match alg {
        Algorithm::EdDSA => Ok(()),
        _ => Err(SecurityError::KeyLoadFailed(format!(
            "Ed key source requires EdDSA algorithm, got {alg:?}"
        ))),
    }
}
