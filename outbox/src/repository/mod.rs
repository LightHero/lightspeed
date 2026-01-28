use std::future::Future;

use crate::model::task::{TaskData, TaskModel};
use c3p0::sqlx::Database;
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait OutboxRepositoryManager: Clone + Send + Sync {
    type DB: Database;
    type C3P0: C3p0Pool<DB = Self::DB>;
    type TaskRepo: for<'a> TaskRepository<DB = Self::DB>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl Future<Output = Result<(), LsError>> + Send;
    fn task_repo(&self) -> Self::TaskRepo;
}

pub trait TaskRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_by_token(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        token_string: &str,
    ) -> impl Future<Output = Result<TaskModel, LsError>> + Send;

    fn fetch_by_username(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        username: &str,
    ) -> impl Future<Output = Result<Vec<TaskModel>, LsError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<TaskData>,
    ) -> impl Future<Output = Result<TaskModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: TaskModel,
    ) -> impl Future<Output = Result<TaskModel, LsError>> + Send;
}
