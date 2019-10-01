use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use c3p0::pg::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: C3p0JsonPg<SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = C3p0JsonPg<SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository {
            repo: C3p0JsonBuilder::new("CMS_SCHEMA").build(),
        }
    }
}

impl SchemaRepository for PgSchemaRepository {
    type CONN = PgConnection;

    fn fetch_by_id(
        &self,
        conn: &Self::CONN,
        id: i64,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        self.repo
            .fetch_one_by_id(conn, &id)?
            .ok_or_else(|| LightSpeedError::BadRequest {
                message: format!("No Schema found with id [{}]", id),
            })
    }

    fn exists_by_name_and_project_id(
        &self,
        conn: &Self::CONN,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LightSpeedError> {
        let sql = r#"
            select count(*) from CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2
        "#;
        Ok(conn.fetch_one(sql, &[&name, &project_id], |row| {
            let count: i64 = row.get(0);
            Ok(count > 0)
        })?)
    }

    fn save(
        &self,
        conn: &Self::CONN,
        model: NewModel<SchemaData>,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.save(conn, model)?)
    }

    fn update(
        &self,
        conn: &Self::CONN,
        model: Model<SchemaData>,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.update(conn, model)?)
    }

    fn delete(&self, conn: &Self::CONN, model: &Model<SchemaData>) -> Result<u64, LightSpeedError> {
        Ok(self.repo.delete(conn, model)?)
    }

    fn delete_by_project_id(
        &self,
        conn: &Self::CONN,
        project_id: i64,
    ) -> Result<u64, LightSpeedError> {
        let sql = r#"
            delete from CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;
        Ok(conn.execute(sql, &[&project_id])?)
    }
}
