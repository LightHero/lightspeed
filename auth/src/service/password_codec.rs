use ls_core::error::LightSpeedError;
use bcrypt::{hash, verify};

pub fn verify_match(plain_password: &str, hash: &str) -> Result<bool, LightSpeedError> {
    verify(plain_password, hash).map_err(|err| LightSpeedError::PasswordEncryptionError{message: format!("{}", err)})
}

pub fn hash_password(plain_password: &str) -> Result<String, LightSpeedError> {
    // Java bcrypt lib uses 10 by default
    hash(plain_password, 7).map_err(|err| LightSpeedError::PasswordEncryptionError{message: format!("{}", err)})
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_encrypt_and_decrypt() -> Result<(), LightSpeedError> {

        let plain_pass = "wrwdsdfast346n534dfsg5353";
        let hash = hash_password(plain_pass)?;

        assert!(verify_match(plain_pass, &hash)?);
        assert!(!verify_match(plain_pass, &hash_password("asfasfasxcva")?)?);

        Ok(())
    }

    #[test]
    fn should_decrypt_admin() -> Result<(), LightSpeedError> {

        let plain_pass = "admin";
        let hash = &hash_password(plain_pass)?;
        let java_bcrypt_hash = r#"$2a$10$TkWSZIawgD9tjkmAV2GjGOt30FQktiTlpZTIHbxatakOHf4G0.aA."#;

        assert!(verify_match(plain_pass, hash)?);
        assert!(verify_match(plain_pass, java_bcrypt_hash)?);

        Ok(())
    }

}