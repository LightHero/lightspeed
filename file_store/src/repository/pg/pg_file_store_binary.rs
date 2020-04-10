use crate::repository::FileStoreBinaryRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};
use std::path::Path;

#[derive(Clone)]
pub struct PgFileStoreBinaryRepository {
    repo: PgC3p0JsonAsync<FileStoreData, FileStoreDataCodec>,
}

impl Default for PgFileStoreBinaryRepository {
    fn default() -> Self {
        PgFileStoreBinaryRepository {
            repo: C3p0JsonBuilder::new("LS_FILE_STORE_BINARY").build_with_codec(FileStoreDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type Conn = PgConnectionAsync;

    async fn save_file(&self, source_path: &str, file_name: &str) -> Result<(), LightSpeedError> {
        unimplemented!()
    }

    async fn delete_by_filename(&self, file_name: &str) -> Result<(), LightSpeedError> {
        unimplemented!()
    }
}

