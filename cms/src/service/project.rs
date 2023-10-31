use crate::dto::create_project_dto::CreateProjectDto;
use crate::model::project::{ProjectData, ProjectModel};
use crate::repository::CmsRepositoryManager;
use crate::repository::ProjectRepository;
use crate::service::schema::SchemaService;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    project_repo: RepoManager::ProjectRepo,
    schema_service: Arc<SchemaService<RepoManager>>,
}

impl<RepoManager: CmsRepositoryManager> ProjectService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        project_repo: RepoManager::ProjectRepo,
        schema_service: Arc<SchemaService<RepoManager>>,
    ) -> Self {
        ProjectService { c3p0, project_repo, schema_service }
    }

    pub async fn create_project(&self, create_project_dto: CreateProjectDto) -> Result<ProjectModel, LightSpeedError> {
        self.c3p0
            .transaction(|conn| async {
                let name_already_exists = self.project_repo.exists_by_name(conn, &create_project_dto.name).await?;

                let data = ProjectData { name: create_project_dto.name };
                Validator::validate(&(&data, &|error_details: &mut ErrorDetails| {
                    if name_already_exists {
                        error_details.add_detail("name", ERR_NOT_UNIQUE);
                    }
                    Ok(())
                }))?;
                self.project_repo.save(conn, NewModel::new(data)).await
            })
            .await
    }

    pub async fn delete(&self, project_model: ProjectModel) -> Result<ProjectModel, LightSpeedError> {
        self.c3p0
            .transaction(|conn| async {
                self.schema_service.delete_by_project_id(conn, project_model.id).await?;
                self.project_repo.delete(conn, project_model).await
            })
            .await
    }
}

#[cfg(test)]
pub mod test {}
