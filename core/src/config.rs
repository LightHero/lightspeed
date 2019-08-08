use structopt::StructOpt;

/// Global Web configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct UIConfig {
    /// The public site URL
    #[structopt(long, default_value = "http://127.0.0.1")]
    pub public_domain: String,
}

impl UIConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}

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
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}

#[cfg(test)]
mod test {}
