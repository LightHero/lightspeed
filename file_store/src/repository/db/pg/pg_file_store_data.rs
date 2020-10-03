use crate::model::{FileStoreDataData, FileStoreDataDataCodec, FileStoreDataModel, Repository};
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
            repo: C3p0JsonBuilder::new("LS_FILE_STORE_DATA")
                .build_with_codec(FileStoreDataDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreDataRepository for PgFileStoreDataRepository {
    type Conn = PgConnection;

    async fn fetch_one_by_id(
        &self,
        conn: &mut Self::Conn,
        id: IdType,
    ) -> Result<FileStoreDataModel, LightSpeedError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn fetch_one_by_repository(
        &self,
        conn: &mut Self::Conn,
        repository: &Repository,
    ) -> Result<FileStoreDataModel, LightSpeedError> {
        let sql =
            "SELECT id, version, DATA FROM LS_FILE_STORE_DATA WHERE (data -> 'repository' ->> '_json_tag') = $1 AND (data -> 'repository' ->> 'repository_name') = $2 AND (data -> 'repository' ->> 'file_path') = $3";

        let (db, repository_name, file_path) = match repository {
            Repository::DB {
                file_path,
                repository_name,
            } => (repository.as_ref(), repository_name, file_path),
            Repository::FS {
                file_path,
                repository_name,
            } => (repository.as_ref(), repository_name, file_path),
        };

        Ok(self
            .repo
            .fetch_one_with_sql(conn, sql, &[&db, repository_name, file_path])
            .await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<FileStoreDataData>,
    ) -> Result<FileStoreDataModel, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn delete_by_id(
        &self,
        conn: &mut Self::Conn,
        id: IdType,
    ) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete_by_id(conn, &id).await?)
    }
}
