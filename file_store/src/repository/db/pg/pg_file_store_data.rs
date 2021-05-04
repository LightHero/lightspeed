use crate::model::{FileStoreDataData, FileStoreDataDataCodec, FileStoreDataModel, Repository, RepositoryFile};
use crate::repository::db::FileStoreDataRepository;
use c3p0::postgres::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;

#[derive(Clone)]
pub struct PgFileStoreDataRepository {
    repo: PgC3p0Json<FileStoreDataData, FileStoreDataDataCodec>,
}

impl Default for PgFileStoreDataRepository {
    fn default() -> Self {
        PgFileStoreDataRepository {
            repo: C3p0JsonBuilder::new("LS_FILE_STORE_DATA").build_with_codec(FileStoreDataDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreDataRepository for PgFileStoreDataRepository {
    type Conn = PgConnection;

    async fn exists_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &RepositoryFile,
    ) -> Result<bool, LightSpeedError> {
        let sql =
            "SELECT EXISTS (SELECT 1 FROM LS_FILE_STORE_DATA WHERE (data -> 'repository' ->> '_json_tag') = $1 AND (data -> 'repository' ->> 'repository_name') = $2 AND (data -> 'repository' ->> 'file_path') = $3)";

        let repo_info = RepoFileInfo::new(repository);

        Ok(conn.fetch_one_value(sql, &[&repo_info.repo_type, &repo_info.repository_name, &repo_info.file_path]).await?)
    }

    async fn fetch_one_by_id(&self, conn: &mut Self::Conn, id: IdType) -> Result<FileStoreDataModel, LightSpeedError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &RepositoryFile,
    ) -> Result<FileStoreDataModel, LightSpeedError> {
        let sql =
            "SELECT id, version, DATA FROM LS_FILE_STORE_DATA WHERE (data -> 'repository' ->> '_json_tag') = $1 AND (data -> 'repository' ->> 'repository_name') = $2 AND (data -> 'repository' ->> 'file_path') = $3";

        let repo_info = RepoFileInfo::new(repository);

        Ok(self
            .repo
            .fetch_one_with_sql(conn, sql, &[&repo_info.repo_type, &repo_info.repository_name, &repo_info.file_path])
            .await?)
    }

    async fn fetch_all_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &Repository,
        offset: usize,
        max: usize,
        sort: &OrderBy,
    ) -> Result<Vec<FileStoreDataModel>, LightSpeedError> {
        let sql = format!(
            r#"SELECT id, version, DATA FROM LS_FILE_STORE_DATA
               WHERE (data -> 'repository' ->> '_json_tag') = $1 AND (data -> 'repository' ->> 'repository_name') = $2
                order by id {}
                limit {}
                offset {}
               "#,
            sort.to_sql(),
            max,
            offset
        );

        let repo_info = RepoInfo::new(repository);

        Ok(self.repo.fetch_all_with_sql(conn, &sql, &[&repo_info.repo_type, &repo_info.repository_name]).await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn delete_by_id(&self, conn: &mut Self::Conn, id: IdType) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete_by_id(conn, &id).await?)
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
