use jsonwebtoken::Algorithm;
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JwtConfig {
    /// The secret key used to encode and decode the JWT.
    ///
    /// Wrapped in `SecretString` so the bytes are zeroed on drop and don't
    /// leak into log/`Debug` output. Currently only HMAC-family algorithms
    /// (HS256/HS384/HS512) are supported by `LsJwtService`; asymmetric algorithms
    /// (RS*, ES*, PS*, EdDSA) need PEM/DER-encoded keys via different
    /// `EncodingKey` / `DecodingKey` constructors and are rejected at
    /// `LsJwtService::new`.
    pub secret: SecretString,

    /// Determines the JWT signature algorithm. Must be one of HS256, HS384,
    /// HS512 — see `LsJwtService::new`.
    pub signature_algorithm: jsonwebtoken::Algorithm,

    /// Determines the token validity minutes
    pub token_validity_minutes: u32,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self { secret: SecretString::default(), signature_algorithm: Algorithm::HS512, token_validity_minutes: 60 }
    }
}

/// Defines the Logger configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CoreConfig {
    #[serde(default)]
    pub jwt: JwtConfig,
}

#[cfg(test)]
mod test {

    use super::*;
    use config::Config;

    #[test]
    fn should_build_config() {
        let config: CoreConfig = Config::builder().build().unwrap().try_deserialize().unwrap();
        assert!(config.jwt.token_validity_minutes > 0);
    }
}
