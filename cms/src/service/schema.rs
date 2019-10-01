use crate::dto::create_schema_dto::CreateSchemaDto;
use crate::model::schema::{SchemaData, SchemaModel};
use crate::repository::CmsRepositoryManager;
use crate::repository::SchemaRepository;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};

#[derive(Clone)]
pub struct SchemaService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    schema_repo: RepoManager::SCHEMA_REPO,
}

impl<RepoManager: CmsRepositoryManager> SchemaService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, schema_repo: RepoManager::SCHEMA_REPO) -> Self {
        SchemaService { c3p0, schema_repo }
    }

    pub fn create_schema(
        &self,
        create_schema_dto: CreateSchemaDto,
    ) -> Result<SchemaModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let name_already_exists = self.schema_repo.exists_by_name_and_project_id(
                conn,
                &create_schema_dto.name,
                create_schema_dto.project_id,
            )?;

            let data = SchemaData {
                name: create_schema_dto.name,
                project_id: create_schema_dto.project_id,
                schema: create_schema_dto.schema,
            };
            Validator::validate((&data, |error_details: &ErrorDetails| {
                if name_already_exists {
                    error_details.add_detail("name", ERR_NOT_UNIQUE);
                }
                Ok(())
            }))?;
            self.schema_repo.save(conn, NewModel::new(data))
        })
    }

    pub fn delete(&self, schema_model: SchemaModel) -> Result<u64, LightSpeedError> {
        self.c3p0
            .transaction(move |conn| self.schema_repo.delete(conn, &schema_model))
    }

    pub fn delete_by_project_id(
        &self,
        conn: &RepoManager::CONN,
        project_id: i64,
    ) -> Result<u64, LightSpeedError> {
        self.schema_repo.delete_by_project_id(conn, project_id)
    }
}

#[cfg(test)]
mod test {}
