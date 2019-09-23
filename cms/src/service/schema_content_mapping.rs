use crate::repository::{CmsRepositoryManager};

#[derive(Clone)]
pub struct SchemaContentMappingService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    schema_content_repo: RepoManager::SCHEMA_CONTENT_MAPPING_REPO,
}

impl<RepoManager: CmsRepositoryManager> SchemaContentMappingService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, schema_content_repo: RepoManager::SCHEMA_CONTENT_MAPPING_REPO) -> Self {
        SchemaContentMappingService {
            c3p0,
            schema_content_repo,
        }
    }

}

#[cfg(test)]
mod test {

}
