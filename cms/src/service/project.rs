use crate::dto::create_project_dto::CreateProjectDto;
use crate::model::project::{ProjectData, ProjectModel};
use crate::repository::CmsRepositoryManager;
use crate::repository::ProjectRepository;
use crate::service::schema::LsSchemaService;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LsError};
use lightspeed_core::service::validator::{ERR_NOT_UNIQUE, Validator};
use std::sync::Arc;

#[derive(Clone)]
pub struct LsProjectService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    project_repo: RepoManager::ProjectRepo,
    schema_service: Arc<LsSchemaService<RepoManager>>,
}

impl<RepoManager: CmsRepositoryManager> LsProjectService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        project_repo: RepoManager::ProjectRepo,
        schema_service: Arc<LsSchemaService<RepoManager>>,
    ) -> Self {
        LsProjectService { c3p0, project_repo, schema_service }
    }

    pub async fn create_project(&self, create_project_dto: CreateProjectDto) -> Result<ProjectModel, LsError> {
        self.c3p0
            .transaction(async |conn| {
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

    pub async fn delete(&self, project_model: ProjectModel) -> Result<ProjectModel, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.schema_service.delete_by_project_id(conn, project_model.id).await?;
                self.project_repo.delete(conn, project_model).await
            })
            .await
    }
}

#[cfg(test)]
pub mod test {}
