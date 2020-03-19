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
        Self {
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

    fn create_table(&self, conn: &mut Self::CONN) -> Result<(), LightSpeedError> {
        Ok(self.repo.create_table_if_not_exists(conn)?)
    }

    fn drop_table(&self, conn: &mut Self::CONN) -> Result<(), LightSpeedError> {
        Ok(self.repo.drop_table_if_exists(conn, true)?)
    }

    fn count_all(&self, conn: &mut Self::CONN) -> Result<u64, LightSpeedError> {
        Ok(self.repo.count_all(conn)?)
    }

    fn count_all_by_field_value(&self, conn: &mut Self::CONN, field_name: &str, field_value: &str) -> Result<u64, LightSpeedError> {
        Ok(conn.fetch_one_value(&format!(
            "SELECT COUNT(*) FROM {} WHERE  (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') = $1 ",
            self.repo.queries().qualified_table_name,
            field_name
        ), &[&field_value]).map(|val: i64| val as u64)?)
    }

    fn create_unique_constraint(
        &self,
        conn: &mut Self::CONN,
        index_name: &str,
        field_name: &str,
    ) -> Result<(), LightSpeedError> {
        Ok(conn.batch_execute(&format!(
            "CREATE UNIQUE INDEX {} ON {}( (DATA -> 'content' -> 'fields' -> '{}' -> 'value' ->> 'value') )",
            index_name,
            self.repo.queries().qualified_table_name,
            field_name
        ))?)
    }

    fn drop_unique_constraint(
        &self,
        conn: &mut Self::CONN,
        index_name: &str,
    ) -> Result<(), LightSpeedError> {
        Ok(conn.batch_execute(&format!("DROP INDEX {} IF EXISTS", index_name))?)
    }

    fn fetch_by_id(
        &self,
        conn: &mut Self::CONN,
        id: i64,
    ) -> Result<Model<ContentData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!(
                    "No content found with id [{}] in [{}]",
                    id,
                    self.repo.queries().qualified_table_name
                ),
            })
    }

    fn save(
        &self,
        conn: &mut Self::CONN,
        model: NewModel<ContentData>,
    ) -> Result<Model<ContentData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn update(
        &self,
        conn: &mut Self::CONN,
        model: Model<ContentData>,
    ) -> Result<Model<ContentData>, LightSpeedError> {
        Ok(self.repo.update(conn, model)?)
    }

    fn delete(
        &self,
        conn: &mut Self::CONN,
        model: &Model<ContentData>,
    ) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }
}
