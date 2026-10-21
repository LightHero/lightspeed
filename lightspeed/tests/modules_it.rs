//! The db-backed modules each bring their own migrations, and an application usually runs them
//! all in a single database: they must be able to start together, on every startup.

#![cfg(all(feature = "core", feature = "account_management", feature = "file_store", feature = "scheduler"))]

use lightspeed::account_management::LsAMModule;
use lightspeed::account_management::config::AMConfig;
use lightspeed::account_management::repository::AMRepositoryManager;
use lightspeed::core::module::LsModule;
use lightspeed::file_store::LsFileStoreModule;
use lightspeed::file_store::config::FileStoreConfig;
use lightspeed::file_store::repository::db::DBFileStoreRepositoryManager;

async fn start_modules<AM: AMRepositoryManager, FS: DBFileStoreRepositoryManager>(am: AM, fs: FS) {
    LsAMModule::new(am, AMConfig::default())
        .expect("account_management must be created")
        .start()
        .await
        .expect("account_management must start");
    LsFileStoreModule::new(fs, FileStoreConfig::default())
        .expect("file_store must be created")
        .start()
        .await
        .expect("file_store must start");
}

#[cfg(feature = "postgres")]
mod postgres {
    use lightspeed::account_management::repository::postgres::PgAMRepositoryManager;
    use lightspeed::file_store::repository::db::postgres::PgFileStoreRepositoryManager;
    use lightspeed::scheduler::PgScheduleRepository;
    use lightspeed_test_utils::pg::new_pg_db;

    #[tokio::test]
    async fn modules_start_together() {
        let (c3p0, _node) = new_pg_db().await;

        for _ in 0..2 {
            super::start_modules(
                PgAMRepositoryManager::new(c3p0.clone()),
                PgFileStoreRepositoryManager::new(c3p0.clone()),
            )
            .await;
            PgScheduleRepository::init(c3p0.clone()).await.expect("scheduler must start");
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql {
    use lightspeed::account_management::repository::mysql::MyAMSqlRepositoryManager;
    use lightspeed::file_store::repository::db::mysql::MySqlFileStoreRepositoryManager;
    use lightspeed::scheduler::MySqlScheduleRepository;
    use lightspeed_test_utils::mysql::new_mysql_db;

    #[tokio::test]
    async fn modules_start_together() {
        let (c3p0, _node) = new_mysql_db().await;

        for _ in 0..2 {
            super::start_modules(
                MyAMSqlRepositoryManager::new(c3p0.clone()),
                MySqlFileStoreRepositoryManager::new(c3p0.clone()),
            )
            .await;
            MySqlScheduleRepository::init(c3p0.clone()).await.expect("scheduler must start");
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use lightspeed::account_management::repository::sqlite::SqliteAMRepositoryManager;
    use lightspeed::file_store::repository::db::sqlite::SqliteFileStoreRepositoryManager;
    use lightspeed::scheduler::SqliteScheduleRepository;
    use lightspeed_test_utils::sqlite::new_sqlite_db;

    #[tokio::test]
    async fn modules_start_together() {
        let c3p0 = new_sqlite_db().await;

        for _ in 0..2 {
            super::start_modules(
                SqliteAMRepositoryManager::new(c3p0.clone()),
                SqliteFileStoreRepositoryManager::new(c3p0.clone()),
            )
            .await;
            SqliteScheduleRepository::init(c3p0.clone()).await.expect("scheduler must start");
        }
    }
}
