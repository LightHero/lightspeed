use crate::model::content::{ContentData, ContentModel};
use crate::model::project::{ProjectData, ProjectModel};
use crate::model::schema::{SchemaData, SchemaModel};
use c3p0::{C3p0Pool, NewModel};
use lightspeed_core::error::LightSpeedError;

pub mod pg;

pub trait CmsRepositoryManager: Clone {
    type CONN;
    type C3P0: C3p0Pool<CONN = Self::CONN>;
    type CONTENT_REPO: ContentRepository<CONN = Self::CONN>;
    type PROJECT_REPO: ProjectRepository<CONN = Self::CONN>;
    type SCHEMA_REPO: SchemaRepository<CONN = Self::CONN>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;

    fn content_repo(&self, qualified_table_name: &str) -> Self::CONTENT_REPO;
    fn project_repo(&self) -> Self::PROJECT_REPO;
    fn schema_repo(&self) -> Self::SCHEMA_REPO;
}

pub trait ProjectRepository: Clone {
    type CONN;

    fn fetch_by_id(&self, conn: &mut Self::CONN, id: i64) -> Result<ProjectModel, LightSpeedError>;

    fn exists_by_name(&self, conn: &mut Self::CONN, name: &str) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::CONN,
        model: NewModel<ProjectData>,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn update(
        &self,
        conn: &mut Self::CONN,
        model: ProjectModel,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::CONN, model: &ProjectModel) -> Result<u64, LightSpeedError>;
}

pub trait SchemaRepository: Clone {
    type CONN;

    fn fetch_by_id(&self, conn: &mut Self::CONN, id: i64) -> Result<SchemaModel, LightSpeedError>;

    fn exists_by_name_and_project_id(
        &self,
        conn: &mut Self::CONN,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::CONN,
        model: NewModel<SchemaData>,
    ) -> Result<SchemaModel, LightSpeedError>;

    fn update(&self, conn: &mut Self::CONN, model: SchemaModel)
        -> Result<SchemaModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::CONN, model: &SchemaModel) -> Result<u64, LightSpeedError>;

    fn delete_by_project_id(
        &self,
        conn: &mut Self::CONN,
        project_id: i64,
    ) -> Result<u64, LightSpeedError>;
}

pub trait ContentRepository: Clone {
    type CONN;

    fn create_table(&self, conn: &mut Self::CONN) -> Result<(), LightSpeedError>;

    fn drop_table(&self, conn: &mut Self::CONN) -> Result<(), LightSpeedError>;

    fn count_all(&self, conn: &mut Self::CONN) -> Result<u64, LightSpeedError>;

    fn count_all_by_field_value(&self, conn: &mut Self::CONN, field_name: &str, field_value: &str) -> Result<u64, LightSpeedError>;

    fn create_unique_constraint(
        &self,
        conn: &mut Self::CONN,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LightSpeedError>;

    fn drop_unique_constraint(
        &self,
        conn: &mut Self::CONN,
        index_name: &str,
    ) -> Result<(), LightSpeedError>;

    fn fetch_by_id(&self, conn: &mut Self::CONN, id: i64) -> Result<ContentModel, LightSpeedError>;

    fn save(
        &self,
        conn: &mut Self::CONN,
        model: NewModel<ContentData>,
    ) -> Result<ContentModel, LightSpeedError>;

    fn update(
        &self,
        conn: &mut Self::CONN,
        model: ContentModel,
    ) -> Result<ContentModel, LightSpeedError>;

    fn delete(&self, conn: &mut Self::CONN, model: &ContentModel) -> Result<u64, LightSpeedError>;
}
