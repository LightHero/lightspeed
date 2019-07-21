use c3p0::C3p0Builder;
use log::*;
use ls_core::error::LightSpeedError;

pub mod config;
pub mod service;

#[derive(Clone)]
pub struct AuthModule {
    pub db: service::db::AuthDbService
}

impl AuthModule {
    pub fn new(c3p0: C3p0Builder) -> Self {
        println!("Creating AuthModule");
        info!("Creating AuthModule");

        let db = service::db::AuthDbService::new(c3p0);

        AuthModule { db }
    }
}

impl ls_core::module::Module for AuthModule {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting AuthModule");
        self.db.start()?;
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
