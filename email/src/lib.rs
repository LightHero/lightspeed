use crate::config::EmailClientConfig;
use lightspeed_core::error::LightSpeedError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod model;
pub mod service;

#[derive(Clone)]
pub struct EmailClientModule {
    pub email_config: EmailClientConfig,
    pub email_client: Arc<Box<dyn service::email::EmailClient>>,
}

impl EmailClientModule {
    pub fn new(email_config: EmailClientConfig) -> Result<Self, LightSpeedError> {
        println!("Creating EmailClientModule");
        info!("Creating EmailClientModule");

        let email_service = Arc::new(service::email::new(email_config.clone())?);

        Ok(EmailClientModule { email_config, email_client: email_service })
    }
}

impl lightspeed_core::module::Module for EmailClientModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting EmailClientModule");
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

        let conf = LoggerConfig { level: String::from("trace"), stdout_output: true, stderr_output: false, file_output_path: None };
        setup_logger(&conf).unwrap();
    }
}
