use crate::repository::{CmsRepositoryManager};

#[derive(Clone)]
pub struct SchemaContentMappingService<RepoManager: CmsRepositoryManager> {
    schema_content_repo: RepoManager::SCHEMA_CONTENT_MAPPING_REPO,
}

impl<RepoManager: CmsRepositoryManager> SchemaContentMappingService<RepoManager> {
    pub fn new(schema_content_repo: RepoManager::SCHEMA_CONTENT_MAPPING_REPO) -> Self {
        SchemaContentMappingService {
            schema_content_repo,
        }
    }

}

#[cfg(test)]
mod test {

}
