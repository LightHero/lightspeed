use base64::{Engine as _, engine::general_purpose};
use sha2::Digest;
use subtle::ConstantTimeEq;

/// Hashing service
#[derive(Clone, Default)]
pub struct LsHashService {}

impl LsHashService {

    /// Create a new hash service
    pub fn new() -> Self {
        Self::default()
    }

    /// Hash `text`
    pub fn hash(&self, text: &str) -> String {
        let mut hasher = sha2::Sha256::default();
        hasher.update(text);
        let result = hasher.finalize();
        general_purpose::STANDARD_NO_PAD.encode(result)
    }

    /// Verify that hashing `text` produces `expected_hash`.
    ///
    /// `constant_time` selects the comparison strategy:
    /// - `false` — Faster, but may leak
    ///   information through timing side-channels.
    /// - `true` — constant-time comparison. Use
    ///   this whenever the hash is a secret-derived value an attacker could
    ///   probe (validation codes, MACs, password-reset tokens, etc.).
    pub fn verify_hash(&self, text: &str, expected_hash: &str, constant_time: bool) -> bool {
        let actual = self.hash(text);
        if constant_time {
            actual.as_bytes().ct_eq(expected_hash.as_bytes()).into()
        } else {
            actual == expected_hash
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use lightspeed_core::utils::new_hyphenated_uuid;

    #[test]
    fn should_hash_a_string() {
        let template = format!("Hello {}!!", new_hyphenated_uuid());
        let hash_service = LsHashService::new();

        let first_hash = hash_service.hash(&template);
        let second_hash = hash_service.hash(&template);

        println!("hash: {first_hash}");

        assert_ne!(template, second_hash);
        assert_eq!(first_hash, second_hash);

        for constant_time in [false, true] {
            assert!(hash_service.verify_hash(&template, &first_hash, constant_time));
            assert!(!hash_service.verify_hash(&template, &format!("{first_hash}1"), constant_time));
            assert!(!hash_service.verify_hash(&template, &template, constant_time));
        }
    }
}
