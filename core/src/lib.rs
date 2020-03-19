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

impl module::Module for CoreModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CoreModule");
        Ok(())
    }
}

#[cfg(test)]
pub mod test_root {

    use lazy_static::lazy_static;
    use lightspeed_logger::config::LoggerConfig;
    use lightspeed_logger::setup_logger;
    use std::sync::Mutex;

    lazy_static! {
        static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
    }

    pub fn init_context() {
        let mut init = INITIALIZED.lock().unwrap();
        if !*init {
            println!("Initialize context");
            start_logger();
            *init = true;
        }
    }

    fn start_logger() {
        println!("Init logger");

        let conf = LoggerConfig {
            level: String::from("trace"),
            stdout_output: true,
            stderr_output: false,
            file_output_path: None,
        };
        setup_logger(&conf).unwrap();
    }
}
