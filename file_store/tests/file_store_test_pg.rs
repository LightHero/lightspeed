use c3p0::postgres::deadpool::{self, Runtime};
use c3p0::postgres::tokio_postgres::NoTls;
use c3p0::postgres::*;
use maybe_single::nio::*;
use testcontainers::*;

use lightspeed_core::module::Module;
use lightspeed_file_store::config::FileStoreConfig;
use lightspeed_file_store::repository::db::pg::PgFileStoreRepositoryManager;
use lightspeed_file_store::FileStoreModule;
use once_cell::sync::OnceCell;
use tokio::time::Duration;

mod tests;

pub type RepoManager = PgFileStoreRepositoryManager;

pub type MaybeType = (FileStoreModule<RepoManager>, Container<'static, clients::Cli, images::postgres::Postgres>);

async fn init() -> MaybeType {
    static DOCKER: OnceCell<clients::Cli> = OnceCell::new();

    let node = DOCKER.get_or_init(|| clients::Cli::default()).run(images::postgres::Postgres::default());

    let mut config = deadpool::postgres::Config::default();
    config.user = Some("postgres".to_owned());
    config.password = Some("postgres".to_owned());
    config.dbname = Some("postgres".to_owned());
    config.host = Some(format!("127.0.0.1"));
    config.port = Some(node.get_host_port(5432).unwrap());

    let mut pool_config = deadpool::managed::PoolConfig::default();
    pool_config.timeouts.create = Some(Duration::from_secs(5));
    pool_config.timeouts.recycle = Some(Duration::from_secs(5));
    pool_config.timeouts.wait = Some(Duration::from_secs(5));
    config.pool = Some(pool_config);

    let c3p0 = PgC3p0Pool::new(config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap());

    let repo_manager = RepoManager::new(c3p0.clone());

    let mut file_store_config = FileStoreConfig::build();
    file_store_config.fs_repo_base_folders.push(("REPO_ONE".to_owned(), "../target/repo_one".to_owned()));
    file_store_config.fs_repo_base_folders.push(("REPO_TWO".to_owned(), "../target/repo_two".to_owned()));

    let mut file_store_module = FileStoreModule::new(repo_manager, file_store_config).unwrap();
    {
        file_store_module.start().await.unwrap();
    }

    (file_store_module, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    static DATA: OnceCell<MaybeSingleAsync<MaybeType>> = OnceCell::new();
    DATA.get_or_init(|| MaybeSingleAsync::new(|| Box::pin(init()))).data(serial).await
}

pub fn test<F: std::future::Future>(f: F) -> F::Output {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Should create a tokio runtime")
    })
    .block_on(f)
}
