use crate::repository::{ CmsRepositoryManager};

#[derive(Clone)]
pub struct SchemaService<RepoManager: CmsRepositoryManager> {
    schema_repo: RepoManager::SCHEMA_REPO,
}

impl<RepoManager: CmsRepositoryManager> SchemaService<RepoManager> {
    pub fn new(schema_repo: RepoManager::SCHEMA_REPO) -> Self {
        SchemaService {
            schema_repo,
        }
    }

}

#[cfg(test)]
mod test {
    
}
