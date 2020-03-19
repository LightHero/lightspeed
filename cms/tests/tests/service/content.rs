use crate::test;
use lightspeed_cms::dto::create_schema_dto::CreateSchemaDto;
use lightspeed_cms::model::content::{Content, ContentData, ContentFieldValue, ContentFieldValueArity};
use lightspeed_cms::model::schema::{Schema, SchemaField, SchemaFieldType, SchemaFieldArity, SCHEMA_FIELD_NAME_MAX_LENGHT};
use lightspeed_core::utils::{new_hyphenated_uuid, random_string};
use std::collections::HashMap;
use lightspeed_core::error::{LightSpeedError, ErrorDetail};
use maplit::*;
use lightspeed_core::service::validator::ERR_NOT_UNIQUE;

#[test]
fn should_create_and_drop_content_table() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let mut schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;

        schema.name = new_hyphenated_uuid();
        let saved_schema_2 = schema_service.create_schema(schema)?;

        assert!(content_service
            .count_all_by_schema_id(saved_schema_1.id)
            .is_err());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_2.id)
            .is_err());

        assert!(content_service
            .create_content_table(&saved_schema_1)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_1.id)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_2.id)
            .is_err());

        assert!(content_service
            .create_content_table(&saved_schema_2)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_1.id)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_2.id)
            .is_ok());

        assert!(content_service
            .drop_content_table(saved_schema_2.id)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_1.id)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_2.id)
            .is_err());

        assert!(content_service
            .drop_content_table(saved_schema_1.id)
            .is_ok());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_1.id)
            .is_err());
        assert!(content_service
            .count_all_by_schema_id(saved_schema_2.id)
            .is_err());

        Ok(())
    });
}

#[test]
fn should_save_and_delete_content() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let mut schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        assert!(content_service
            .create_content_table(&saved_schema_1)
            .is_ok());

        schema.name = new_hyphenated_uuid();
        let saved_schema_2 = schema_service.create_schema(schema)?;
        assert!(content_service
            .create_content_table(&saved_schema_2)
            .is_ok());

        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );
        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_2.id)?
        );

        let mut content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::new(),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        let content_model_1 =
            content_service.create_content(&saved_schema_1.data.schema, content.clone())?;
        assert_eq!(
            1,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );
        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_2.id)?
        );

        content.schema_id = saved_schema_2.id;
        let content_model_2 =
            content_service.create_content(&saved_schema_2.data.schema, content.clone())?;
        assert_eq!(
            1,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );
        assert_eq!(
            1,
            content_service.count_all_by_schema_id(saved_schema_2.id)?
        );

        assert!(content_service.delete_content(content_model_1).is_ok());
        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );
        assert_eq!(
            1,
            content_service.count_all_by_schema_id(saved_schema_2.id)?
        );

        assert!(content_service.delete_content(content_model_2).is_ok());
        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );
        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_2.id)?
        );

        Ok(())
    });
}

#[test]
fn should_validate_content_on_save() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![SchemaField {
                    name: "label1".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        arity: SchemaFieldArity::Single,
                        default: None,
                    },
                    required: false,
                }],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        assert!(content_service
            .create_content_table(&saved_schema_1)
            .is_ok());

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: HashMap::new(),
                created_ms: 0,
                updated_ms: 0,
            },
        };

        let result =
            content_service.create_content(&saved_schema_1.data.schema, content);

        match result {
            Err(LightSpeedError::ValidationError { .. }) => {},
            _ => assert!(false),
        }

        assert_eq!(
            0,
            content_service.count_all_by_schema_id(saved_schema_1.id)?
        );

        Ok(())
    });
}

#[test]
fn should_create_unique_constraints_for_slug_schema_fields() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "slug";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![
                    SchemaField{
                        name: field_name.to_owned(),
                        required: true,
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Slug
                    }
                ],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        content_service.create_content_table(&saved_schema_1)?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: hashmap![
                    field_name.to_owned() =>
                    ContentFieldValue::Slug{value: ContentFieldValueArity::Single { value: Some("a-valid-slug".to_owned()) }},
                ],
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone())?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone());
        assert!(result.is_err());

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                println!("details: {:#?}", details);
                assert_eq!(
                    details.details[&format!("fields[{}]", field_name)],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => assert!(false),
        };

        Ok(())
    });
}

