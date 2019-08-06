use c3p0::*;
use log::*;
use lightspeed_core::{error::LightSpeedError};
use crate::config::{EmailConfig};
use std::sync::Arc;

pub mod config;
pub mod model;
pub mod service;

pub type PoolManager = PgPoolManager;

#[derive(Clone)]
pub struct EmailModule {
    pub email_config: EmailConfig,

    pub c3p0: C3p0Pool<PoolManager>,

    pub email_service: Arc<Box<service::email::EmailService>>,
}

impl EmailModule {
    pub fn new(email_config: EmailConfig, c3p0: C3p0Pool<PoolManager>) -> Result<Self, LightSpeedError> {
        println!("Creating EmailModule");
        info!("Creating EmailModule");

        let email_service = Arc::new(service::email::new(email_config.clone()));

        Ok(EmailModule {
            email_config,

            c3p0,

            email_service,
        })
    }
}

impl lightspeed_core::module::Module for EmailModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting EmailModule");
        Ok(())
    }
}

#[cfg(test)]
pub mod test_root {

    use lazy_static::lazy_static;
    use ls_logger::config::LoggerConfig;
    use ls_logger::setup_logger;
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
