use bcrypt::{hash, verify};
use ls_core::error::LightSpeedError;

#[derive(Clone)]
pub struct PasswordCodec {
    hash_cost: u32
}

impl PasswordCodec {

    /// Cost needs to be between 4 and 31
    /// Java bcrypt lib uses 10 by default
    pub fn new(hash_cost: u32) -> Self {
        PasswordCodec{
            hash_cost
        }
    }

    pub fn verify_match(&self, plain_password: &str, hash: &str) -> Result<bool, LightSpeedError> {
        verify(plain_password, hash).map_err(|err| LightSpeedError::PasswordEncryptionError {
            message: format!("{}", err),
        })
    }

    pub fn hash_password(&self, plain_password: &str) -> Result<String, LightSpeedError> {
        hash(plain_password, self.hash_cost).map_err(|err| LightSpeedError::PasswordEncryptionError {
            message: format!("{}", err),
        })
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_encrypt_and_decrypt() -> Result<(), LightSpeedError> {
        let password_codec = PasswordCodec::new(4);
        let plain_pass = "wrwdsdfast346n534dfsg5353";
        let hash = password_codec.hash_password(plain_pass)?;

        assert!(password_codec.verify_match(plain_pass, &hash)?);
        assert!(!password_codec.verify_match(plain_pass, &password_codec.hash_password("asfasfasxcva")?)?);

        Ok(())
    }

    #[test]
    fn should_decrypt_admin() -> Result<(), LightSpeedError> {
        let password_codec = PasswordCodec::new(4);
        let plain_pass = "admin";
        let hash = &password_codec.hash_password(plain_pass)?;
        let java_bcrypt_hash = r#"$2a$10$TkWSZIawgD9tjkmAV2GjGOt30FQktiTlpZTIHbxatakOHf4G0.aA."#;

        assert!(password_codec.verify_match(plain_pass, hash)?);
        assert!(password_codec.verify_match(plain_pass, java_bcrypt_hash)?);

        Ok(())
    }

}
