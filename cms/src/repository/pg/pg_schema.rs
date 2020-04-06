use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use c3p0::pg_async::*;
use c3p0::*;
use lightspeed_core::error::LightSpeedError;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: PgC3p0JsonAsync<SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = PgC3p0JsonAsync<SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository {
            repo: C3p0JsonBuilder::new("LS_CMS_SCHEMA").build(),
        }
    }
}

#[async_trait::async_trait]
impl SchemaRepository for PgSchemaRepository {
    type Conn = PgConnectionAsync;

    async fn fetch_by_id(
        &self,
        conn: &mut Self::Conn,
        id: i64,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn exists_by_name_and_project_id(
        &self,
        conn: &mut Self::Conn,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LightSpeedError> {
        let sql = r#"
            select count(*) from LS_CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2
        "#;
        Ok(conn.fetch_one(sql, &[&name, &project_id], |row| {
            let count: i64 = row.get(0);
            Ok(count > 0)
        }).await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<SchemaData>,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<SchemaData>,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<SchemaData>,
    ) -> Result<Model<SchemaData>, LightSpeedError> {
        Ok(self.repo.delete(conn, model).await?)
    }

    async fn delete_by_project_id(
        &self,
        conn: &mut Self::Conn,
        project_id: i64,
    ) -> Result<u64, LightSpeedError> {
        let sql = r#"
            delete from LS_CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;
        Ok(conn.execute(sql, &[&project_id]).await?)
    }
}
