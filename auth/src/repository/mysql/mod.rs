use std::sync::Arc;

use crate::repository::AuthRepositoryManager;
use ::sqlx::{migrate::Migrator, *};
use c3p0::{sqlx::*, IdType};
use lightspeed_core::error::LsError;

pub mod mysql_auth_account;
pub mod mysql_token;

static MIGRATOR: Migrator = ::sqlx::migrate!("src_resources/db/mysql/migrations");

#[derive(Clone)]
pub struct MysqlAuthRepositoryManager {
    c3p0: SqlxMySqlC3p0Pool,
}

impl MysqlAuthRepositoryManager {
    pub fn new(c3p0: SqlxMySqlC3p0Pool) -> MysqlAuthRepositoryManager {
        MysqlAuthRepositoryManager { c3p0 }
    }
}