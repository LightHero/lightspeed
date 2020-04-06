use crate::config::CmsConfig;
use crate::repository::CmsRepositoryManager;
use crate::service::content::ContentService;
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

    pub content_service: ContentService<RepoManager>,
    pub project_service: ProjectService<RepoManager>,
    pub schema_service: SchemaService<RepoManager>,
}

impl<RepoManager: CmsRepositoryManager> CmsModule<RepoManager> {
    pub fn new(repo_manager: RepoManager, cms_config: CmsConfig) -> Self {
        println!("Creating CmsModule");
        info!("Creating CmsModule");

        let content_service =
            ContentService::new(repo_manager.c3p0().clone(), repo_manager.clone());

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

            content_service,
            project_service,
            schema_service,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<RepoManager: CmsRepositoryManager> lightspeed_core::module::Module for CmsModule<RepoManager> {
    async fn start(&mut self) -> Result<(), LightSpeedError> {
        info!("Starting CmsModule");
        self.repo_manager.start().await?;
        Ok(())
    }
}
