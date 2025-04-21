use crate::config::RepositoryType;
use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel};
use crate::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager, FileStoreDataRepository};
use crate::repository::opendal::opendal_file_store_binary::OpendalFileStoreBinaryRepository;
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
    repositories: HashMap<String, RepositoryStoreType>,
}

#[derive(Clone)]
enum RepositoryStoreType {
    DB,
    Opendal(OpendalFileStoreBinaryRepository),
}

impl<RepoManager: DBFileStoreRepositoryManager> LsFileStoreService<RepoManager> {
    pub fn new(repo_manager: &RepoManager, repositories: HashMap<String, RepositoryType>) -> Self {
        LsFileStoreService {
            c3p0: repo_manager.c3p0().clone(),
            db_binary_repo: repo_manager.file_store_binary_repo(),
            db_data_repo: repo_manager.file_store_data_repo(),
            repositories: repositories
                .into_iter()
                .map(|(name, repo)| {
                    let repo = match repo {
                        RepositoryType::DB => RepositoryStoreType::DB,
                        RepositoryType::Opendal(repo) => {
                            RepositoryStoreType::Opendal(OpendalFileStoreBinaryRepository::new(repo))
                        }
                    };
                    (name, repo)
                })
                .collect(),
        }
    }

    pub async fn read_file_data_by_id(&self, id: u64) -> Result<FileStoreDataModel, LsError> {
        self.c3p0.transaction(async |conn| self.read_file_data_by_id_with_conn(conn, id).await).await
    }

    pub async fn read_file_data_by_id_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        id: u64,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file by id [{}]", id);
        self.db_data_repo.fetch_one_by_id(conn, id).await
    }

    pub async fn exists_by_repository(&self, repository: &str, file_path: &str) -> Result<bool, LsError> {
        self.c3p0.transaction(async |conn| self.exists_by_repository_with_conn(conn, repository, file_path).await).await
    }

    pub async fn exists_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsError> {
        debug!("LsFileStoreService - Check if file exists by repository [{:?}]", repository);
        self.db_data_repo.exists_by_repository(conn, repository, file_path).await
    }

    pub async fn read_file_data_by_repository(
        &self,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        self.c3p0
            .transaction(async |conn| self.read_file_data_by_repository_with_conn(conn, repository, file_path).await)
            .await
    }

    pub async fn read_file_data_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{:?}]", repository);
        self.db_data_repo.fetch_one_by_repository(conn, repository, file_path).await
    }

    pub async fn read_all_file_data_by_repository(
        &self,
        repository: &str,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.read_all_file_data_by_repository_with_conn(conn, repository, offset, max, sort).await
            })
            .await
    }

    pub async fn read_all_file_data_by_repository_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        repository: &str,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{:?}]", repository);
        self.db_data_repo.fetch_all_by_repository(conn, repository, offset, max, sort).await
    }

    pub async fn read_file_content(&self, repository: &str, file_path: &str) -> Result<BinaryContent<'_>, LsError> {
        debug!("LsFileStoreService - Read repository [{}] file [{}]", repository, file_path);
        match self.get_repository(repository)? {
            RepositoryStoreType::DB => {
                self.c3p0
                    .transaction(async |conn| self.db_binary_repo.read_file(conn, repository, file_path).await)
                    .await
            }
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.read_file(file_path).await
            }
        }
    }

    pub async fn read_file_content_with_conn(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        repository: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        debug!("LsFileStoreService - Read repository [{}] file [{}]", repository, file_path);
        match self.get_repository(repository)? {
            RepositoryStoreType::DB => self.db_binary_repo.read_file(conn, repository, file_path).await,
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.read_file(file_path).await
            }
        }
    }

    pub async fn save_file_with_conn<'a>(
        &self,
        conn: &mut RepoManager::Tx<'_>,
        repository: String,
        file_path: String,
        filename: String,
        content_type: String,
        content: &'a BinaryContent<'a>,
    ) -> Result<FileStoreDataModel, LsError> {
        info!(
            "LsFileStoreService - Repository [{}] - Save file [{}], content type [{}]",
            repository, file_path, content_type
        );

        match self.get_repository(&repository)? {
            RepositoryStoreType::DB => {
                self.db_binary_repo.save_file(conn, &repository, &file_path, content).await?;
            }
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.save_file(&file_path, content).await?;
            }
        };

        self.db_data_repo
            .save(
                conn,
                NewModel::new(FileStoreDataData {
                    repository,
                    file_path,
                    content_type,
                    filename,
                    created_date_epoch_seconds: current_epoch_seconds(),
                }),
            )
            .await
    }

    pub async fn save_file<'a>(
        &self,
        repository: String,
        file_path: String,
        filename: String,
        content_type: String,
        content: &'a BinaryContent<'a>,
    ) -> Result<FileStoreDataModel, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.save_file_with_conn(conn, repository, file_path, filename, content_type, content).await
            })
            .await
    }

    pub async fn delete_file_by_id(&self, id: u64) -> Result<(), LsError> {
        self.c3p0.transaction(async |conn| self.delete_file_by_id_with_conn(conn, id).await).await
    }

    pub async fn delete_file_by_id_with_conn(&self, conn: &mut RepoManager::Tx<'_>, id: u64) -> Result<(), LsError> {
        info!("LsFileStoreService - Delete file by id [{}]", id);

        let file_data = self.read_file_data_by_id_with_conn(conn, id).await?;

        self.db_data_repo.delete_by_id(conn, id).await?;

        match self.get_repository(&file_data.data.repository)? {
            RepositoryStoreType::DB => self
                .db_binary_repo
                .delete_file(conn, &file_data.data.repository, &file_data.data.file_path)
                .await
                .map(|_| ()),
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.delete_by_filename(&file_data.data.file_path).await
            }
        }
    }

    #[inline]
    fn get_repository(&self, repository_name: &str) -> Result<&RepositoryStoreType, LsError> {
        self.repositories.get(repository_name).ok_or_else(|| LsError::BadRequest {
            message: format!("LsFileStoreService - Cannot find FS repository with name [{repository_name}]"),
            code: ErrorCodes::NOT_FOUND,
        })
    }
}