#[test]
fn should_create_unique_constraints_for_string_unique_schema_fields() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "name";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![
                    SchemaField{
                        name: field_name.to_owned(),
                        required: true,
                        description: "".to_owned(),
                        field_type: SchemaFieldType::String {
                            default: None,
                            max_length: None,
                            min_length: None,
                            arity: SchemaFieldArity::Unique
                        }
                    }
                ],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        content_service.create_content_table(&saved_schema_1)?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: hashmap![
                    field_name.to_owned() =>
                    ContentFieldValue::String{value: ContentFieldValueArity::Single { value: Some("a-valid-string".to_owned()) }},
                ],
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone())?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone());
        assert!(result.is_err());

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                println!("details: {:#?}", details);
                assert_eq!(
                    details.details[&format!("fields[{}]", field_name)],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => assert!(false),
        };

        Ok(())
    });
}

#[test]
fn should_create_unique_constraints_for_number_unique_schema_fields() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "epoch";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![
                    SchemaField{
                        name: field_name.to_owned(),
                        required: true,
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Number {
                            default: None,
                            max: None,
                            min: None,
                            arity: SchemaFieldArity::Unique
                        }
                    }
                ],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        content_service.create_content_table(&saved_schema_1)?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: hashmap![
                    field_name.to_owned() =>
                    ContentFieldValue::Number{value: ContentFieldValueArity::Single { value: Some(123456789) }},
                ],
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone())?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone());
        assert!(result.is_err());

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                println!("details: {:#?}", details);
                assert_eq!(
                    details.details[&format!("fields[{}]", field_name)],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => assert!(false),
        };

        Ok(())
    });
}

#[test]
fn should_create_unique_constraints_for_boolean_unique_schema_fields() {

    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = "male";
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![
                    SchemaField{
                        name: field_name.to_owned(),
                        required: true,
                        description: "".to_owned(),
                        field_type: SchemaFieldType::Boolean {
                            default: None,
                            arity: SchemaFieldArity::Unique
                        }
                    }
                ],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        content_service.create_content_table(&saved_schema_1)?;

        let content = ContentData {
            schema_id: saved_schema_1.id,
            content: Content {
                fields: hashmap![
                    field_name.to_owned() =>
                    ContentFieldValue::Boolean{value: ContentFieldValueArity::Single { value: Some(true) }},
                ],
                created_ms: 0,
                updated_ms: 0,
            },
        };

        content_service.create_content(&saved_schema_1.data.schema, content.clone())?;

        let result = content_service.create_content(&saved_schema_1.data.schema, content.clone());
        assert!(result.is_err());

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                println!("details: {:#?}", details);
                assert_eq!(
                    details.details[&format!("fields[{}]", field_name)],
                    vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])]
                )
            }
            _ => assert!(false),
        };

        Ok(())
    });

}

#[test]
fn should_create_unique_constraints_for_field_name_with_max_length() {
    test(|cms_module| {
        let content_service = &cms_module.content_service;
        let schema_service = &cms_module.schema_service;

        let field_name = random_string(SCHEMA_FIELD_NAME_MAX_LENGHT).to_lowercase();
        let schema = CreateSchemaDto {
            name: new_hyphenated_uuid(),
            project_id: -1,
            schema: Schema {
                created_ms: 0,
                updated_ms: 0,
                fields: vec![
                    SchemaField{
                        name: field_name.to_owned(),
                        required: true,
                        description: "".to_owned(),
                        field_type: SchemaFieldType::String {
                            default: None,
                            max_length: None,
                            min_length: None,
                            arity: SchemaFieldArity::Unique
                        }
                    }
                ],
            },
        };

        let saved_schema_1 = schema_service.create_schema(schema.clone())?;
        assert!(content_service.create_content_table(&saved_schema_1).is_ok());

        Ok(())
    });
}