use crate::dto::FileData;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::pg::*;
use lightspeed_core::error::LightSpeedError;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct PgFileStoreBinaryRepository {
    table_name: &'static str,
}

impl Default for PgFileStoreBinaryRepository {
    fn default() -> Self {
        PgFileStoreBinaryRepository {
            table_name: "LS_FILE_STORE_BINARY",
        }
    }
}

#[async_trait::async_trait]
impl DBFileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type Conn = PgConnectionAsync;

    async fn read_file(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
    ) -> Result<FileData, LightSpeedError> {
        let sql = &format!("SELECT DATA FROM {} WHERE filename = $1", self.table_name);

        let content = conn
            .fetch_one(&sql, &[&file_name], |row| {
                let content: Vec<u8> = row.try_get(0).map_err(into_c3p0_error)?;
                Ok(content)
            })
            .await?;
        Ok(FileData::InMemory { content })
    }

    async fn save_file(
        &self,
        conn: &mut Self::Conn,
        source_path: &str,
        file_name: &str,
    ) -> Result<(), LightSpeedError> {
        let mut file =
            File::open(source_path)
                .await
                .map_err(|err| LightSpeedError::BadRequest {
                    message: format!(
                        "PgFileStoreBinaryRepository - Cannot open file [{}]. Err: {}",
                        file_name, err
                    ),
                })?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)
            .await
            .map_err(|err| LightSpeedError::BadRequest {
                message: format!(
                    "PgFileStoreBinaryRepository - Cannot read file [{}]. Err: {}",
                    file_name, err
                ),
            })?;

        let sql = &format!(
            "INSERT INTO {} (filename, data) VALUES ($1, $2)",
            self.table_name
        );

        Ok(conn
            .execute(&sql, &[&file_name, &contents])
            .await
            .map(|_| ())?)
    }

    async fn delete_by_filename(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
    ) -> Result<u64, LightSpeedError> {
        let sql = &format!("DELETE FROM {} WHERE filename = $1", self.table_name);
        Ok(conn.execute(&sql, &[&file_name]).await?)
    }
}
