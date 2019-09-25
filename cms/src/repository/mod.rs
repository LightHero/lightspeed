use c3p0::{C3p0Pool, Connection, NewModel};
use lightspeed_core::error::LightSpeedError;
use crate::model::project::{ProjectModel, ProjectData};
use crate::model::schema::{SchemaModel, SchemaData};

pub mod pg;

pub trait CmsRepositoryManager: Clone {
    type CONN: Connection;
    type C3P0: C3p0Pool<CONN = Self::CONN>;
    type PROJECT_REPO: ProjectRepository<CONN = Self::CONN>;
    type SCHEMA_REPO: SchemaRepository<CONN = Self::CONN>;

    fn c3p0(&self) -> &Self::C3P0;
    fn start(&self) -> Result<(), LightSpeedError>;

    fn project_repo(&self) -> Self::PROJECT_REPO;
    fn schema_repo(&self) -> Self::SCHEMA_REPO;

}

pub trait ProjectRepository: Clone {
    type CONN: Connection;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        id: i64,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn exists_by_name(
        &self,
        conn: &Self::CONN,
        name: &str,
    ) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<ProjectData>,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn update(
        &self,
        conn: &Self::CONN,
        model: ProjectModel,
    ) -> Result<ProjectModel, LightSpeedError>;

    fn delete(&self, conn: &Self::CONN, model: &ProjectModel) -> Result<u64, LightSpeedError>;
}

pub trait SchemaRepository: Clone {
    type CONN: Connection;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        id: i64,
    ) -> Result<SchemaModel, LightSpeedError>;

    fn exists_by_name_and_project_id(
        &self,
        conn: &Self::CONN,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LightSpeedError>;

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<SchemaData>,
    ) -> Result<SchemaModel, LightSpeedError>;

    fn update(
        &self,
        conn: &Self::CONN,
        model: SchemaModel,
    ) -> Result<SchemaModel, LightSpeedError>;

    fn delete(&self, conn: &Self::CONN, model: &SchemaModel) -> Result<u64, LightSpeedError>;
}
