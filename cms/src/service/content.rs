use crate::dto::create_content_dto::CreateContentDto;
use crate::model::content::ContentModel;
use crate::model::schema::Schema;
use crate::repository::CmsRepositoryManager;
use crate::repository::ContentRepository;
use c3p0::*;
use chashmap::{CHashMap, ReadGuard};
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::Validator;

#[derive(Clone)]
pub struct ContentService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    repo_factory: RepoManager,
    content_repos: CHashMap<i64, RepoManager::CONTENT_REPO>,
}

impl<RepoManager: CmsRepositoryManager> ContentService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, repo_factory: RepoManager) -> Self {
        ContentService {
            c3p0,
            repo_factory,
            content_repos: CHashMap::new(),
        }
    }

    pub fn create_content_table(&self, schema_id: i64) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            repo.create_table(conn)
        })
    }

    pub fn drop_content_table(&self, schema_id: i64) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            repo.drop_table(conn)
        })
    }

    pub fn create_unique_index(
        &self,
        schema_id: i64,
        field_name: &str,
    ) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            let index_name = self.unique_index_name(schema_id, field_name);
            repo.create_unique_constraint(conn, &index_name, field_name)
        })
    }

    pub fn drop_unique_index(
        &self,
        schema_id: i64,
        field_name: &str,
    ) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            let index_name = self.unique_index_name(schema_id, field_name);
            repo.drop_unique_constraint(conn, &index_name)
        })
    }

    pub fn create_content(
        &self,
        schema: &Schema,
        create_content_dto: CreateContentDto,
    ) -> Result<ContentModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            Validator::validate(|error_details: &ErrorDetails| {
                create_content_dto.content.validate(&schema, error_details);
                Ok(())
            })?;
            let repo = self.get_content_repo_by_schema_id(create_content_dto.schema_id);
            repo.save(conn, NewModel::new(create_content_dto))
        })
    }

    pub fn delete_content(&self, content_model: ContentModel) -> Result<u64, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(content_model.data.schema_id);
            repo.delete(conn, &content_model)
        })
    }

    // ToDo: remove unwraps!! AND REWRITE...
    fn get_content_repo_by_schema_id(
        &self,
        schema_id: i64,
    ) -> ReadGuard<'_, i64, RepoManager::CONTENT_REPO> {
        let content_repo = self.content_repos.get(&schema_id);
        if content_repo.is_none() {
            {
                self.content_repos.insert(
                    schema_id,
                    self.repo_factory
                        .content_repo(&self.content_table_name(schema_id)),
                );
            }
            self.content_repos.get(&schema_id).unwrap()
        } else {
            content_repo.unwrap()
        }
    }

    fn content_table_name(&self, schema_id: i64) -> String {
        format!("CMS_CONTENT_{}", schema_id)
    }
    fn unique_index_name(&self, schema_id: i64, field_name: &str) -> String {
        format!("CMS_CONTENT_{}_UNIQUE_{}", schema_id, field_name)
    }
}

#[cfg(test)]
mod test {}
