use crate::error::LsFileStoreError;
use crate::model::BinaryContent;
use crate::repository::db::DBFileStoreBinaryRepository;
use c3p0::PgC3p0Pool;
use c3p0::sqlx::{Postgres, Row, query};
use futures::StreamExt;
use futures::stream::BoxStream;
use sqlx::postgres::types::Oid;
use sqlx::{AssertSqlSafe, PgConnection};
use tokio::sync::{Mutex, mpsc};

// 64 KiB. Empirically a good trade-off between SQL round-trip overhead and
// per-message buffer size for the Postgres frontend/backend protocol.
const LO_CHUNK_SIZE: usize = 64 * 1024;

// Postgres LO open modes: see include/storage/large_object.h.
//   INV_WRITE = 0x00020000
//   INV_READ  = 0x00040000
const LO_INV_WRITE: i32 = 0x00020000;
const LO_INV_READ: i32 = 0x00040000;

// Bounded mpsc channel size for the streaming-read pipeline. Pinned at four
// in-flight chunks: enough to overlap database I/O with consumer work, small
// enough that peak memory is just `LO_CHUNK_SIZE * STREAM_CHANNEL_CAP` bytes.
const STREAM_CHANNEL_CAP: usize = 4;

#[derive(Clone)]
pub struct PgFileStoreBinaryRepository {
    table_name: &'static str,
    pool: PgC3p0Pool,
}

impl PgFileStoreBinaryRepository {
    pub fn new(pool: PgC3p0Pool) -> Self {
        PgFileStoreBinaryRepository { table_name: "LS_FILE_STORE_BINARY", pool }
    }
}

impl DBFileStoreBinaryRepository for PgFileStoreBinaryRepository {
    type DB = Postgres;

    /// Reads the LO referenced by the (repository, filepath)
    /// into a single in-memory buffer. For true
    /// streaming reads see [`Self::read_file_streamed`].
    async fn read_file(
        &self,
        tx: &mut PgConnection,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'_>, LsFileStoreError> {
        let oid = Self::lookup_oid(&mut *tx, self.table_name, repository_name, file_path).await?;
        let fd = Self::lo_open_read(&mut *tx, oid).await?;

        let mut buf: Vec<u8> = Vec::new();
        loop {
            let chunk = Self::loread_chunk(&mut *tx, fd).await?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }

        Self::lo_close(&mut *tx, fd).await?;
        Ok(BinaryContent::InMemory { content: std::borrow::Cow::Owned(buf) })
    }

    /// True streaming read: acquires a dedicated sqlx connection from the
    /// pool, opens its own transaction, and spawns a task that loops over
    /// `loread` while pushing each chunk into a bounded mpsc.
    async fn read_file_streamed(
        &self,
        repository_name: &str,
        file_path: &str,
    ) -> Result<BinaryContent<'static>, LsFileStoreError> {
        // Acquire an OWNED connection so we can move it into the producer
        // task. We manage BEGIN/COMMIT manually rather than going through
        // sqlx::Transaction because Transaction borrows from the connection
        // and that borrow can't be moved across a tokio::spawn boundary.
        let mut conn = self.pool.pool().acquire().await.map_err(|err| LsFileStoreError::OpenDalError {
            message: format!("PgFileStoreBinaryRepository - Cannot acquire connection: {err:?}"),
        })?;

        query("BEGIN").execute(&mut *conn).await?;

        let oid = match Self::lookup_oid(&mut *conn, self.table_name, repository_name, file_path).await {
            Ok(o) => o,
            Err(e) => {
                let _ = query("ROLLBACK").execute(&mut *conn).await;
                return Err(e);
            }
        };
        let fd = match Self::lo_open_read(&mut *conn, oid).await {
            Ok(f) => f,
            Err(e) => {
                let _ = query("ROLLBACK").execute(&mut *conn).await;
                return Err(e);
            }
        };

        let (sender, receiver) = mpsc::channel::<Result<Vec<u8>, LsFileStoreError>>(STREAM_CHANNEL_CAP);

