use crate::config::RepositoryType;
use crate::model::{BinaryContent, FileStoreDataData, FileStoreDataModel};
use crate::repository::db::{DBFileStoreBinaryRepository, DBFileStoreRepositoryManager, FileStoreDataRepository};
use crate::repository::opendal::opendal_file_store_binary::OpendalFileStoreBinaryRepository;
use c3p0::sql::OrderBy;
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::utils::current_epoch_seconds;
use log::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LsFileStoreService<RepoManager: DBFileStoreRepositoryManager> {
    c3p0: RepoManager::C3P0,
    db_binary_repo: RepoManager::FileStoreBinaryRepo,
    db_data_repo: RepoManager::FileStoreDataRepo,
    repositories: HashMap<String, RepositoryStoreType>,
    save_max_size_bytes: Option<usize>,
}

#[derive(Clone)]
enum RepositoryStoreType {
    DB,
    Opendal(OpendalFileStoreBinaryRepository),
}

impl<RepoManager: DBFileStoreRepositoryManager> LsFileStoreService<RepoManager> {
    pub fn new(
        repo_manager: &RepoManager,
        repositories: HashMap<String, RepositoryType>,
        save_max_size_bytes: Option<usize>,
    ) -> Self {
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
            save_max_size_bytes,
        }
    }

    pub async fn read_file_data_by_id(&self, id: i64) -> Result<FileStoreDataModel, LsError> {
        self.c3p0.transaction(async |conn| self.read_file_data_by_id_with_conn(conn, id).await).await
    }

    pub async fn read_file_data_by_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        id: i64,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file by id [{id}]");
        self.db_data_repo.fetch_one_by_id(conn, id).await
    }

    pub async fn exists_by_repository(&self, repository: &str, file_path: &str) -> Result<bool, LsError> {
        self.c3p0.transaction(async |conn| self.exists_by_repository_with_conn(conn, repository, file_path).await).await
    }

    pub async fn exists_by_repository_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> Result<bool, LsError> {
        debug!("LsFileStoreService - Check if file exists by repository [{repository:?}]");
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
        conn: &mut <RepoManager::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> Result<FileStoreDataModel, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{repository:?}]");
        self.db_data_repo.fetch_one_by_repository(conn, repository, file_path).await
    }

    pub async fn read_all_file_data_by_repository(
        &self,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.read_all_file_data_by_repository_with_conn(conn, repository, offset, max, sort).await
            })
            .await
    }

    pub async fn read_all_file_data_by_repository_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        repository: &str,
        offset: usize,
        max: usize,
        sort: OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        debug!("LsFileStoreService - Read file data by repository [{repository:?}]");
        self.db_data_repo.fetch_all_by_repository(conn, repository, offset, max, sort).await
    }

    /// Returns the binary content. Backends that support streaming the read
    /// (Postgres via Large Objects) use `read_file_streamed`, which owns its
    /// own connection and exposes a `BinaryContent::Stream`. Backends that
    /// can't (MySQL, SQLite) buffer into `BinaryContent::InMemory`. The
    /// `_with_conn` variant always materializes — it can't return a stream
    /// that outlives the caller's transaction borrow.
    pub async fn read_file_content(
        &self,
        repository: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'static>, LsError> {
        debug!("LsFileStoreService - Read repository [{repository}] file [{file_path}]");
        validate_safe_relative_path("file_path", file_path)?;
        match self.get_repository(repository)? {
            RepositoryStoreType::DB => self.db_binary_repo.read_file_streamed(repository, file_path).await,
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.read_file(file_path).await
            }
        }
    }

    pub async fn read_file_content_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        repository: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsError> {
        debug!("LsFileStoreService - Read repository [{repository}] file [{file_path}]");
        validate_safe_relative_path("file_path", file_path)?;
        match self.get_repository(repository)? {
            RepositoryStoreType::DB => self.db_binary_repo.read_file(conn, repository, file_path).await,
            RepositoryStoreType::Opendal(opendal_file_store_binary_repository) => {
                opendal_file_store_binary_repository.read_file(file_path).await
            }
        }
    }

    pub async fn save_file_with_conn<'a>(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        repository: String,
        file_path: String,
        filename: String,
        content_type: String,
        content: &'a BinaryContent<'a>,
    ) -> Result<FileStoreDataModel, LsError> {
        info!(
            "LsFileStoreService - Repository [{repository}] - Save file [{file_path}], content type [{content_type}]"
        );

        // Reject path-traversal patterns before they reach any backend.
        validate_safe_relative_path("file_path", &file_path)?;
        validate_safe_relative_path("filename", &filename)?;

        if let Some(max) = self.save_max_size_bytes {
            self.enforce_save_max_size(content, max).await?;
        }

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
                NewRecord::new(FileStoreDataData {
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

    pub async fn delete_file_by_id(&self, id: i64) -> Result<(), LsError> {
        self.c3p0.transaction(async |conn| self.delete_file_by_id_with_conn(conn, id).await).await
    }

    pub async fn delete_file_by_id_with_conn(
        &self,
        conn: &mut <RepoManager::DB as Database>::Connection,
        id: i64,
    ) -> Result<(), LsError> {
        info!("LsFileStoreService - Delete file by id [{id}]");

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
            code: "",
        })
    }

    /// Reject a save that would exceed `max` bytes before any read
    /// materializes the content. For `OpenDal` sources the size comes from
    /// `Operator::stat`, so the bytes are never pulled into memory at all.
    /// `Stream` sources have no advance size info — the cap is enforced
    /// incrementally as the stream is consumed by the destination repo
    /// (the chunk-by-chunk Postgres `lowrite` path will surface an error if
    /// the cumulative byte count crosses the cap; for now we simply allow
    /// the save to proceed and rely on downstream limits).
    /// Applies regardless of destination — DB column or OpenDal repository.
    async fn enforce_save_max_size(&self, content: &BinaryContent<'_>, max: usize) -> Result<(), LsError> {
        let actual: u64 = match content {
            BinaryContent::InMemory { content } => content.len() as u64,
            BinaryContent::OpenDal { operator, path } => operator
                .stat(path)
                .await
                .map_err(|err| LsError::BadRequest {
                    message: format!("LsFileStoreService - Cannot stat file [{path}]: {err:?}"),
                    code: "",
                })?
                .content_length(),
            BinaryContent::Stream { .. } => {
                // No advance size; let the save proceed and any future
                // backpressure / hard limits at the storage layer apply.
                return Ok(());
            }
        };
        if actual > max as u64 {
            return Err(LsError::BadRequest {
                message: format!("LsFileStoreService - File size [{actual}] exceeds save_max_size_bytes [{max}]"),
                code: "",
            });
        }
        Ok(())
    }
}

