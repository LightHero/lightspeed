use base64::{engine::general_purpose, Engine as _};
use sha2::Digest;

#[derive(Clone, Default)]
pub struct LsHashService {}

impl LsHashService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash(&self, text: &str) -> String {
        let mut hasher = sha2::Sha256::default();
        hasher.update(text);
        let result = hasher.finalize();
        general_purpose::STANDARD_NO_PAD.encode(result)
    }

    pub fn verify_hash(&self, text: &str, expected_hash: &str) -> bool {
        self.hash(text).eq(expected_hash)
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

        assert!(hash_service.verify_hash(&template, &first_hash));
        assert!(!hash_service.verify_hash(&template, &format!("{first_hash}1")));
        assert!(!hash_service.verify_hash(&template, &template));
    }
}
