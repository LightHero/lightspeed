use crate::data;
use lightspeed_cms::dto::create_schema_dto::CreateSchemaDto;
use lightspeed_cms::model::content::{Content, ContentData, ContentFieldValue, ContentFieldValueArity};
use lightspeed_cms::model::schema::{Schema, SchemaField, SchemaFieldArity, SchemaFieldType};
use lightspeed_core::error::{ErrorDetail, LsError};
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;
use lightspeed_core::utils::new_hyphenated_uuid;
use lightspeed_test_utils::tokio_test;
use std::collections::HashMap;

#[test]
fn should_save_and_delete_content() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let mut schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema { created_ms: 0, updated_ms: 0, fields: vec![] },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        schema.name = new_hyphenated_uuid();
        let saved_schema_2 = schema_service.create_schema(schema).await?;

        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_1.id).await?);
        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_2.id).await?);

        let mut content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content { fields: HashMap::new(), created_ms: 0, updated_ms: 0 },
        };

        let content_model_1 = content_service.create_content(&saved_schema_1.data.schema, content.clone()).await?;
        assert_eq!(1, content_service.count_all_by_schema_id(saved_schema_1.id).await?);
        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_2.id).await?);

        content.schema_id = saved_schema_2.id;
        let content_model_2 = content_service.create_content(&saved_schema_2.data.schema, content.clone()).await?;
        assert_eq!(1, content_service.count_all_by_schema_id(saved_schema_1.id).await?);
        assert_eq!(1, content_service.count_all_by_schema_id(saved_schema_2.id).await?);

        assert!(content_service.delete_content(content_model_1).await.is_ok());
        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_1.id).await?);
        assert_eq!(1, content_service.count_all_by_schema_id(saved_schema_2.id).await?);

        assert!(content_service.delete_content(content_model_2).await.is_ok());
        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_1.id).await?);
        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_2.id).await?);

        Ok(())
    })
}

#[test]
fn should_validate_content_on_save() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: "label1".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean { arity: SchemaFieldArity::Single, default: None },
                    required: false,
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content { fields: HashMap::new(), created_ms: 0, updated_ms: 0 },
        };

        let result = content_service.create_content(&saved_schema_1.data.schema, content).await;

        match result {
            Err(LsError::ValidationError { .. }) => {}
            _ => panic!(),
        }

        assert_eq!(0, content_service.count_all_by_schema_id(saved_schema_1.id).await?);

        Ok(())
    })
}

#[test]
fn should_create_unique_constraints_for_slug_schema_fields() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "slug";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: field_name.to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Slug,
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::from([(
                    field_name.to_owned(),
                    ContentFieldValue::Slug {
                        value: ContentFieldValueArity::Single { value: Some("a-valid-slug".to_owned()) },
                    },
                )]),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone()).await?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone()).await;
        assert!(result.is_err());

        match result {
            Err(LsError::ValidationError { details }) => {
                println!("details: {details:#?}");
                assert_eq!(
                    details.details[&format!("fields[{field_name}]")],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn should_create_unique_constraints_for_string_unique_schema_fields() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "name";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: field_name.to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::String {
                        default: None,
                        max_length: None,
                        min_length: None,
                        arity: SchemaFieldArity::Unique,
                    },
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::from([(
                    field_name.to_owned(),
                    ContentFieldValue::String {
                        value: ContentFieldValueArity::Single { value: Some("a-valid-string".to_owned()) },
                    },
                )]),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone()).await?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone()).await;
        assert!(result.is_err());

        match result {
            Err(LsError::ValidationError { details }) => {
                println!("details: {details:#?}");
                assert_eq!(
                    details.details[&format!("fields[{field_name}]")],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn should_create_unique_constraints_for_number_unique_schema_fields() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "epoch";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: field_name.to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Number {
                        default: None,
                        max: None,
                        min: None,
                        arity: SchemaFieldArity::Unique,
                    },
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::from([(
                    field_name.to_owned(),
                    ContentFieldValue::Number { value: ContentFieldValueArity::Single { value: Some(123456789) } },
                )]),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone()).await?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone()).await;
        assert!(result.is_err());

        match result {
            Err(LsError::ValidationError { details }) => {
                println!("details: {details:#?}");
                assert_eq!(
                    details.details[&format!("fields[{field_name}]")],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => panic!(),
        };

        Ok(())
    })
}

#[test]
fn should_create_unique_constraints_for_boolean_unique_schema_fields() -> Result<(), LsError> {
    tokio_test(async {
        let data = data(false).await;
        let cms_module = &data.0;

        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "male";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: 0,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: field_name.to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean { default: None, arity: SchemaFieldArity::Unique },
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone()).await?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::from([(
                    field_name.to_owned(),
                    ContentFieldValue::Boolean { value: ContentFieldValueArity::Single { value: Some(true) } },
                )]),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone()).await?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone()).await;
        assert!(result.is_err());

        match result {
            Err(LsError::ValidationError { details }) => {
                println!("details: {details:#?}");
                assert_eq!(
                    details.details[&format!("fields[{field_name}]")],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => panic!(),
        };

        Ok(())
    })
}
