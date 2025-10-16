use crate::dto::create_content_dto::CreateContentDto;
use crate::model::content::{ContentFieldValue, ContentFieldValueArity, ContentModel};
use crate::model::schema::{Schema, SchemaFieldArity};
use crate::repository::CmsRepositoryManager;
use crate::repository::ContentRepository;
use c3p0::*;
use lightspeed_core::error::LsError;
use lightspeed_core::service::validator::{ERR_NOT_UNIQUE, Validator};

#[derive(Clone)]
pub struct LsContentService<RepoManager: CmsRepositoryManager> {
    c3p0: RepoManager::C3P0,
    content_repo: RepoManager::ContentRepo,
}

impl<RepoManager: CmsRepositoryManager> LsContentService<RepoManager> {
    pub fn new(c3p0: RepoManager::C3P0, content_repo: RepoManager::ContentRepo) -> Self {
        LsContentService { c3p0, content_repo }
    }

    pub async fn count_all_by_schema_id(&self, schema_id: u64) -> Result<u64, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.content_repo.count_all_by_schema(conn, schema_id).await
            })
            .await
    }

    pub async fn create_content(
        &self,
        schema: &Schema,
        create_content_dto: CreateContentDto,
    ) -> Result<ContentModel, LsError> {
        self.c3p0
            .transaction(async |conn| {
                let mut validator = Validator::new();

                create_content_dto.content.validate(schema, validator.error_details());

                for field in &schema.fields {
                    if let SchemaFieldArity::Unique = field.field_type.get_arity() {
                        if let Some(content_field) = create_content_dto.content.fields.get(&field.name) {
                            let field_value = match content_field {
                                ContentFieldValue::Slug { value } | ContentFieldValue::String { value } => {
                                    match value {
                                        ContentFieldValueArity::Single { value: Some(field_value) } => {
                                            Some(field_value.to_string())
                                        }
                                        _ => None,
                                    }
                                }
                                ContentFieldValue::Boolean { value } => match value {
                                    ContentFieldValueArity::Single { value: Some(field_value) } => {
                                        Some(field_value.to_string())
                                    }
                                    _ => None,
                                },
                                ContentFieldValue::Number { value } => match value {
                                    ContentFieldValueArity::Single { value: Some(field_value) } => {
                                        Some(field_value.to_string())
                                    }
                                    _ => None,
                                },
                            };

                            if let Some(value) = field_value {
                                let count = self.content_repo.count_all_by_schema_field_value(conn, create_content_dto.schema_id, &field.name, &value).await?;
                                if count > 0 {
                                    let scoped_name = format!("fields[{}]", &field.name);
                                    validator.error_details().add_detail(scoped_name, ERR_NOT_UNIQUE);
                                }
                            }
                        }
                    }
                }

                validator.do_validate()?;

                self.content_repo.save(conn, NewRecord::new(create_content_dto)).await
            })
            .await
    }

    pub async fn delete_content(&self, content_model: ContentModel) -> Result<ContentModel, LsError> {
        self.c3p0
            .transaction(async |conn| {
                self.content_repo.delete(conn, content_model).await
            })
            .await
    }

}
