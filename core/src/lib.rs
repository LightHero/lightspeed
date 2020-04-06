pub mod config;
pub mod error;
pub mod model;
pub mod module;
pub mod service;
pub mod utils;

#[cfg(feature = "actix-web")]
pub mod web;

use crate::error::LightSpeedError;
use crate::service::auth::InMemoryRolesProvider;
use log::info;

#[derive(Clone)]
pub struct CoreModule {
    pub auth: service::auth::AuthService<InMemoryRolesProvider>,
    pub jwt: service::jwt::JwtService,
}

impl CoreModule {
    pub fn new(config: config::CoreConfig) -> Result<CoreModule, LightSpeedError> {
        println!("Creating CoreModule");
        info!("Creating CoreModule");

        let jwt = service::jwt::JwtService::new(&config.jwt)?;
        let auth = service::auth::AuthService::new(InMemoryRolesProvider::new(vec![]));
        Ok(CoreModule { jwt, auth })
    }
}

#[async_trait::async_trait(?Send)]
impl module::Module for CoreModule {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CoreModule");
        Ok(())
    }
}
