use crate::config::CmsConfig;
use crate::repository::CmsRepositoryManager;
use lightspeed_core::{config::UIConfig, error::LightSpeedError};
use log::*;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct CmsModule<RepoManager: CmsRepositoryManager> {
    pub ui_config: UIConfig,
    pub cms_config: CmsConfig,

    pub repo_manager: RepoManager,
}

impl<RepoManager: CmsRepositoryManager> CmsModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, cms_config: CmsConfig, ui_config: UIConfig) -> Self {
        println!("Creating AuthModule");
        info!("Creating AuthModule");

        CmsModule {
            ui_config,
            cms_config,

            repo_manager,
        }
    }
}

impl<RepoManager: CmsRepositoryManager> lightspeed_core::module::Module for CmsModule<RepoManager> {
    fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CmsModule");
        self.repo_manager.start()?;
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
