use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::postgres::*;
use lightspeed_core::error::{ErrorCodes, LightSpeedError};
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
    type Conn = PgConnection;

    async fn read_file(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
    ) -> Result<BinaryContent, LightSpeedError> {
        let sql = &format!("SELECT DATA FROM {} WHERE filename = $1", self.table_name);

        let content = conn
            .fetch_one(&sql, &[&file_name], |row| {
                let content: Vec<u8> = row.try_get(0).map_err(into_c3p0_error)?;
                Ok(content)
            })
            .await?;
        Ok(BinaryContent::InMemory { content })
    }

    async fn save_file(
        &self,
        conn: &mut Self::Conn,
        file_name: &str,
        content: BinaryContent,
    ) -> Result<(), LightSpeedError> {

        let binary_content = match content {
            BinaryContent::InMemory { content } => content,
            BinaryContent::FromFs { file_path } => {
                let mut file =
                    File::open(file_path)
                        .await
                        .map_err(|err| LightSpeedError::BadRequest {
                            message: format!(
                                "PgFileStoreBinaryRepository - Cannot open file [{}]. Err: {}",
                                file_name, err
                            ),
                            code: ErrorCodes::IO_ERROR,
                        })?;
                let mut contents = vec![];
                file.read_to_end(&mut contents)
                    .await
                    .map_err(|err| LightSpeedError::BadRequest {
                        message: format!(
                            "PgFileStoreBinaryRepository - Cannot read file [{}]. Err: {}",
                            file_name, err
                        ),
                        code: ErrorCodes::IO_ERROR,
                    })?;
                contents
            }
        };

        let sql = &format!(
            "INSERT INTO {} (filename, data) VALUES ($1, $2)",
            self.table_name
        );

        Ok(conn
            .execute(&sql, &[&file_name, &binary_content])
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
