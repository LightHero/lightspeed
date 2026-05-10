use bcrypt::{hash, verify};
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct LsPasswordCodecService {
    hash_cost: u32,
}

impl LsPasswordCodecService {
    /// Cost needs to be between 4 and 31
    /// Java bcrypt lib uses 10 by default
    pub fn new(hash_cost: u32) -> Self {
        LsPasswordCodecService { hash_cost }
    }

    pub async fn verify_match(&self, plain_password: &str, hashed: &str) -> Result<bool, LsError> {
        let plain = plain_password.to_owned();
        let hashed = hashed.to_owned();
        tokio::task::spawn_blocking(move || {
            verify(&plain, &hashed).map_err(|err| LsError::PasswordEncryptionError { message: format!("{err:?}") })
        })
        .await
        .map_err(|err| LsError::PasswordEncryptionError { message: format!("bcrypt task join error: {err:?}") })?
    }

    pub async fn hash_password(&self, plain_password: &str) -> Result<String, LsError> {
        let plain = plain_password.to_owned();
        let cost = self.hash_cost;
        tokio::task::spawn_blocking(move || {
            hash(&plain, cost).map_err(|err| LsError::PasswordEncryptionError { message: format!("{err:?}") })
        })
        .await
        .map_err(|err| LsError::PasswordEncryptionError { message: format!("bcrypt task join error: {err:?}") })?
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[tokio::test]
    async fn should_encrypt_and_decrypt() -> Result<(), LsError> {
        let password_codec = LsPasswordCodecService::new(4);
        let plain_pass = "wrwdsdfast346n534dfsg5353";
        let hash = password_codec.hash_password(plain_pass).await?;

        assert!(password_codec.verify_match(plain_pass, &hash).await?);
        let other_hash = password_codec.hash_password("asfasfasxcva").await?;
        assert!(!password_codec.verify_match(plain_pass, &other_hash).await?);

        Ok(())
    }

    #[tokio::test]
    async fn should_decrypt_admin() -> Result<(), LsError> {
        let password_codec = LsPasswordCodecService::new(4);
        let plain_pass = "admin";
        let hash = &password_codec.hash_password(plain_pass).await?;
        let java_bcrypt_hash = r#"$2a$10$TkWSZIawgD9tjkmAV2GjGOt30FQktiTlpZTIHbxatakOHf4G0.aA."#;

        assert!(password_codec.verify_match(plain_pass, hash).await?);
        assert!(password_codec.verify_match(plain_pass, java_bcrypt_hash).await?);

        Ok(())
    }
}
