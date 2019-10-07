use crate::repository::CmsRepositoryManager;
use crate::repository::SchemaRepository;
use c3p0::*;
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use chashmap::{CHashMap, ReadGuard};
use crate::dto::create_content_dto::CreateContentDto;
use crate::model::content::ContentModel;
use evmap::{ReadHandle, WriteHandle};

#[derive(Clone)]
pub struct ContentService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    repo_factory: RepoManager,
    content_repos: CHashMap<i64, RepoManager::CONTENT_REPO>,
}

impl<RepoManager: CmsRepositoryManager> ContentService<RepoManager> {

    pub fn new(c3p0: RepoManager::C3P0, repo_factory: RepoManager) -> Self {
        ContentService { c3p0, repo_factory, content_repos: CHashMap::new() }
    }

    pub fn create_content_table(
        &self,
        schema_id: i64,
    ) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            unimplemented!()
        })
    }

    pub fn drop_content_table(
        &self,
        conn: &RepoManager::CONN,
        project_id: i64,
    ) -> Result<(), LightSpeedError> {
        unimplemented!()
    }

    pub fn create_content(
        &self,
        create_content_dto: CreateContentDto,
    ) -> Result<ContentModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            unimplemented!()
        })
    }

    pub fn delete_content(&self, content_model: ContentModel) -> Result<u64, LightSpeedError> {
        self.c3p0
            .transaction(move |conn|
            unimplemented!()
            )
    }

    // ToDo: remove unwraps!! AND REWRITE...
    fn get_content_repo_by_schema_id(&self,  schema_id: &i64) -> ReadGuard<'_, i64, RepoManager::CONTENT_REPO> {
        let content_repo = self.content_repos.get(schema_id);
        if content_repo.is_none() {
            {
                self.content_repos.insert(*schema_id, self.repo_factory.content_repo(&self.content_table_name(schema_id)));
            }
            self.content_repos.get(schema_id).unwrap()
        } else {
            content_repo.unwrap()
        }
    }

    fn content_table_name(&self, schema_id: &i64) -> String {
        format!("CMS_CONTENT_{}", schema_id)
    }

}

#[cfg(test)]
mod test {}
