use crate::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager, FileStoreDataRepository};
use c3p0::*;
use lightspeed_core::error::{LightSpeedError, ErrorCodes};
use log::*;
use std::collections::HashMap;
use crate::repository::filesystem::fs_file_store_binary::FsFileStoreBinaryRepository;
use crate::model::{FileStoreDataModel, BinaryContent, Repository};

#[derive(Clone)]
pub struct FileStoreService<RepoManager: DBFileStoreRepositoryManager> {
    c3p0: RepoManager::C3P0,
    db_binary_repo: RepoManager::FileStoreBinaryRepo,
    db_data_repo: RepoManager::FileStoreDataRepo,
    fs_repositories: HashMap<String, FsFileStoreBinaryRepository>
}

impl<RepoManager: DBFileStoreRepositoryManager> FileStoreService<RepoManager> {
    pub fn new(repo_manager: &RepoManager, fs_repositories: HashMap<String, String>) -> Self {
        FileStoreService {
            c3p0: repo_manager.c3p0().clone(),
            db_binary_repo: repo_manager.file_store_binary_repo(),
            db_data_repo: repo_manager.file_store_data_repo(),
            fs_repositories: fs_repositories.into_iter().map(|(name, base_path)| (name, FsFileStoreBinaryRepository::new(base_path)) ).collect()
        }
    }

    pub async fn read_file_data_by_id(&self, id: IdType) -> Result<FileStoreDataModel, LightSpeedError> {
        self.c3p0.transaction(|mut conn| async move {
            self.read_file_data_by_id_with_conn(&mut conn,id).await
        }).await
    }

    pub async fn read_file_data_by_id_with_conn(&self, conn: &mut RepoManager::Conn, id: IdType) -> Result<FileStoreDataModel, LightSpeedError> {
        debug!("FileStoreService - Read file by id [{}]", id);
        self.db_data_repo.fetch_one_by_id(conn, id).await
    }

    pub async fn read_file_content(&self, repository: &Repository) -> Result<BinaryContent, LightSpeedError> {
        debug!("FileStoreService - Read file [{:?}]", repository);
        match repository {
            Repository::DB {file_id} => {
                self.c3p0.transaction(|mut conn| async move {
                    self.db_binary_repo.read_file(&mut conn, *file_id).await
                }).await
            },
            Repository::FS {relative_path, repository_name} => {
                self.read_file_content_from_fs(&relative_path, &repository_name).await
            }
        }
    }

    #[inline]
    async fn read_file_content_from_fs(&self, filename: &str, repository_name: &str) -> Result<BinaryContent, LightSpeedError> {
        let repo = self.fs_repositories.get(repository_name).ok_or_else(|| LightSpeedError::BadRequest {
            message: format!("FileStoreService - Cannot find FS repository with name [{}]", repository_name),
            code: ErrorCodes::NOT_FOUND
        })?;
        repo.read_file(filename).await
    }

    pub async fn read_file_content_with_conn(&self, conn: &mut RepoManager::Conn, repository: &Repository) -> Result<BinaryContent, LightSpeedError> {
        debug!("FileStoreService - Read file [{:?}]", repository);
        match repository {
            Repository::DB {file_id} => {
                self.db_binary_repo.read_file(conn, *file_id).await
            },
            Repository::FS {relative_path, repository_name} => {
                self.read_file_content_from_fs(relative_path, repository_name).await
            }
        }
    }



    /*
    pub async fn read_file_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        file_name: &str,
    ) -> Result<FileData, LightSpeedError> {
        debug!("FileStoreService - Read file [{}]", file_name);
        match &self.repo_manager {
            FileStoreRepoManager::FS{fs} => fs.read_file(file_name).await,
            FileStoreRepoManager::DB{db} => {
                db
                    .file_store_binary_repo()
                    .read_file(conn, file_name)
                    .await
            }
        }
    }

    pub async fn save_file(
        &self,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError> {
        info!(
            "FileStoreService - Save file from [{}] to [{}]",
            source_path, file_name
        );
        match &self.repo_manager {
            FileStoreRepoManager::FS{fs} => fs.save_file(source_path, file_name).await,
            FileStoreRepoManager::DB{db} => {
                db
                    .c3p0()
                    .transaction(|mut conn| async move {
                        repo_manager
                            .file_store_binary_repo()
                            .save_file(&mut conn, source_path, file_name)
                            .await
                    })
                    .await
            }
        }
    }

    pub async fn save_file_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError> {
        info!(
            "FileStoreService - Save file from [{}] to [{}]",
            source_path, file_name
        );
        match &self.repo_manager {
            FileStoreRepoManager::FS{fs} => fs.save_file(source_path, file_name).await,
            FileStoreRepoManager::DB{db} => {
                db
                    .file_store_binary_repo()
                    .save_file(conn, source_path, file_name)
                    .await
            }
        }
    }

    pub async fn delete_by_filename(&self, file_name: &str) -> Result<u64, LightSpeedError> {
        info!("FileStoreService - Delete file [{}]", file_name);
        match &self.repo_manager {
            FileStoreRepoManager::FS{fs} => fs.delete_by_filename(file_name).await,
            FileStoreRepoManager::DB{db} => {
                db
                    .c3p0()
                    .transaction(|mut conn| async move {
                        repo_manager
                            .file_store_binary_repo()
                            .delete_by_filename(&mut conn, file_name)
                            .await
                    })
                    .await
            }
        }
    }

    pub async fn delete_by_filename_with_conn(
        &self,
        conn: &mut RepoManager::Conn,
        file_name: &str,
    ) -> Result<u64, LightSpeedError> {
        info!("FileStoreService - Delete file [{}]", file_name);
        match &self.repo_manager {
            FileStoreRepoManager::FS{fs} => fs.delete_by_filename(file_name).await,
            FileStoreRepoManager::DB{db} => {
                db
                    .file_store_binary_repo()
                    .delete_by_filename(conn, file_name)
                    .await
            }
        }
    }

     */
}
