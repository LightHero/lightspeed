use crate::repository::{CmsRepositoryManager};

#[derive(Clone)]
pub struct ProjectService<RepoManager: CmsRepositoryManager> {
    project_repo: RepoManager::PROJECT_REPO,
}

impl<RepoManager: CmsRepositoryManager> ProjectService<RepoManager> {
    pub fn new(project_repo: RepoManager::PROJECT_REPO) -> Self {
        ProjectService {
            project_repo,
        }
    }
}

#[cfg(test)]
pub mod test {

}
