use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AMConfig {
    /// Determines the activation token validity minutes
    pub activation_token_validity_minutes: u32,

    /// Determines the maximum session validity minutes.
    /// Once the session expires it is not possible to refresh it
    /// and the user needs to reenter his credentials.
    pub auth_session_max_validity_minutes: u32,

    /// Argon2id memory cost, in KiB. Must be >= 8 * `argon2_parallelism`.
    pub argon2_memory_kib: u32,
    /// Argon2id time cost (number of iterations). Must be >= 1.
    pub argon2_iterations: u32,
    /// Argon2id parallelism factor. Must be >= 1.
    pub argon2_parallelism: u32,

    pub default_roles_on_account_creation: Vec<String>,

    /// Maximum age, in seconds, of a user's password before login is rejected
    /// and the user is required to set a new password. `None` disables the
    /// check.
    pub password_expiration_seconds: Option<u32>,
}

impl Default for AMConfig {
    fn default() -> Self {
        Self {
            activation_token_validity_minutes: 120,
            auth_session_max_validity_minutes: 240,
            // OWASP-recommended Argon2id settings (m=19 MiB, t=2, p=1).
            argon2_memory_kib: 19_456,
            argon2_iterations: 2,
            argon2_parallelism: 1,
            default_roles_on_account_creation: vec![],
            password_expiration_seconds: None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config: AMConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
        assert!(config.default_roles_on_account_creation.is_empty());
    }
}
