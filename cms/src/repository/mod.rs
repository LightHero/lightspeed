use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "postgres")]
pub mod postgres;

pub trait CmsRepositoryManager: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;
    type C3P0: for<'a> C3p0Pool<Tx<'a> = Self::Tx<'a>>;
    type ContentRepo: for<'a> ContentRepository<Tx<'a> = Self::Tx<'a>>;
    type ProjectRepo: for<'a> ProjectRepository<Tx<'a> = Self::Tx<'a>>;
    type SchemaRepo: for<'a> SchemaRepository<Tx<'a> = Self::Tx<'a>>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn content_repo(&self, qualified_table_name: &str) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

pub trait ProjectRepository: Clone + Send + Sync {
     type Tx<'a>: Send + Sync;

    fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn exists_by_name(&self, tx: &mut Self::Tx<'_>, name: &str) -> impl std::future::Future<Output = Result<bool, LsError>> + Send;

    fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<ProjectData>) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn update(&self, tx: &mut Self::Tx<'_>, model: ProjectModel) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;

    fn delete(&self, tx: &mut Self::Tx<'_>, model: ProjectModel) -> impl std::future::Future<Output = Result<ProjectModel, LsError>> + Send;
}

pub trait SchemaRepository: Clone + Send + Sync {
     type Tx<'a>: Send + Sync;

    fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn exists_by_name_and_project_id(
        &self,
        tx: &mut Self::Tx<'_>,
        name: &str,
        project_id: u64,
    ) -> impl std::future::Future<Output = Result<bool, LsError>> + Send;

    fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<SchemaData>) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn update(&self, tx: &mut Self::Tx<'_>, model: SchemaModel) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn delete(&self, tx: &mut Self::Tx<'_>, model: SchemaModel) -> impl std::future::Future<Output = Result<SchemaModel, LsError>> + Send;

    fn delete_by_project_id(&self, tx: &mut Self::Tx<'_>, project_id: u64) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;
}

pub trait ContentRepository: Clone + Send + Sync {
   type Tx<'a>: Send + Sync;

    fn create_table(&self, tx: &mut Self::Tx<'_>) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn drop_table(&self, tx: &mut Self::Tx<'_>) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn count_all(&self, tx: &mut Self::Tx<'_>) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;

    fn count_all_by_field_value(
        &self,
        tx: &mut Self::Tx<'_>,
        field_name: &str,
        field_value: &str,
    ) -> impl std::future::Future<Output = Result<u64, LsError>> + Send;

    fn create_unique_constraint(
        &self,
        tx: &mut Self::Tx<'_>,
        index_name: &str,
        field_name: &str,
    ) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn drop_unique_constraint(&self, tx: &mut Self::Tx<'_>, index_name: &str) -> impl std::future::Future<Output = Result<(), LsError>> + Send;

    fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<ContentData>) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn update(&self, tx: &mut Self::Tx<'_>, model: ContentModel) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;

    fn delete(&self, tx: &mut Self::Tx<'_>, model: ContentModel) -> impl std::future::Future<Output = Result<ContentModel, LsError>> + Send;
}
