use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait CmsRepositoryManager: Clone + Send + Sync {
    type Tx: SqlTx;
    type C3P0: C3p0Pool<Tx = Self::Tx>;
    type ContentRepo: ContentRepository<Tx = Self::Tx>;
    type ProjectRepo: ProjectRepository<Tx = Self::Tx>;
    type SchemaRepo: SchemaRepository<Tx = Self::Tx>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;

    fn content_repo(&self, qualified_table_name: &str) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

#[async_trait::async_trait]
pub trait ProjectRepository: Clone + Send + Sync {
    type Tx;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<ProjectModel, LsError>;

    async fn exists_by_name(&self, tx: &mut Self::Tx, name: &str) -> Result<bool, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<ProjectData>) -> Result<ProjectModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx, model: ProjectModel) -> Result<ProjectModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: ProjectModel) -> Result<ProjectModel, LsError>;
}

#[async_trait::async_trait]
pub trait SchemaRepository: Clone + Send + Sync {
    type Tx;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<SchemaModel, LsError>;

    async fn exists_by_name_and_project_id(
        &self,
        tx: &mut Self::Tx,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<SchemaData>) -> Result<SchemaModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete_by_project_id(&self, tx: &mut Self::Tx, project_id: i64) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait ContentRepository: Clone + Send + Sync {
    type Tx;

    async fn create_table(&self, tx: &mut Self::Tx) -> Result<(), LsError>;

    async fn drop_table(&self, tx: &mut Self::Tx) -> Result<(), LsError>;

    async fn count_all(&self, tx: &mut Self::Tx) -> Result<u64, LsError>;

    async fn count_all_by_field_value(
        &self,
        tx: &mut Self::Tx,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError>;

    async fn create_unique_constraint(
        &self,
        tx: &mut Self::Tx,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LsError>;

    async fn drop_unique_constraint(&self, tx: &mut Self::Tx, index_name: &str) -> Result<(), LsError>;

    async fn fetch_by_id(&self, tx: &mut Self::Tx, id: i64) -> Result<ContentModel, LsError>;

    async fn save(&self, tx: &mut Self::Tx, model: NewModel<ContentData>) -> Result<ContentModel, LsError>;

    async fn update(&self, tx: &mut Self::Tx, model: ContentModel) -> Result<ContentModel, LsError>;

    async fn delete(&self, tx: &mut Self::Tx, model: ContentModel) -> Result<ContentModel, LsError>;
}
