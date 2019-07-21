use c3p0::*;
use c3p0::pg::r2d2::{Pool, PostgresConnectionManager, TlsMode};
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use testcontainers::*;

use lightspeed_auth::AuthModule;
use ls_core::module::Module;

mod tests;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        AuthModule,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    AuthModule,
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = C3p0Builder::new(pool);

    let mut auth_module = AuthModule::new(pool);
    auth_module.start().unwrap();

    (auth_module, node)
}

pub fn test<F: FnOnce(&AuthModule)>(callback: F) {
    SINGLETON.get(|(auth_module, _)| {
        callback(&auth_module);
    });
}
