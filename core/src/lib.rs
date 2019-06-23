pub mod config;
pub mod error;
pub mod module;
pub mod service;

use log::info;
use crate::error::{LightSpeedError};

#[derive(Clone)]
pub struct CoreModule {
    pub jwt: service::jwt::JwtService,
}

impl CoreModule {
    pub fn new(config: config::CoreConfig) -> CoreModule {
        println!("Creating CoreModule with configuration:\n{:#?}", config);
        info!("Creating CoreModule with configuration:\n{:#?}", config);

        let jwt = service::jwt::JwtService::new(&config.jwt);

        CoreModule {
            jwt,
        }
    }
}

impl module::Module for CoreModule {

    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Core start");
        Ok(())
    }
}
