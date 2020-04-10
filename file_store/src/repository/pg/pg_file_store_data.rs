use crate::repository::FileStoreDataRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};

#[derive(Clone)]
pub struct PgFileStoreDataRepository {
    repo: PgC3p0JsonAsync<FileStoreData, FileStoreDataCodec>,
}

impl Default for PgFileStoreDataRepository {
    fn default() -> Self {
        PgFileStoreDataRepository {
            repo: C3p0JsonBuilder::new("LS_FILE_STORE_DATA").build_with_codec(FileStoreDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreDataRepository for PgFileStoreDataRepository {
    type Conn = PgConnectionAsync;

}

