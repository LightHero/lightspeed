use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::*;
use lightspeed_core::error::LsError;

pub mod pg;

#[async_trait::async_trait]
pub trait CmsRepositoryManager: Clone + Send + Sync {
    type Conn: SqlConnection;
    type C3P0: C3p0Pool<Conn = Self::Conn>;
    type ContentRepo: ContentRepository<Conn = Self::Conn>;
    type ProjectRepo: ProjectRepository<Conn = Self::Conn>;
    type SchemaRepo: SchemaRepository<Conn = Self::Conn>;

    fn c3p0(&self) -> &Self::C3P0;
    async fn start(&self) -> Result<(), LsError>;

    fn content_repo(&self, qualified_table_name: &str) -> Self::ContentRepo;
    fn project_repo(&self) -> Self::ProjectRepo;
    fn schema_repo(&self) -> Self::SchemaRepo;
}

#[async_trait::async_trait]
pub trait ProjectRepository: Clone + Send + Sync {
    type Conn;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<ProjectModel, LsError>;

    async fn exists_by_name(&self, conn: &mut Self::Conn, name: &str) -> Result<bool, LsError>;

    async fn save(&self, conn: &mut Self::Conn, model: NewModel<ProjectData>) -> Result<ProjectModel, LsError>;

    async fn update(&self, conn: &mut Self::Conn, model: ProjectModel) -> Result<ProjectModel, LsError>;

    async fn delete(&self, conn: &mut Self::Conn, model: ProjectModel) -> Result<ProjectModel, LsError>;
}

#[async_trait::async_trait]
pub trait SchemaRepository: Clone + Send + Sync {
    type Conn;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<SchemaModel, LsError>;

    async fn exists_by_name_and_project_id(
        &self,
        conn: &mut Self::Conn,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LsError>;

    async fn save(&self, conn: &mut Self::Conn, model: NewModel<SchemaData>) -> Result<SchemaModel, LsError>;

    async fn update(&self, conn: &mut Self::Conn, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete(&self, conn: &mut Self::Conn, model: SchemaModel) -> Result<SchemaModel, LsError>;

    async fn delete_by_project_id(&self, conn: &mut Self::Conn, project_id: i64) -> Result<u64, LsError>;
}

#[async_trait::async_trait]
pub trait ContentRepository: Clone + Send + Sync {
    type Conn;

    async fn create_table(&self, conn: &mut Self::Conn) -> Result<(), LsError>;

    async fn drop_table(&self, conn: &mut Self::Conn) -> Result<(), LsError>;

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, LsError>;

    async fn count_all_by_field_value(
        &self,
        conn: &mut Self::Conn,
        field_name: &str,
        field_value: &str,
    ) -> Result<u64, LsError>;

    async fn create_unique_constraint(
        &self,
        conn: &mut Self::Conn,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LsError>;

    async fn drop_unique_constraint(&self, conn: &mut Self::Conn, index_name: &str) -> Result<(), LsError>;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<ContentModel, LsError>;

    async fn save(&self, conn: &mut Self::Conn, model: NewModel<ContentData>) -> Result<ContentModel, LsError>;

    async fn update(&self, conn: &mut Self::Conn, model: ContentModel) -> Result<ContentModel, LsError>;

    async fn delete(&self, conn: &mut Self::Conn, model: ContentModel) -> Result<ContentModel, LsError>;
}