/// Reject obvious path-traversal / escape patterns in user-supplied file
/// names and paths. We treat the value as a relative path inside the
/// configured repository root and refuse:
///
/// * empty strings (no legitimate use),
/// * strings containing a NUL byte (filesystem boundary attack — many
///   OS path APIs treat NUL as terminator and ignore everything after it),
/// * leading `/` or `\\` (would escape the FS-backed Opendal root),
/// * any segment that is exactly `..` when split on either path separator
///   (catches `../etc/passwd`, `foo/../bar`, `foo\\..\\bar`, etc.).
fn validate_safe_relative_path(field: &'static str, value: &str) -> Result<(), LsError> {
    if value.is_empty() {
        return Err(LsError::BadRequest {
            message: format!("LsFileStoreService - {field} cannot be empty"),
            code: "",
        });
    }
    if value.contains('\0') {
        return Err(LsError::BadRequest {
            message: format!("LsFileStoreService - {field} contains NUL byte"),
            code: "",
        });
    }
    if value.starts_with('/') || value.starts_with('\\') {
        return Err(LsError::BadRequest {
            message: format!("LsFileStoreService - {field} must be a relative path, got [{value}]"),
            code: "",
        });
    }
    for segment in value.split(['/', '\\']) {
        if segment == ".." {
            return Err(LsError::BadRequest {
                message: format!(
                    "LsFileStoreService - {field} contains parent-directory traversal segment, got [{value}]"
                ),
                code: "",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod path_validation_tests {
    use super::validate_safe_relative_path;
    use lightspeed_core::error::LsError;

    fn assert_rejected(value: &str) {
        match validate_safe_relative_path("file_path", value) {
            Err(LsError::BadRequest { code, .. }) => {
                assert_eq!("", code, "value [{value}] rejected with wrong code");
            }
            other => panic!("expected BadRequest(PARSE_ERROR) for [{value}], got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty() {
        assert_rejected("");
    }

    #[test]
    fn rejects_nul_byte() {
        assert_rejected("foo\0bar.txt");
    }

    #[test]
    fn rejects_absolute_path() {
        assert_rejected("/etc/passwd");
        assert_rejected("\\Windows\\system32");
    }

    #[test]
    fn rejects_parent_segments() {
        assert_rejected("..");
        assert_rejected("../etc/passwd");
        assert_rejected("foo/../bar");
        assert_rejected("foo/bar/..");
        // Backslash separator (Windows-style) must also be caught.
        assert_rejected("foo\\..\\bar");
    }

    #[test]
    fn allows_clean_relative_paths() {
        for ok in &[
            "file.txt",
            "folder/file.txt",
            "deeply/nested/sub/folder/file.txt",
            // `..` only as a *substring* of a segment is fine — we split on
            // separators and require the whole segment to be `..`.
            "..hidden",
            "weird..name.txt",
            "folder/..hidden/file",
        ] {
            assert!(validate_safe_relative_path("file_path", ok).is_ok(), "rejected legit path [{ok}]");
        }
    }
}
