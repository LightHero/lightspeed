use crate::repository::{CmsRepositoryManager};
use lightspeed_core::error::{LightSpeedError, ErrorDetails};
use crate::model::project::{ProjectModel, ProjectData};
use crate::dto::create_project_dto::CreateProjectDto;
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use crate::repository::ProjectRepository;
use c3p0::*;

#[derive(Clone)]
pub struct ProjectService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    project_repo: RepoManager::PROJECT_REPO,
}

impl<RepoManager: CmsRepositoryManager> ProjectService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, project_repo: RepoManager::PROJECT_REPO) -> Self {
        ProjectService {
            c3p0,
            project_repo,
        }
    }

    pub fn create_project(&self, create_project_dto: CreateProjectDto) -> Result<ProjectModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {

            let name_already_exists = self.project_repo.exists_by_name(conn, &create_project_dto.name)?;

            let data = ProjectData{
                 name: create_project_dto.name
            };
            Validator::validate((&data, |error_details: &ErrorDetails| {
                if name_already_exists {
                    error_details.add_detail("name", ERR_NOT_UNIQUE);
                }
                Ok(())
            }))?;
            self.project_repo.save(conn, NewModel::new(data))
        })
    }

    pub fn delete(
        &self,
        project_model: ProjectModel,
    ) -> Result<u64, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            self.project_repo.delete(conn, &project_model)
        })
    }
}

#[cfg(test)]
pub mod test {

}
