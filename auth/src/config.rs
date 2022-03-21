use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Determines the activation token validity minutes
    pub activation_token_validity_minutes: i64,

    /// Determines the maximum session validity minutes.
    /// Once the session expires it is not possible to refresh it
    /// and the user needs to reenter his credentials.
    pub auth_session_max_validity_minutes: i64,
    pub bcrypt_password_hash_cost: u32,
    pub default_roles_on_account_creation: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            activation_token_validity_minutes: 120,
            auth_session_max_validity_minutes: 240,
            bcrypt_password_hash_cost: 10,
            default_roles_on_account_creation: vec![],
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config: AuthConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
        assert!(config.default_roles_on_account_creation.is_empty());
    }
}
