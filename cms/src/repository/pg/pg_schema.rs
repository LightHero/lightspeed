use crate::model::schema::SchemaData;
use crate::repository::SchemaRepository;
use c3p0::sqlx::{*, error::into_c3p0_error};
use c3p0::*;
use lightspeed_core::error::LsError;
use ::sqlx::Row;
use std::ops::Deref;

#[derive(Clone)]
pub struct PgSchemaRepository {
    repo: SqlxPgC3p0Json<SchemaData, DefaultJsonCodec>,
}

impl Deref for PgSchemaRepository {
    type Target = SqlxPgC3p0Json<SchemaData, DefaultJsonCodec>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl Default for PgSchemaRepository {
    fn default() -> Self {
        PgSchemaRepository { repo: C3p0JsonBuilder::new("LS_CMS_SCHEMA").build() }
    }
}

#[async_trait::async_trait]
impl SchemaRepository for PgSchemaRepository {
    type Conn = SqlxPgConnection;

    async fn fetch_by_id(&self, conn: &mut Self::Conn, id: i64) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.fetch_one_by_id(conn, &id).await?)
    }

    async fn exists_by_name_and_project_id(
        &self,
        conn: &mut Self::Conn,
        name: &str,
        project_id: i64,
    ) -> Result<bool, LsError> {
        let sql = r#"
        SELECT EXISTS (SELECT 1 from LS_CMS_SCHEMA
            where DATA ->> 'name' = $1 AND (DATA ->> 'project_id')::bigint = $2 )
        "#;

        let res = ::sqlx::query(sql).bind(name).bind(project_id)
        .fetch_one(&mut **conn.get_conn())
        .await
        .and_then(|row| {row.try_get(0)        })
        .map_err(into_c3p0_error)?;
    Ok(res)

        // Ok(conn
        //     .fetch_one(sql, &[&name, &project_id], |row| {
        //         let count: i64 = row.get(0);
        //         Ok(count > 0)
        //     })
        //     .await?)
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        model: NewModel<SchemaData>,
    ) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.save(conn, model).await?)
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        model: Model<SchemaData>,
    ) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.update(conn, model).await?)
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        model: Model<SchemaData>,
    ) -> Result<Model<SchemaData>, LsError> {
        Ok(self.repo.delete(conn, model).await?)
    }

    async fn delete_by_project_id(&self, conn: &mut Self::Conn, project_id: i64) -> Result<u64, LsError> {
        let sql = r#"
            delete from LS_CMS_SCHEMA
            where (DATA ->> 'project_id')::bigint = $1
        "#;

        let res = ::sqlx::query(sql).bind(project_id)
        .execute(&mut **conn.get_conn())
        .await
        .map_err(into_c3p0_error)?;
    Ok(res.rows_affected())

        // Ok(conn.execute(sql, &[&project_id]).await?)
    }
}
