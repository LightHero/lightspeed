use crate::repository::FileStoreRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use crate::model::{FileStoreData, FileStoreDataCodec};

#[derive(Clone)]
pub struct PgFileStoreRepository {
    repo: PgC3p0JsonAsync<FileStoreData, FileStoreDataCodec>,
}

impl Default for PgFileStoreRepository {
    fn default() -> Self {
        PgFileStoreRepository {
            repo: C3p0JsonBuilder::new("LS_AUTH_ACCOUNT").build_with_codec(FileStoreDataCodec {}),
        }
    }
}

#[async_trait::async_trait]
impl FileStoreRepository for PgFileStoreRepository {
    type Conn = PgConnectionAsync;

}

