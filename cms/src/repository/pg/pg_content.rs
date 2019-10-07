use crate::model::content::ContentData;
use crate::repository::ContentRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgContentRepository {
    repo: PgC3p0Json<ContentData, DefaultJsonCodec>,
}

impl PgContentRepository {
    pub fn new(table_name: &str) -> Self {
        Self{
            repo: C3p0JsonBuilder::new(table_name).build(),
        }
    }
}

impl Deref for PgContentRepository {
    type Target = PgC3p0Json<ContentData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl ContentRepository for PgContentRepository {
    type CONN = PgConnection;

    fn create_table(&self, conn: &Self::CONN) -> Result<(), LightSpeedError> {
        Ok(self.repo.create_table_if_not_exists(conn)?)
    }

    fn drop_table(&self, conn: &Self::CONN) -> Result<(), LightSpeedError> {
        Ok(self.repo.drop_table_if_exists(conn)?)
    }

    fn crate_unique_constraint(&self, conn: &Self::CONN, index_name: &str, field_name: &str) -> Result<(), LightSpeedError> {
        Ok(conn.batch_execute(&format!("CREATE UNIQUE INDEX {} ON {}( (DATA->>'name') )", index_name, self.repo.queries().qualified_table_name))?)
    }

    fn drop_unique_constraint(&self, conn: &Self::CONN, field_name: &str) -> Result<(), LightSpeedError> {
        unimplemented!()
    }

    fn fetch_by_id(&self, conn: &Self::CONN, id: i64) -> Result<Model<ContentData>, LightSpeedError> {
        unimplemented!()
    }

    fn save(&self, conn: &Self::CONN, model: NewModel<ContentData>) -> Result<Model<ContentData>, LightSpeedError> {
        unimplemented!()
    }

    fn update(&self, conn: &Self::CONN, model: Model<ContentData>) -> Result<Model<ContentData>, LightSpeedError> {
        unimplemented!()
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<ContentData>) -> Result<u64, LightSpeedError> {
        unimplemented!()
    }
}
