use crate::model::content::{Content, ContentField};
use crate::model::schema::Schema;
use lightspeed_core::error::{ErrorDetail, ErrorDetails, LightSpeedError};
use lightspeed_core::service::validator::Validator;

pub struct ContentService {}

impl ContentService {
    pub fn validate_content(schema: &Schema, content: &Content) -> Result<(), LightSpeedError> {
        Validator::validate(|error_details: &ErrorDetails| {
            let content_fields_not_in_schema = get_content_fields_not_in_schema(schema, content);
            if !content_fields_not_in_schema.is_empty() {
                error_details.add_detail(
                    "fields",
                    ErrorDetail::new("UNKNOWN", content_fields_not_in_schema),
                );
            }

            let mut content_missing_fields = vec![];

            for schema_field in &schema.fields {
                if let Some(content_field) = get_content_field_by_name(&schema_field.name, content)
                {
                    content_field.validate(schema_field, error_details);
                } else if schema_field.required {
                    content_missing_fields.push(schema_field.name.to_owned())
                }
            }

            if !content_missing_fields.is_empty() {
                error_details.add_detail(
                    "fields",
                    ErrorDetail::new("MISSING_REQUIRED", content_missing_fields),
                );
            }

            Ok(())
        })
    }
}



/*
#[cfg(test)]
mod test {

    use super::*;
    use crate::model::content::ContentFieldValue;
    use crate::model::schema::{SchemaField, SchemaFieldValidation};

    #[test]
    fn empty_schema_should_validate_empty_content() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![],
        };

        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_ok());
    }

    #[test]
    fn validation_should_fail_if_content_has_unknown_fields() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Boolean { default: None },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![
                ContentField {
                    label: "one".to_owned(),
                    value: ContentFieldValue::Boolean(Some(true)),
                },
                ContentField {
                    label: "two".to_owned(),
                    value: ContentFieldValue::Boolean(Some(true)),
                },
            ],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields"],
                vec![ErrorDetail::new("UNKNOWN", vec!["two".to_owned()])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_missing_fields() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![
                SchemaField {
                    label: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
                SchemaField {
                    label: "two".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
                SchemaField {
                    label: "three".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
                SchemaField {
                    label: "four".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
            ],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "two".to_owned(),
                value: ContentFieldValue::Boolean(Some(true)),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields"],
                vec![ErrorDetail::new(
                    "MISSING_REQUIRED",
                    vec!["one".to_owned(), "four".to_owned()]
                )]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_boolean_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Boolean { default: None },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new("SHOULD_BE_OF_TYPE_BOOLEAN", vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_string_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: None,
                    max_length: None,
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::Boolean(Some(false)),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new("SHOULD_BE_OF_TYPE_STRING", vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_number_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: None,
                    max: None,
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new("SHOULD_BE_OF_TYPE_NUMBER", vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_number_field_with_value_less_than_min() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: Some(100),
                    max: None,
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::Number(Some(99)),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new(
                    "MUST_BE_GREATER_OR_EQUAL",
                    vec!["100".to_owned()]
                )]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_number_field_with_value_more_than_max() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: Some(100),
                    max: Some(1000),
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::Number(Some(1099)),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new(
                    "MUST_BE_LESS_OR_EQUAL",
                    vec!["1000".to_owned()]
                )]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_string_field_with_value_less_than_min() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: Some(1000),
                    max_length: None,
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new(
                    "MUST_BE_GREATER_OR_EQUAL",
                    vec!["1000".to_owned()]
                )]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_string_field_with_value_more_than_max() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                label: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: Some(1),
                    max_length: Some(10),
                    default: None,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![ContentField {
                label: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world?!?!?!?!?!".to_owned())),
            }],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new(
                    "MUST_BE_LESS_OR_EQUAL",
                    vec!["10".to_owned()]
                )]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_field_with_none_required_value() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![
                SchemaField {
                    label: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
                SchemaField {
                    label: "two".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean { default: None },
                },
            ],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            schema_name: "".to_owned(),
            fields: vec![
                ContentField {
                    label: "one".to_owned(),
                    value: ContentFieldValue::Boolean(None),
                },
                ContentField {
                    label: "two".to_owned(),
                    value: ContentFieldValue::Boolean(None),
                },
            ],
        };

        let result = ContentService::validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields.one"],
                vec![ErrorDetail::new("VALUE_REQUIRED", vec![])]
            ),
            _ => assert!(false),
        };
    }

}
*/
