use c3p0::sqlx::SqlxPgC3p0Pool;
use c3p0::sqlx::sqlx::PgPool;
use c3p0::sqlx::sqlx::postgres::PgConnectOptions;
use testcontainers::postgres::Postgres;
use testcontainers::testcontainers::ContainerAsync;
use testcontainers::testcontainers::runners::AsyncRunner;

/// Starts a new Postgres database in a container and creates a new Sqlx pool connected to it.
///
/// The returned tuple contains the Sqlx pool and the container.
///
/// The container is started with the defaults, i.e. username and password are both "postgres" and
/// the database name is "postgres".
///
/// The returned Sqlx pool is configured to connect to the database with the same defaults as the
/// container.
///
pub async fn new_pg_db() -> (SqlxPgC3p0Pool, ContainerAsync<Postgres>) {
    let node = Postgres::default().start().await.unwrap();

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .database("postgres")
        .host("127.0.0.1")
        .port(node.get_host_port_ipv4(5432).await.unwrap());

    let pool = PgPool::connect_with(options).await.unwrap();

    let c3p0 = SqlxPgC3p0Pool::new(pool);

    (c3p0, node)
}
