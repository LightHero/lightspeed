use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::*;
use lightspeed_core::error::LsError;

#[cfg(feature = "postgres")]
pub mod pg;

pub trait CmsRepositoryManager: Clone + Send + Sync {
    type Tx<'a>: Send + Sync;
    type C3P0: for<'a> C3p0Pool<Tx<'a> = Self::Tx<'a>>;
    type ContentRepo: for<'a> ContentRepository<Tx<'a> = Self::Tx<'a>>;
    type ProjectRepo: for<'a> ProjectRepository<Tx<'a> = Self::Tx<'a>>;
    type SchemaRepo: for<'a> SchemaRepository<Tx<'a> = Self::Tx<'a>>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;

    fn content_repo(&self, qualified_table_name: &str) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

pub trait ProjectRepository: Clone + Send + Sync {
     type Tx<'a>: Send + Sync;

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<ProjectModel, LsError>;

    async fn exists_by_name(&self, tx: &mut Self::Tx<'_>, name: &str) -> Result<bool, LsError>;

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<ProjectData>) -> Result<ProjectModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx<'_>, model: ProjectModel) -> Result<ProjectModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: ProjectModel) -> Result<ProjectModel, LsError>;
}

pub trait SchemaRepository: Clone + Send + Sync {
     type Tx<'a>: Send + Sync;

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<SchemaModel, LsError>;

    async fn exists_by_name_and_project_id(
        &self,
        tx: &mut Self::Tx<'_>,
        name: &str,
        project_id: u64,
    ) -> Result<bool, LsError>;

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<SchemaData>) -> Result<SchemaModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx<'_>, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete_by_project_id(&self, tx: &mut Self::Tx<'_>, project_id: u64) -> Result<u64, LsError>;
}

pub trait ContentRepository: Clone + Send + Sync {
   type Tx<'a>: Send + Sync;

    async fn create_table(&self, tx: &mut Self::Tx<'_>) -> Result<(), LsError>;

    async fn drop_table(&self, tx: &mut Self::Tx<'_>) -> Result<(), LsError>;

    async fn count_all(&self, tx: &mut Self::Tx<'_>) -> Result<u64, LsError>;

    async fn count_all_by_field_value(
        &self,
        tx: &mut Self::Tx<'_>,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError>;

    async fn create_unique_constraint(
        &self,
        tx: &mut Self::Tx<'_>,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LsError>;

    async fn drop_unique_constraint(&self, tx: &mut Self::Tx<'_>, index_name: &str) -> Result<(), LsError>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx<'_>, id: u64) -> Result<ContentModel, LsError>;

    async fn save(&self, tx: &mut Self::Tx<'_>, model: NewModel<ContentData>) -> Result<ContentModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx<'_>, model: ContentModel) -> Result<ContentModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx<'_>, model: ContentModel) -> Result<ContentModel, LsError>;
}
