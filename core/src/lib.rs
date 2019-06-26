pub mod config;
pub mod error;
pub mod module;
pub mod service;

use crate::error::LightSpeedError;
use log::info;

#[derive(Clone)]
pub struct CoreModule {
    pub jwt: service::jwt::JwtService,
}

impl CoreModule {
    pub fn new(config: config::CoreConfig) -> CoreModule {
        println!("Creating CoreModule with configuration:\n{:#?}", config);
        info!("Creating CoreModule with configuration:\n{:#?}", config);

        let jwt = service::jwt::JwtService::new(&config.jwt);

        CoreModule { jwt }
    }
}

impl module::Module for CoreModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CoreModule");
        Ok(())
    }
}
