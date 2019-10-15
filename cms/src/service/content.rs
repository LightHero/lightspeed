use crate::dto::create_content_dto::CreateContentDto;
use crate::model::content::{ContentModel, ContentFieldValue, ContentFieldValueArity};
use crate::model::schema::{Schema, SchemaModel, SchemaFieldArity};
use crate::repository::CmsRepositoryManager;
use crate::repository::ContentRepository;
use c3p0::*;
use chashmap::{CHashMap, ReadGuard};
use lightspeed_core::error::{ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::Validator;

#[derive(Clone)]
pub struct ContentService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    repo_factory: RepoManager,
    content_repos: CHashMap<i64, RepoManager::CONTENT_REPO>,
}

impl<RepoManager: CmsRepositoryManager> ContentService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, repo_factory: RepoManager) -> Self {
        ContentService {
            c3p0,
            repo_factory,
            content_repos: CHashMap::new(),
        }
    }

    pub fn create_content_table(&self, schema: &SchemaModel) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let schema_id = schema.id;
            let repo = self.get_content_repo_by_schema_id(schema_id);
            repo.create_table(conn)?;

            for field in &schema.data.schema.fields {
                match field.field_type.get_arity() {
                    SchemaFieldArity::Unique => {
                        let index_name = self.unique_index_name(schema_id, &field.name);
                        repo.create_unique_constraint(conn, &index_name, &field.name)?;
                    },
                    _ => {}
                }
            }
            Ok(())
        })

    }

    pub fn drop_content_table(&self, schema_id: i64) -> Result<(), LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            repo.drop_table(conn)
        })
    }

    pub fn count_all_by_schema_id(&self, schema_id: i64) -> Result<u64, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(schema_id);
            repo.count_all(conn)
        })
    }

    pub fn create_content(
        &self,
        schema: &Schema,
        create_content_dto: CreateContentDto,
    ) -> Result<ContentModel, LightSpeedError> {
        self.c3p0.transaction(move |conn| {

            let repo = self.get_content_repo_by_schema_id(create_content_dto.schema_id);

                            // TODO: refactor and implement
            for field in &schema.fields {
                match field.field_type.get_arity() {
                    SchemaFieldArity::Unique => {
                        if let Some(content_field) = create_content_dto.content.fields.get(&field.name) {

                            let field_value = match content_field {
                                ContentFieldValue::Slug {value} |ContentFieldValue::String {value} => {
                                    match value {
                                        ContentFieldValueArity::Single{value: Some(field_value)} => Some(field_value.to_string()),
                                        _ => None
                                    }
                                },
                                ContentFieldValue::Boolean {value} => {
                                    match value {
                                        ContentFieldValueArity::Single{value: Some(field_value)} => Some(field_value.to_string()),
                                        _ => None
                                    }
                                },
                                ContentFieldValue::Number {value} => {
                                    match value {
                                        ContentFieldValueArity::Single{value: Some(field_value)} => Some(field_value.to_string()),
                                        _ => None
                                    }
                                },
                            };

                            if let Some (value) =  field_value {
                                let count = repo.count_all_by_field_value(conn, &field.name, &value)?;
                                println!("------------------");
                                println!("Found {} entries with field [{}] value [{}]", count, &field.name, value);
                                println!("------------------");
                            }

                        }
                    },
                    _ => {}
                }
            }

            Validator::validate(|error_details: &ErrorDetails| {
                create_content_dto.content.validate(&schema, error_details);
                Ok(())
            })?;
            repo.save(conn, NewModel::new(create_content_dto))
        })
    }

    pub fn delete_content(&self, content_model: ContentModel) -> Result<u64, LightSpeedError> {
        self.c3p0.transaction(move |conn| {
            let repo = self.get_content_repo_by_schema_id(content_model.data.schema_id);
            repo.delete(conn, &content_model)
        })
    }

    // ToDo: remove unwraps!! AND REWRITE...
    fn get_content_repo_by_schema_id(
        &self,
        schema_id: i64,
    ) -> ReadGuard<'_, i64, RepoManager::CONTENT_REPO> {
        let content_repo = self.content_repos.get(&schema_id);
        if content_repo.is_none() {
            {
                self.content_repos.insert(
                    schema_id,
                    self.repo_factory
                        .content_repo(&self.content_table_name(schema_id)),
                );
            }
            self.content_repos.get(&schema_id).unwrap()
        } else {
            content_repo.unwrap()
        }
    }

    fn content_table_name(&self, schema_id: i64) -> String {
        format!("CMS_CONTENT_{}", schema_id)
    }
    fn unique_index_name(&self, schema_id: i64, field_name: &str) -> String {
        format!("CMS_CONTENT_{}_UNIQUE_{}", schema_id, field_name)
    }
}

#[cfg(test)]
mod test {}
