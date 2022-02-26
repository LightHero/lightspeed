use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct AuthConfig {
    /// Determines the activation token validity minutes
    #[clap(long, env = "LS_AUTH_ACTIVATION_TOKEN_VALIDITY_MINUTES", default_value = "120")]
    pub activation_token_validity_minutes: i64,

    /// Determines the maximum session validity minutes.
    /// Once the session expires it is not possible to refresh it
    /// and the user needs to reenter his credentials.
    #[clap(long, env = "LS_AUTH_SESSION_MAX_VALIDITY_MINUTES", default_value = "240")]
    pub auth_session_max_validity_minutes: i64,

    #[clap(long, env = "LS_AUTH_PASSWORD_HASH_COST", default_value = "10")]
    pub bcrypt_password_hash_cost: u32,

    #[clap(long, env = "LS_AUTH_DEFAULT_ROLES_ON_ACCOUNT_CREATION")]
    pub default_roles_on_account_creation: Vec<String>,
}

impl AuthConfig {
    pub fn build() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config = AuthConfig::build();
        assert!(config.default_roles_on_account_creation.is_empty());
    }
}
