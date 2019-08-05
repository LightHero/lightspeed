use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct AuthConfig {

    /// Determines the token validity minutes
    #[structopt(long, default_value = "120")]
    pub token_activation_validity_minutes: i64,

    #[structopt(long, default_value = "/auth/token/activation/")]
    pub activation_token_ui_url: String,

    #[structopt(long, default_value = "/auth/token/resetPassword/")]
    pub reset_password_token_ui_url: String,

    #[structopt(long, default_value = "test@test.com")]
    pub auth_email_account_created_recipient: String,

    #[structopt(long, default_value = "10")]
    pub bcrypt_password_hash_cost: u32,

}

impl AuthConfig {
    pub fn build() -> Self {
        let app = Self::clap().setting(structopt::clap::AppSettings::AllowExternalSubcommands);
        Self::from_clap(&app.get_matches())
    }
}