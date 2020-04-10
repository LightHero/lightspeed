use c3p0::*;
use log::*;
use crate::repository::FileStoreRepositoryManager;

#[derive(Clone)]
pub struct FileStoreService<RepoManager: FileStoreRepositoryManager> {
    c3p0: RepoManager::C3P0,
    file_store_repo: RepoManager::FileStoreRepo,
}

impl<RepoManager: FileStoreRepositoryManager> FileStoreService<RepoManager> {
    pub fn new(
        c3p0: RepoManager::C3P0,
        file_store_repo: RepoManager::FileStoreRepo,
    ) -> Self {
        FileStoreService {
            c3p0,
            file_store_repo,
        }
    }
}
