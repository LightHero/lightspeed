use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng};
use argon2::{Algorithm, Argon2, Params, Version};
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct LsPasswordCodecService {
    argon2: Argon2<'static>,
    /// A real Argon2id hash produced with the same parameters as `argon2`,
    /// kept around so callers can perform a constant-time "verify" against
    /// it when the requested user does not exist. Computed at construction
    /// so verify timing matches the configured params.
    dummy_hash: String,
}

impl LsPasswordCodecService {
    /// Panics if the Argon2 parameters are invalid (per RFC 9106:
    /// `memory_kib >= 8 * parallelism`, `iterations >= 1`, `parallelism >= 1`).
    pub fn new(memory_kib: u32, iterations: u32, parallelism: u32) -> Self {
        let params = Params::new(memory_kib, iterations, parallelism, None)
            .expect("invalid argon2 parameters: require memory_kib >= 8 * parallelism, iterations >= 1, parallelism >= 1");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let salt = SaltString::generate(&mut OsRng);
        let dummy_hash = argon2
            .hash_password(b"lightspeed-dummy-password-for-timing-safety", &salt)
            .expect("failed to compute argon2 dummy hash")
            .to_string();

        LsPasswordCodecService { argon2, dummy_hash }
    }

    pub fn dummy_hash(&self) -> &str {
        &self.dummy_hash
    }

    pub async fn verify_match(&self, plain_password: &str, hashed: &str) -> Result<bool, LsError> {
        let plain = plain_password.to_owned();
        let hashed = hashed.to_owned();
        tokio::task::spawn_blocking(move || -> Result<bool, LsError> {
            let parsed = PasswordHash::new(&hashed)
                .map_err(|err| LsError::PasswordEncryptionError { message: format!("argon2 parse: {err:?}") })?;
            // verify_password uses the algorithm/params encoded in the hash,
            // so it works regardless of the verifier instance's settings.
            Ok(Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok())
        })
        .await
        .map_err(|err| LsError::PasswordEncryptionError { message: format!("argon2 task join error: {err:?}") })?
    }

    pub async fn hash_password(&self, plain_password: &str) -> Result<String, LsError> {
        let plain = plain_password.to_owned();
        let argon2 = self.argon2.clone();
        tokio::task::spawn_blocking(move || -> Result<String, LsError> {
            let salt = SaltString::generate(&mut OsRng);
            argon2
                .hash_password(plain.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|err| LsError::PasswordEncryptionError { message: format!("argon2 hash: {err:?}") })
        })
        .await
        .map_err(|err| LsError::PasswordEncryptionError { message: format!("argon2 task join error: {err:?}") })?
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    /// Spec-minimum parameters: `m=8` KiB, `t=1`, `p=1`. Used by tests so the
    /// Argon2 work factor doesn't dominate test time.
    fn fast_codec() -> LsPasswordCodecService {
        LsPasswordCodecService::new(8, 1, 1)
    }

    #[tokio::test]
    async fn should_encrypt_and_decrypt() -> Result<(), LsError> {
        let codec = fast_codec();
        let plain_pass = "wrwdsdfast346n534dfsg5353";
        let hash = codec.hash_password(plain_pass).await?;

        assert!(hash.starts_with("$argon2id$"));
        assert!(codec.verify_match(plain_pass, &hash).await?);

        let other_hash = codec.hash_password("asfasfasxcva").await?;
        assert!(!codec.verify_match(plain_pass, &other_hash).await?);

        Ok(())
    }

    #[tokio::test]
    async fn dummy_hash_should_verify_against_no_password() -> Result<(), LsError> {
        // The dummy hash is unguessable, so verifying any plaintext against it
        // must return false — but it must not error, since it's used in the
        // login timing-safety path.
        let codec = fast_codec();
        assert!(!codec.verify_match("any password", codec.dummy_hash()).await?);
        Ok(())
    }

    #[tokio::test]
    async fn each_hash_should_use_a_fresh_salt() -> Result<(), LsError> {
        let codec = fast_codec();
        let plain_pass = "same-password";
        let h1 = codec.hash_password(plain_pass).await?;
        let h2 = codec.hash_password(plain_pass).await?;
        assert_ne!(h1, h2);
        assert!(codec.verify_match(plain_pass, &h1).await?);
        assert!(codec.verify_match(plain_pass, &h2).await?);
        Ok(())
    }
}
