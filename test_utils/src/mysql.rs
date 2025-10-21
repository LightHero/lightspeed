use c3p0::sqlx::MySqlPool;
use c3p0::sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use c3p0::*;
use testcontainers::mysql::Mysql;
use testcontainers::testcontainers::ContainerAsync;
use testcontainers::testcontainers::runners::AsyncRunner;

/// Starts a new Mysql database in a container and creates a new Sqlx pool connected to it.
///
/// The returned tuple contains the Sqlx pool and the container.
///
/// The container is started with the defaults, i.e. username and password are both "mysql" and
/// the database name is "mysql".
///
/// The returned Sqlx pool is configured to connect to the database with the same defaults as the
/// container.
pub async fn new_mysql_db() -> (MySqlC3p0Pool, ContainerAsync<Mysql>) {
    let node = Mysql::default().start().await.unwrap();

    let options = MySqlConnectOptions::new()
        // .username("mysql")
        // .password("mysql")
        .database("test")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(3306).await.unwrap())
        .ssl_mode(MySqlSslMode::Disabled);

    let pool = MySqlPool::connect_with(options).await.unwrap();

    let c3p0 = MySqlC3p0Pool::new(pool);

    (c3p0, node)
}
