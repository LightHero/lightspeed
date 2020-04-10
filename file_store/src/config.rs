use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct FileStoreConfig {
    /*
    /// Determines the activation token validity minutes
    #[structopt(long, env = "LS_AUTH_TOKEN_VALIDITY_MINUTES", default_value = "120")]
    pub token_validity_minutes: i64,

    #[structopt(long, env = "LS_AUTH_PASSWORD_HASH_COST", default_value = "10")]
    pub bcrypt_password_hash_cost: u32,

    #[structopt(long, env = "LS_AUTH_DEFAULT_ROLES_ON_ACCOUNT_CREATION")]
    pub default_roles_on_account_creation: Vec<String>,
    */
}

impl FileStoreConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_config() {
        FileStoreConfig::build();
    }
}
