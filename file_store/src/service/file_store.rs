use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel, Repository, RepositoryFile, SaveRepository};
use crate::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager, FileStoreDataRepository};
use crate::repository::filesystem::fs_file_store_binary::FsFileStoreBinaryRepository;
use c3p0::*;
use lightspeed_core::error::{ErrorCodes, LsError};
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LsFileStoreService<RepoManager: DBFileStoreRepositoryManager> {
    c3p0: RepoManager::C3P0,
    db_binary_repo: RepoManager::FileStoreBinaryRepo,
    db_data_repo: RepoManager::FileStoreDataRepo,
    fs_repositories: HashMap<String, FsFileStoreBinaryRepository>,
}

impl<RepoManager: DBFileStoreRepositoryManager> LsFileStoreService<RepoManager> {
    pub fn new(repo_manager: &RepoManager, fs_repositories: Vec<(String, String)>) -> Self {
        LsFileStoreService {
            c3p0: repo_manager.c3p0().clone(),
            db_binary_repo: repo_manager.file_store_binary_repo(),
            db_data_repo: repo_manager.file_store_data_repo(),
            fs_repositories: fs_repositories
                .into_iter()
                .map(|(name, base_path)| (name, FsFileStoreBinaryRepository::new(base_path)))
                .collect(),
        }
    }

    pub async fn read_file_data_by_id(&self, id: IdType) -> Result<FileStoreDataModel, LsError> {
        self.c3p0.transaction(|conn| async { self.read_file_data_by_id_with_conn(conn, id).await }).await
    }

    pub async fn read_file_data_by_id_with_conn(
        &self,
        conn: &mut RepoManager::Tx,
        id: IdType,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file by id [{}]", id);
        self.db_data_repo.fetch_one_by_id(conn, id).await
    }

    pub async fn exists_by_repository(&self, repository: &RepositoryFile) -> Result<bool, LsError> {
        self.c3p0.transaction(|conn| async { self.exists_by_repository_with_conn(conn, repository).await }).await
    }

    pub async fn exists_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx,
        repository: &RepositoryFile,
    ) -> Result<bool, LsError> {
        debug!("LsFileStoreService - Check if file exists by repository [{:?}]", repository);
        self.db_data_repo.exists_by_repository(conn, repository).await
    }

    pub async fn read_file_data_by_repository(
        &self,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LsError> {
        self.c3p0
            .transaction(|conn| async { self.read_file_data_by_repository_with_conn(conn, repository).await })
            .await
    }

    pub async fn read_file_data_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{:?}]", repository);
        self.db_data_repo.fetch_one_by_repository(conn, repository).await
    }

    pub async fn read_all_file_data_by_repository(
        &self,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        self.c3p0
            .transaction(|conn| async {
                self.read_all_file_data_by_repository_with_conn(conn, repository, offset, max, sort).await
            })
            .await
    }

    pub async fn read_all_file_data_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{:?}]", repository);
        self.db_data_repo.fetch_all_by_repository(conn, repository, offset, max, sort).await
    }

    pub async fn read_file_content(&self, repository: &RepositoryFile) -> Result<BinaryContent<'_>, LsError> {
        debug!("LsFileStoreService - Read file [{:?}]", repository);
        match repository {
            RepositoryFile::DB { file_path, repository_name } => {
                self.c3p0
                    .transaction(|conn| async { self.db_binary_repo.read_file(conn, repository_name, file_path).await })
                    .await
            }
            RepositoryFile::FS { file_path, repository_name } => {
                self.read_file_content_from_fs(file_path, repository_name).await
            }
        }
    }

    pub async fn read_file_content_with_conn(
        &self,
        conn: &mut RepoManager::Tx,
        repository: &RepositoryFile,
    ) -> Result<BinaryContent<'_>, LsError> {
        debug!("LsFileStoreService - Read file [{:?}]", repository);
        match repository {
            RepositoryFile::DB { file_path, repository_name } => {
                self.db_binary_repo.read_file(conn, repository_name, file_path).await
            }
            RepositoryFile::FS { file_path, repository_name } => {
                self.read_file_content_from_fs(file_path, repository_name).await
            }
        }
    }

    pub async fn save_file_with_conn<'a>(
        &self,
        conn: &mut RepoManager::Tx,
        filename: String,
        content_type: String,
        content: &'a BinaryContent<'a>,
        repository: SaveRepository,
    ) -> Result<FileStoreDataModel, LsError> {
        info!(
            "LsFileStoreService - Save file [{}], content type [{}], destination [{:?}]",
            filename, content_type, repository
        );

        let repository_file = RepositoryFile::from(&repository, &filename);
        match repository {
            SaveRepository::FS { repository_name, .. } => {
                let repo = self.get_fs_repository(&repository_name)?;
                let file_path = repository_file.file_path();
                repo.save_file(file_path, content).await?;
            }
            SaveRepository::DB { repository_name, .. } => {
                let file_path = repository_file.file_path();
                self.db_binary_repo.save_file(conn, &repository_name, file_path, content).await?;
            }
        };

        self.db_data_repo
            .save(
                conn,
                NewModel::new(FileStoreDataData {
                    repository: repository_file,
                    content_type,
                    filename,
                    created_date_epoch_seconds: current_epoch_seconds(),
                }),
            )
            .await
    }

    pub async fn save_file<'a>(
        &self,
        filename: String,
        content_type: String,
        content: &'a BinaryContent<'a>,
        repository: SaveRepository,
    ) -> Result<FileStoreDataModel, LsError> {
        self.c3p0
            .transaction(|conn| async {
                self.save_file_with_conn(conn, filename, content_type, content, repository).await
            })
            .await
    }

    pub async fn delete_file_by_id(&self, id: IdType) -> Result<u64, LsError> {
        self.c3p0.transaction(|conn| async { self.delete_file_by_id_with_conn(conn, id).await }).await
    }

    pub async fn delete_file_by_id_with_conn(&self, conn: &mut RepoManager::Tx, id: IdType) -> Result<u64, LsError> {
        info!("LsFileStoreService - Delete file by id [{}]", id);

        let file_data = self.read_file_data_by_id_with_conn(conn, id).await?;

        self.db_data_repo.delete_by_id(conn, id).await?;

        match file_data.data.repository {
            RepositoryFile::DB { file_path, repository_name } => {
                self.db_binary_repo.delete_file(conn, &repository_name, &file_path).await
            }
            RepositoryFile::FS { file_path, repository_name } => {
                let repo = self.get_fs_repository(&repository_name)?;
                repo.delete_by_filename(&file_path).await
            }
        }
    }

    #[inline]
    async fn read_file_content_from_fs(
        &self,
        file_path: &str,
        repository_name: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        let repo = self.get_fs_repository(repository_name)?;
        repo.read_file(file_path).await
    }

    #[inline]
    fn get_fs_repository(&self, repository_name: &str) -> Result<&FsFileStoreBinaryRepository, LsError> {
        self.fs_repositories.get(repository_name).ok_or_else(|| LsError::BadRequest {
            message: format!("LsFileStoreService - Cannot find FS repository with name [{repository_name}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }
}
