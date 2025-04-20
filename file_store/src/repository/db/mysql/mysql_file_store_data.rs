use crate::model::{FileStoreDataData, FileStoreDataDataCodec, FileStoreDataModel, Repository, RepositoryFile};
use crate::repository::db::FileStoreDataRepository;
use ::sqlx::{query, MySql, Row, Transaction};
use c3p0::sqlx::error::into_c3p0_error;
use c3p0::{sqlx::*, *};
use lightspeed_core::error::LsError;

#[derive(Clone)]
pub struct MySqlFileStoreDataRepository {
    repo: SqlxMySqlC3p0Json<u64, FileStoreDataData, FileStoreDataDataCodec>,
}

impl Default for MySqlFileStoreDataRepository {
    fn default() -> Self {
        MySqlFileStoreDataRepository {
            repo: SqlxMySqlC3p0JsonBuilder::new("LS_FILE_STORE_DATA").build_with_codec(FileStoreDataDataCodec {}),
        }
    }
}

impl FileStoreDataRepository for MySqlFileStoreDataRepository {
    type Tx<'a> = Transaction<'a, MySql>;

    async fn exists_by_repository(&self, tx: &mut Self::Tx<'_>, repository: &RepositoryFile) -> Result<bool, LsError> {
        let sql = "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE (data -> '$.repository._json_tag') = ? AND (data -> '$.repository.repository_name') = ? AND (data -> '$.repository.file_path') = ?)";

        let repo_info = RepoFileInfo::new(repository);

        let res = query(sql)
            .bind(repo_info.repo_type)
            .bind(repo_info.repository_name)
            .bind(repo_info.file_path)
            .fetch_one(tx.as_mut())
            .await
            .and_then(|row| row.try_get(0))
            .map_err(into_c3p0_error)?;
        Ok(res)
    }

    async fn fetch_one_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<FileStoreDataModel, LsError> {
        Ok(self.repo.fetch_one_by_id(tx, &id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LsError> {
        let sql = format!(
            r#"
            {}
            WHERE (data -> '$.repository._json_tag') = ? AND (data -> '$.repository.repository_name') = ? AND (data -> '$.repository.file_path') = ?
        "#,
            self.repo.queries().find_base_sql_query
        );
        let repo_info = RepoFileInfo::new(repository);

        Ok(self
            .repo
            .fetch_one_with_sql(
                tx,
                ::sqlx::query(&sql).bind(repo_info.repo_type).bind(repo_info.repository_name).bind(repo_info.file_path),
            )
            .await?)
    }

    async fn fetch_all_by_repository(
        &self,
        tx: &mut Self::Tx<'_>,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LsError> {
        let sql = format!(
            r#"{}
               WHERE (data -> '$.repository._json_tag') = ? AND (data -> '$.repository.repository_name') = ?
                order by id {}
                limit {}
                offset {}
               "#,
            self.repo.queries().find_base_sql_query,
            sort.to_sql(),
            max,
            offset
        );

        let repo_info = RepoInfo::new(repository);

        Ok(self
            .repo
            .fetch_all_with_sql(tx, ::sqlx::query(&sql).bind(repo_info.repo_type).bind(repo_info.repository_name))
            .await?)
    }

    async fn save(
        &self,
        tx: &mut Self::Tx<'_>,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LsError> {
        Ok(self.repo.save(tx, model).await?)
    }

    async fn delete_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<u64, LsError> {
        Ok(self.repo.delete_by_id(tx, &id).await?)
    }
}

struct RepoFileInfo<'a> {
    repo_type: &'a str,
    repository_name: &'a str,
    file_path: &'a str,
}

impl<'a> RepoFileInfo<'a> {
    fn new(repo: &'a RepositoryFile) -> Self {
        match repo {
            RepositoryFile::DB { file_path, repository_name } => {
                RepoFileInfo { repo_type: repo.as_ref(), repository_name, file_path }
            }
            RepositoryFile::FS { file_path, repository_name } => {
                RepoFileInfo { repo_type: repo.as_ref(), repository_name, file_path }
            }
        }
    }
}

struct RepoInfo<'a> {
    repo_type: &'a str,
    repository_name: &'a str,
}

impl<'a> RepoInfo<'a> {
    fn new(repo: &'a Repository) -> Self {
        match repo {
            Repository::DB { repository_name } => RepoInfo { repo_type: repo.as_ref(), repository_name },
            Repository::FS { repository_name } => RepoInfo { repo_type: repo.as_ref(), repository_name },
        }
    }
}
