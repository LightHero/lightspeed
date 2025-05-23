use crate::config::CmsConfig;
use crate::repository::CmsRepositoryManager;
use crate::service::content::LsContentService;
use crate::service::project::LsProjectService;
use crate::service::schema::LsSchemaService;
use lightspeed_core::error::LsError;
use log::*;
use std::sync::Arc;

pub mod config;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct LsCmsModule<RepoManager: CmsRepositoryManager> {
    pub cms_config: CmsConfig,

    pub repo_manager: RepoManager,

    pub content_service: Arc<LsContentService<RepoManager>>,
    pub project_service: Arc<LsProjectService<RepoManager>>,
    pub schema_service: Arc<LsSchemaService<RepoManager>>,
}

impl<RepoManager: CmsRepositoryManager> LsCmsModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, cms_config: CmsConfig) -> Self {
        println!("Creating LsCmsModule");
        info!("Creating LsCmsModule");

        let content_service = Arc::new(LsContentService::new(repo_manager.c3p0().clone(), repo_manager.clone()));

        let schema_service = Arc::new(LsSchemaService::new(repo_manager.c3p0().clone(), repo_manager.schema_repo()));

        let project_service = Arc::new(LsProjectService::new(
            repo_manager.c3p0().clone(),
            repo_manager.project_repo(),
            schema_service.clone(),
        ));

        LsCmsModule { cms_config, repo_manager, content_service, project_service, schema_service }
    }
}

impl<RepoManager: CmsRepositoryManager> lightspeed_core::module::LsModule for LsCmsModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LsError> {
        info!("Starting LsCmsModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
