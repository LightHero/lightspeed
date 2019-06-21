pub mod config;

use log::info;
use coreutils_module::ModuleError;

#[derive(Clone)]
pub struct CoreModule {
    pub json: coreutils_json::JsonService,
    pub jwt: coreutils_jwt::JwtService,
}

impl CoreModule {
    pub fn new(config: config::CoreConfig) -> CoreModule {
        println!("Creating CoreModule with configuration:\n{:#?}", config);
        info!("Creating CoreModule with configuration:\n{:#?}", config);

        let jwt = coreutils_jwt::JwtService::new(&config.jwt);

        CoreModule {
            json: coreutils_json::new(),
            jwt,
        }
    }
}

impl coreutils_module::Module for CoreModule {

    fn start(&mut self) -> Result<(), ModuleError> {
        info!("Core start");
        Ok(())
    }
}
