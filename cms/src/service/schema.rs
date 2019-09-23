use crate::repository::{ CmsRepositoryManager};

#[derive(Clone)]
pub struct SchemaService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    schema_repo: RepoManager::SCHEMA_REPO,
}

impl<RepoManager: CmsRepositoryManager> SchemaService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, schema_repo: RepoManager::SCHEMA_REPO) -> Self {
        SchemaService {
            c3p0,
            schema_repo,
        }
    }

}

#[cfg(test)]
mod test {
    
}
