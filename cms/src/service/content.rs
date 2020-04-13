use crate::dto::create_content_dto::CreateContentDto;
use crate::model::content::{ContentFieldValue, ContentFieldValueArity, ContentModel};
use crate::model::schema::{Schema, SchemaFieldArity, SchemaModel};
use crate::repository::CmsRepositoryManager;
use crate::repository::ContentRepository;
use c3p0::*;
use lightspeed_cache::Cache;
use lightspeed_core::error::LightSpeedError;
use lightspeed_core::service::validator::{Validator, ERR_NOT_UNIQUE};
use std::sync::Arc;

const CMS_CONTENT_TABLE_PREFIX: &str = "LS_CMS_CONTENT_";

#[derive(Clone)]
pub struct ContentService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    repo_factory: RepoManager,
    content_repos: Cache<i64, RepoManager::ContentRepo>,
}

impl<RepoManager: CmsRepositoryManager> ContentService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, repo_factory: RepoManager) -> Self {
        ContentService {
            c3p0,
            repo_factory,
            content_repos: Cache::new(u32::max_value()),
        }
    }

    pub async fn create_content_table(&self, schema: &SchemaModel) -> Result<(), LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                let schema_id = schema.id;
                let repo = self.get_content_repo_by_schema_id(schema_id).await;
                repo.create_table(&mut conn).await?;

                for field in &schema.data.schema.fields {
                    if let SchemaFieldArity::Unique = field.field_type.get_arity() {
                        let index_name = self.unique_index_name(schema_id, &field.name);
                        repo.create_unique_constraint(&mut conn, &index_name, &field.name)
                            .await?;
                    }
                }
                Ok(())
            })
            .await
    }

    pub async fn drop_content_table(&self, schema_id: i64) -> Result<(), LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                let repo = self.get_content_repo_by_schema_id(schema_id).await;
                repo.drop_table(&mut conn).await
            })
            .await
    }

    pub async fn count_all_by_schema_id(&self, schema_id: i64) -> Result<u64, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                let repo = self.get_content_repo_by_schema_id(schema_id).await;
                repo.count_all(&mut conn).await
            })
            .await
    }

    pub async fn create_content(
        &self,
        schema: &Schema,
        create_content_dto: CreateContentDto,
    ) -> Result<ContentModel, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                let conn = &mut conn;
                let repo = self
                    .get_content_repo_by_schema_id(create_content_dto.schema_id)
                    .await;

                let mut validator = Validator::new();

                create_content_dto
                    .content
                    .validate(&schema, validator.error_details());

                for field in &schema.fields {
                    if let SchemaFieldArity::Unique = field.field_type.get_arity() {
                        if let Some(content_field) =
                            create_content_dto.content.fields.get(&field.name)
                        {
                            let field_value = match content_field {
                                ContentFieldValue::Slug { value }
                                | ContentFieldValue::String { value } => match value {
                                    ContentFieldValueArity::Single {
                                        value: Some(field_value),
                                    } => Some(field_value.to_string()),
                                    _ => None,
                                },
                                ContentFieldValue::Boolean { value } => match value {
                                    ContentFieldValueArity::Single {
                                        value: Some(field_value),
                                    } => Some(field_value.to_string()),
                                    _ => None,
                                },
                                ContentFieldValue::Number { value } => match value {
                                    ContentFieldValueArity::Single {
                                        value: Some(field_value),
                                    } => Some(field_value.to_string()),
                                    _ => None,
                                },
                            };

                            if let Some(value) = field_value {
                                let count = repo
                                    .count_all_by_field_value(conn, &field.name, &value)
                                    .await?;
                                if count > 0 {
                                    let scoped_name = format!("fields[{}]", &field.name);
                                    validator
                                        .error_details()
                                        .add_detail(scoped_name, ERR_NOT_UNIQUE);
                                }
                            }
                        }
                    }
                }

                validator.do_validate()?;

                repo.save(conn, NewModel::new(create_content_dto)).await
            })
            .await
    }

    pub async fn delete_content(
        &self,
        content_model: ContentModel,
    ) -> Result<ContentModel, LightSpeedError> {
        self.c3p0
            .transaction(|mut conn| async move {
                let repo = self
                    .get_content_repo_by_schema_id(content_model.data.schema_id)
                    .await;
                repo.delete(&mut conn, content_model).await
            })
            .await
    }

    async fn get_content_repo_by_schema_id(&self, schema_id: i64) -> Arc<RepoManager::ContentRepo> {
        self.content_repos
            .get_or_insert_with(schema_id, || async {
                self.repo_factory
                    .content_repo(&self.content_table_name(schema_id))
            })
            .await
    }

    fn content_table_name(&self, schema_id: i64) -> String {
        format!("{}{}", CMS_CONTENT_TABLE_PREFIX, schema_id)
    }

    fn unique_index_name(&self, schema_id: i64, field_name: &str) -> String {
        format!(
            "{}{}_UNIQUE_{}",
            CMS_CONTENT_TABLE_PREFIX, schema_id, field_name
        )
    }
}

#[cfg(test)]
mod test {}
