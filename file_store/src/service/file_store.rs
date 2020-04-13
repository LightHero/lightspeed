use crate::repository::db::{DBFileStoreRepositoryManager, DBFileStoreBinaryRepository};
use c3p0::*;
use log::*;
use crate::repository::FileStoreRepoManager;
use crate::dto::FileData;
use lightspeed_core::error::LightSpeedError;


#[derive(Clone)]
pub struct FileStoreService<RepoManager: DBFileStoreRepositoryManager> {
    repo_manager: FileStoreRepoManager<RepoManager>,
}

impl<RepoManager: DBFileStoreRepositoryManager> FileStoreService<RepoManager> {

    pub fn new(
        repo_manager: FileStoreRepoManager<RepoManager>,
    ) -> Self {
        FileStoreService {
            repo_manager,
        }
    }

    pub async fn read_file(
        &self,
        file_name: &str,
    ) -> Result<FileData, LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.read_file(file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.c3p0().transaction(|mut conn| async move {
                    repo_manager.file_store_binary_repo().read_file(&mut conn, file_name).await
                }).await
            }
        }
    }

    pub async fn read_file_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        file_name: &str,
    ) -> Result<FileData, LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.read_file(file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.file_store_binary_repo().read_file(conn, file_name).await
            }
        }
    }

    pub async fn save_file(
        &self,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.save_file(source_path, file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.c3p0().transaction(|mut conn| async move {
                    repo_manager.file_store_binary_repo().save_file(&mut conn, source_path, file_name).await
                }).await
            }
        }
    }

    pub async fn save_file_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.save_file(source_path, file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.file_store_binary_repo().save_file(conn, source_path, file_name).await
            }
        }
    }

    pub async fn delete_by_filename(
        &self,
        file_name: &str,
    ) -> Result<u64, LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.delete_by_filename(file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.c3p0().transaction(|mut conn| async move {
                    repo_manager.file_store_binary_repo().delete_by_filename(&mut conn, file_name).await
                }).await
            }
        }
    }

    pub async fn delete_by_filename_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        file_name: &str,
    ) -> Result<u64, LightSpeedError> {
        match &self.repo_manager {
            FileStoreRepoManager::FS(repo) => {
                repo.delete_by_filename(file_name).await
            },
            FileStoreRepoManager::DB(repo_manager) => {
                repo_manager.file_store_binary_repo().delete_by_filename(conn, file_name).await

            }
        }
    }

}
