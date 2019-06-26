use structopt::StructOpt;

/// Defines the JSON Web Token configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct JwtConfig {
    /// The secret key used to encode and decode the JWT
    #[structopt(long)]
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
    #[structopt(long, default_value = "HS256")]
    pub signature_algorithm: jsonwebtoken::Algorithm,

    /// Determines the token validity minutes
    #[structopt(long, default_value = "60")]
    pub token_validity_minutes: u32,
}

/// Defines the Logger configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CoreConfig {

    #[structopt(flatten)]
    pub jwt: JwtConfig,
}

impl CoreConfig {
    pub fn build() -> Self {
        CoreConfig::from_args()
    }
}

#[cfg(test)]
mod test {}