        tokio::spawn(async move {
            loop {
                match Self::loread_chunk(&mut *conn, fd).await {
                    Ok(chunk) if chunk.is_empty() => break,
                    Ok(chunk) => {
                        if sender.send(Ok(chunk)).await.is_err() {
                            // Consumer dropped the stream; abort early.
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err)).await;
                        break;
                    }
                }
            }
            let _ = Self::lo_close(&mut *conn, fd).await;
            let _ = query("COMMIT").execute(&mut *conn).await;
        });

        let stream: BoxStream<'static, Result<Vec<u8>, LsFileStoreError>> =
            Box::pin(futures::stream::unfold(receiver, |mut rx| async move { rx.recv().await.map(|item| (item, rx)) }));
        Ok(BinaryContent::Stream { stream: Mutex::new(stream) })
    }

    /// Streams the source into a freshly created Large Object.
    async fn save_file<'a>(
        &self,
        tx: &mut PgConnection,
        repository_name: &str,
        file_path: &str,
        content: &'a BinaryContent<'a>,
    ) -> Result<u64, LsFileStoreError> {
        let oid: Oid = query("SELECT lo_create(0)").fetch_one(&mut *tx).await?.try_get(0)?;
        let fd: i32 =
            query("SELECT lo_open($1, $2)").bind(oid).bind(LO_INV_WRITE).fetch_one(&mut *tx).await?.try_get(0)?;

        match content {
            BinaryContent::InMemory { content } => {
                for chunk in content.chunks(LO_CHUNK_SIZE) {
                    query("SELECT lowrite($1, $2)").bind(fd).bind(chunk).execute(&mut *tx).await?;
                }
            }
            BinaryContent::OpenDal { operator, path } => {
                let reader = operator.reader_with(path).chunk(LO_CHUNK_SIZE).await.map_err(|err| {
                    LsFileStoreError::OpenDalError {
                        message: format!("PgFileStoreBinaryRepository - Cannot open reader for [{path}]: {err:?}"),
                    }
                })?;
                let mut stream = reader.into_bytes_stream(..).await.map_err(|err| LsFileStoreError::OpenDalError {
                    message: format!("PgFileStoreBinaryRepository - Cannot stream [{path}]: {err:?}"),
                })?;
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result.map_err(|err| LsFileStoreError::OpenDalError {
                        message: format!("PgFileStoreBinaryRepository - Stream error on [{path}]: {err:?}"),
                    })?;
                    query("SELECT lowrite($1, $2)").bind(fd).bind(chunk.as_ref()).execute(&mut *tx).await?;
                }
            }
            BinaryContent::Stream { stream } => {
                let mut guard = stream.lock().await;
                while let Some(chunk) = guard.next().await {
                    let chunk = chunk?;
                    query("SELECT lowrite($1, $2)").bind(fd).bind(&chunk).execute(&mut *tx).await?;
                }
            }
        };

        Self::lo_close(&mut *tx, fd).await?;

        let insert_sql = format!("INSERT INTO {} (repository, filepath, data) VALUES ($1, $2, $3)", self.table_name);
        let res =
            query(AssertSqlSafe(insert_sql)).bind(repository_name).bind(file_path).bind(oid).execute(&mut *tx).await?;
        Ok(res.rows_affected())
    }

    /// Deletes the row and unlinks the underlying LO in the same transaction.
    async fn delete_file(
        &self,
        tx: &mut PgConnection,
        repository_name: &str,
        file_path: &str,
    ) -> Result<u64, LsFileStoreError> {
        let delete_sql =
            format!("DELETE FROM {} WHERE repository = $1 AND filepath = $2 RETURNING data", self.table_name);
        let row =
            query(AssertSqlSafe(delete_sql)).bind(repository_name).bind(file_path).fetch_optional(&mut *tx).await?;

        let Some(row) = row else { return Ok(0) };
        let oid: Oid = row.try_get(0)?;
        // This drops the Large Object.
        query("SELECT lo_unlink($1)").bind(oid).execute(&mut *tx).await?;
        Ok(1)
    }
}

impl PgFileStoreBinaryRepository {
    async fn lookup_oid<'c, E>(
        executor: E,
        table_name: &str,
        repository_name: &str,
        file_path: &str,
    ) -> Result<Oid, LsFileStoreError>
    where
        E: sqlx::Executor<'c, Database = Postgres>,
    {
        let sql = format!("SELECT data FROM {table_name} WHERE repository = $1 AND filepath = $2");
        let row = query(AssertSqlSafe(sql)).bind(repository_name).bind(file_path).fetch_one(executor).await?;
        Ok(row.try_get(0)?)
    }

    async fn lo_open_read<'c, E>(executor: E, oid: Oid) -> Result<i32, LsFileStoreError>
    where
        E: sqlx::Executor<'c, Database = Postgres>,
    {
        let row = query("SELECT lo_open($1, $2)").bind(oid).bind(LO_INV_READ).fetch_one(executor).await?;
        Ok(row.try_get(0)?)
    }

    async fn loread_chunk<'c, E>(executor: E, fd: i32) -> Result<Vec<u8>, LsFileStoreError>
    where
        E: sqlx::Executor<'c, Database = Postgres>,
    {
        let row = query("SELECT loread($1, $2)").bind(fd).bind(LO_CHUNK_SIZE as i32).fetch_one(executor).await?;
        Ok(row.try_get(0)?)
    }

    async fn lo_close<'c, E>(executor: E, fd: i32) -> Result<(), LsFileStoreError>
    where
        E: sqlx::Executor<'c, Database = Postgres>,
    {
        query("SELECT lo_close($1)").bind(fd).execute(executor).await?;
        Ok(())
    }
}
