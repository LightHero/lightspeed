use crate::repository::ProjectRepository;
use c3p0::pg::*;
use c3p0::*;
use crate::model::project::ProjectData;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgProjectRepository {
    repo: C3p0JsonPg<ProjectData, DefaultJsonCodec>,
}

impl Deref for PgProjectRepository {
    type Target = C3p0JsonPg<ProjectData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgProjectRepository {
    fn default() -> Self {
        PgProjectRepository {
            repo: C3p0JsonBuilder::new("CMS_PROJECT")
                .with_data_field_name("data_json")
                .build(),
        }
    }
}

impl ProjectRepository for PgProjectRepository {
    type CONN = PgConnection;
}