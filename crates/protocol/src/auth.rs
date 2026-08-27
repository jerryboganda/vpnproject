//! Cryptographic Authentication & Token Verification

use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Generate a cryptographically secure 256-bit hexadecimal session token.
pub fn generate_secure_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Constant-time comparison of authentication tokens to prevent timing side-channels.
pub fn verify_auth_token(expected: &str, candidate: &str) -> bool {
    let exp_bytes = expected.as_bytes();
    let cand_bytes = candidate.as_bytes();

    if exp_bytes.len() != cand_bytes.len() {
        return false;
    }

    exp_bytes.ct_eq(cand_bytes).into()
}

/// Compute HMAC-SHA256 proof for challenge-response authentication.
pub fn compute_challenge_proof(secret: &str, nonce: &str, challenge: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(nonce.as_bytes());
    mac.update(b":");
    mac.update(challenge.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Compute HMAC-SHA256 tag over arbitrary data.
pub fn compute_hmac_tag(secret: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

/// Constant-time verification of an HMAC-SHA256 tag over arbitrary data.
pub fn verify_hmac_tag(secret: &[u8], data: &[u8], expected_tag_hex: &str) -> bool {
    let computed_hex = compute_hmac_tag(secret, data);
    verify_auth_token(&computed_hex, expected_tag_hex)
}

pub mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let bytes = data.as_ref();
        let mut hex = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_constant_time_eq() {
        let token = generate_secure_token();
        assert_eq!(token.len(), 64);
        assert!(verify_auth_token(&token, &token));
        assert!(!verify_auth_token(&token, "wrong-token"));
    }

    #[test]
    fn test_challenge_proof() {
        let secret = "test-secret-key";
        let nonce = "client-nonce-123";
        let challenge = "server-challenge-456";

        let proof1 = compute_challenge_proof(secret, nonce, challenge);
        let proof2 = compute_challenge_proof(secret, nonce, challenge);
        assert_eq!(proof1, proof2);

        let proof3 = compute_challenge_proof(secret, "different-nonce", challenge);
        assert_ne!(proof1, proof3);
    }
}
