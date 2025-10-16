use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::*;
use lightspeed_core::error::LsError;
use ::sqlx::Database;

#[cfg(feature = "postgres")]
pub mod postgres;

pub trait CmsRepositoryManager: Clone + Send + Sync {
    type DB: Database;
    type C3P0: C3p0Pool<DB = Self::DB>;
    type ContentRepo: ContentRepository<DB = Self::DB>;
    type ProjectRepo: ProjectRepository<DB = Self::DB>;
    type SchemaRepo: SchemaRepository<DB = Self::DB>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn content_repo(&self) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

pub trait ProjectRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: u64,
    ) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn exists_by_name(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        name: &str,
    ) -> impl std::future::Future<Output = Result<bool, LsError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<ProjectData>,
    ) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn update(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: ProjectModel,
    ) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: ProjectModel,
    ) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;
}

pub trait SchemaRepository: Clone + Send + Sync {
    type DB: Database;

    fn fetch_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: u64,
    ) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn exists_by_name_and_project_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        name: &str,
        project_id: u64,
    ) -> impl std::future::Future<Output = Result<bool, LsError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<SchemaData>,
    ) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn update(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: SchemaModel,
    ) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: SchemaModel,
    ) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn delete_by_project_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        project_id: u64,
    ) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;
}

pub trait ContentRepository: Clone + Send + Sync {
    type DB: Database;

    fn count_all(&self, tx: &mut <Self::DB as Database>::Connection) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;

    fn count_all_by_field_value(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        field_name: &str,
        field_value: &str,
    ) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;

    fn fetch_by_id(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        id: u64,
    ) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn save(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: NewRecord<ContentData>,
    ) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn update(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: ContentModel,
    ) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn delete(
        &self,
        tx: &mut <Self::DB as Database>::Connection,
        model: ContentModel,
    ) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;
}
