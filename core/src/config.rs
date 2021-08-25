use clap::Clap;

/// Defines the JSON Web Token configuration.
#[derive(Debug, Clone, Clap)]
#[clap(rename_all = "kebab-case")]
#[clap(setting = clap::AppSettings::AllowExternalSubcommands)]
pub struct JwtConfig {
    /// The secret key used to encode and decode the JWT
    #[clap(long, env = "LS_CORE_JWS_SECRET", default_value = "")]
    pub secret: String,

    /// Determines the JWT signature algorithm.
    /*
    /// Valid values are:
    /// - HS256 -> HMAC using SHA-256
    /// - HS384 -> HMAC using SHA-384
    /// - HS512 -> HMAC using SHA-512
    /// - ES256 -> ECDSA using SHA-256
    /// - ES384 -> ECDSA using SHA-384
    /// - RS256 -> RSASSA-PKCS1-v1_5 using SHA-256,
    /// - RS384 -> RSASSA-PKCS1-v1_5 using SHA-384,
    /// - RS512 -> RSASSA-PKCS1-v1_5 using SHA-512,
     */
    #[clap(long, env = "LS_CORE_JWS_SIGNATURE_ALGORITHM", default_value = "HS512")]
    pub signature_algorithm: jsonwebtoken::Algorithm,

    /// Determines the token validity minutes
    #[clap(long, env = "LS_CORE_JWS_TOKEN_VALIDITY_MINUTES", default_value = "60")]
    pub token_validity_minutes: u32,
}

/// Defines the Logger configuration.
#[derive(Debug, Clone, Clap)]
#[clap(rename_all = "kebab-case")]
pub struct CoreConfig {
    #[clap(flatten)]
    pub jwt: JwtConfig,
}

impl CoreConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config = CoreConfig::build();
        assert!(config.jwt.token_validity_minutes > 0);
    }
}
