use crate::repository::OutboxRepositoryManager;
use c3p0::sqlx::{migrate::Migrator, *};
use c3p0::*;
use lightspeed_core::error::LsError;
use mysql_task::MySqlTaskRepository;

pub mod mysql_task;

static MIGRATOR: Migrator = c3p0::sqlx::migrate!("src_resources/db/mysql/migrations");

#[derive(Clone)]
pub struct MySqlOutboxRepositoryManager {
    c3p0: MySqlC3p0Pool,
}

impl MySqlOutboxRepositoryManager {
    pub fn new(c3p0: MySqlC3p0Pool) -> MySqlOutboxRepositoryManager {
        MySqlOutboxRepositoryManager { c3p0 }
    }
}

impl OutboxRepositoryManager for MySqlOutboxRepositoryManager {
    type DB = MySql;
    type C3P0 = MySqlC3p0Pool;
    type TaskRepo = MySqlTaskRepository;

    fn c3p0(&self) -> &Self::C3P0 {
        &self.c3p0
    }

    async fn start(&self) -> Result<(), LsError> {
        MIGRATOR.run(self.c3p0.pool()).await.map_err(|err| LsError::ModuleStartError {
            message: format!("MySqlAuthRepositoryManager - db migration failed: {err:?}"),
        })
    }

    fn task_repo(&self) -> Self::TaskRepo {
        MySqlTaskRepository::new()
    }
}
