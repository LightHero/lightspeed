use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OutboxConfig {}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        let config: OutboxConfig = config::Config::builder().build().unwrap().try_deserialize().unwrap();
        // assert!(config.default_roles_on_account_creation.is_empty());
    }
}
