use crate::config::CmsConfig;
use crate::repository::CmsRepositoryManager;
use crate::service::project::ProjectService;
use crate::service::schema::SchemaService;
use lightspeed_core::error::LightSpeedError;
use log::*;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct CmsModule<RepoManager: CmsRepositoryManager> {
    pub cms_config: CmsConfig,

    pub repo_manager: RepoManager,

    pub project_service: ProjectService<RepoManager>,
    pub schema_service: SchemaService<RepoManager>,
}

impl<RepoManager: CmsRepositoryManager> CmsModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, cms_config: CmsConfig) -> Self {
        println!("Creating CmsModule");
        info!("Creating CmsModule");

        let schema_service =
            SchemaService::new(repo_manager.c3p0().clone(), repo_manager.schema_repo());
        let project_service = ProjectService::new(
            repo_manager.c3p0().clone(),
            repo_manager.project_repo(),
            schema_service.clone(),
        );

        CmsModule {
            cms_config,

            repo_manager,

            project_service,
            schema_service,
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
