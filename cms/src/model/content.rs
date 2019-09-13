use crate::model::schema::{Schema, SchemaField, SchemaFieldValidation};
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};
use std::collections::BTreeMap;

pub struct Content {
    pub fields: Vec<ContentField>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

pub struct ContentField {
    pub name: String,
    pub value: ContentFieldValue,
}

pub enum ContentFieldValue {
    Number(Option<usize>),
    String(Option<String>),
    Boolean(Option<bool>),
}

impl Content {
    pub fn validate(&self, schema: &Schema, error_details: &ErrorDetails) {
        let mut schema_fields = BTreeMap::new();
        schema.fields.iter().for_each(|field| {
            schema_fields.insert(&field.name, field);
        });

        {
            let mut field_names = vec![];
            let mut count = 0;
            for content_field in &self.fields {
                let scoped_err = error_details.with_scope(format!("fields[{}]", count));

                if field_names.contains(&&content_field.name) {
                    scoped_err.add_detail("name", "MUST_BE_UNIQUE");
                } else if let Some(schema_field) = schema_fields.remove(&content_field.name) {
                    content_field.validate(schema_field, &scoped_err);
                } else {
                    scoped_err.add_detail("name", "UNKNOWN");
                }
                field_names.push(&content_field.name);

                count += 1;
            }
        }

        {
            if !schema_fields.is_empty() {
                error_details.add_detail(
                    "fields",
                    (
                        "MISSING_REQUIRED",
                        schema_fields
                            .iter()
                            .filter(|(_, value)| value.required)
                            .map(|(key, _)| key.to_string())
                            .collect(),
                    ),
                );
            }
        }
    }
}

impl ContentField {
    fn validate(&self, schema_field: &SchemaField, error_details: &ErrorDetails) {
        validate_number_ge(error_details, "name", 1, self.name.len());

        let full_field_name = "value";
        match schema_field.field_validation {
            SchemaFieldValidation::Boolean { .. } => match &self.value {
                ContentFieldValue::Boolean(value) => {
                    if value.is_none() && schema_field.required {
                        error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                    }
                }
                _ => error_details.add_detail(full_field_name, "MUST_BE_OF_TYPE_BOOLEAN"),
            },
            SchemaFieldValidation::Number { min, max, .. } => match &self.value {
                ContentFieldValue::Number(value) => {
                    if let Some(value) = value {
                        if let Some(min) = min {
                            validate_number_ge(error_details, full_field_name, min, *value)
                        }
                        if let Some(max) = max {
                            validate_number_le(error_details, full_field_name, max, *value)
                        }
                    } else if schema_field.required {
                        error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                    }
                }
                _ => error_details.add_detail(full_field_name, "MUST_BE_OF_TYPE_NUMBER"),
            },
            SchemaFieldValidation::Slug => unimplemented!(),
            SchemaFieldValidation::String {
                min_length,
                max_length,
                ..
            } => match &self.value {
                ContentFieldValue::String(value) => {
                    if let Some(value) = value {
                        if let Some(min_length) = min_length {
                            validate_number_ge(
                                error_details,
                                full_field_name,
                                min_length,
                                value.len(),
                            )
                        }
                        if let Some(max_length) = max_length {
                            validate_number_le(
                                error_details,
                                full_field_name,
                                max_length,
                                value.len(),
                            )
                        }
                    } else if schema_field.required {
                        error_details.add_detail(full_field_name, "VALUE_REQUIRED");
                    }
                }
                _ => error_details.add_detail(full_field_name, "MUST_BE_OF_TYPE_STRING"),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::schema::{SchemaField, SchemaFieldValidation, SchemaFieldValue};
    use lightspeed_core::error::{ErrorDetail, LightSpeedError};
    use lightspeed_core::service::validator::number::{
        MUST_BE_GREATER_OR_EQUAL, MUST_BE_LESS_OR_EQUAL,
    };
    use lightspeed_core::service::validator::Validator;

    #[test]
    fn content_validation_should_fail_if_fields_with_same_name() {
        // Arrange
        let mut content = Content {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![],
        };
        content.fields.push(ContentField {
            name: "one".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });
        content.fields.push(ContentField {
            name: "one".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });
        content.fields.push(ContentField {
            name: "two".to_owned(),
            value: ContentFieldValue::Boolean(None),
        });

        let schema = Schema {
            name: "schema".to_owned(),
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                SchemaField {
                    name: "one".to_owned(),
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        value: SchemaFieldValue::Single { unique: false },
                        default: None,
                    },
                    required: false,
                },
                SchemaField {
                    name: "two".to_owned(),
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        value: SchemaFieldValue::Single { unique: false },
                        default: None,
                    },
                    required: false,
                },
            ],
        };

        // Act
        let result = Validator::validate(|error_details: &ErrorDetails| {
            content.validate(&schema, error_details);
            Ok(())
        });

        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(details.details().borrow().len(), 1);
                assert_eq!(
                    details.details().borrow().get("fields[1].name"),
                    Some(&vec![ErrorDetail::new("MUST_BE_UNIQUE", vec![])])
                );
            }
            _ => assert!(false),
        }
    }

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
            fields: vec![],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_ok());
    }

    #[test]
    fn validation_should_fail_if_content_has_unknown_fields() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            name: "".to_owned(),
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Boolean {
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                ContentField {
                    name: "one".to_owned(),
                    value: ContentFieldValue::Boolean(Some(true)),
                },
                ContentField {
                    name: "two".to_owned(),
                    value: ContentFieldValue::Boolean(Some(true)),
                },
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[1].name"],
                vec![ErrorDetail::new("UNKNOWN", vec![])]
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
                    name: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
                SchemaField {
                    name: "two".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
                SchemaField {
                    name: "three".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
                SchemaField {
                    name: "four".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
            ],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "two".to_owned(),
                value: ContentFieldValue::Boolean(Some(true)),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields"],
                vec![ErrorDetail::new(
                    "MISSING_REQUIRED",
                    vec!["four".to_owned(), "one".to_owned()]
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Boolean {
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new("MUST_BE_OF_TYPE_BOOLEAN", vec![])]
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: None,
                    max_length: None,
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(Some(false)),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new("MUST_BE_OF_TYPE_STRING", vec![])]
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: None,
                    max: None,
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new("MUST_BE_OF_TYPE_NUMBER", vec![])]
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: Some(100),
                    max: None,
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Number(Some(99)),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(
                    MUST_BE_GREATER_OR_EQUAL,
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::Number {
                    min: Some(100),
                    max: Some(1000),
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Number(Some(1099)),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(
                    MUST_BE_LESS_OR_EQUAL,
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: Some(1000),
                    max_length: None,
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world".to_owned())),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(
                    MUST_BE_GREATER_OR_EQUAL,
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
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_validation: SchemaFieldValidation::String {
                    min_length: Some(1),
                    max_length: Some(10),
                    default: None,
                    value: SchemaFieldValue::Single { unique: false },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(Some("hello world?!?!?!?!?!".to_owned())),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(
                    MUST_BE_LESS_OR_EQUAL,
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
                    name: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
                SchemaField {
                    name: "two".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_validation: SchemaFieldValidation::Boolean {
                        default: None,
                        value: SchemaFieldValue::Single { unique: false },
                    },
                },
            ],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                ContentField {
                    name: "one".to_owned(),
                    value: ContentFieldValue::Boolean(None),
                },
                ContentField {
                    name: "two".to_owned(),
                    value: ContentFieldValue::Boolean(None),
                },
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new("VALUE_REQUIRED", vec![])]
            ),
            _ => assert!(false),
        };
    }

    pub fn validate_content(schema: &Schema, content: &Content) -> Result<(), LightSpeedError> {
        Validator::validate(|error_details: &ErrorDetails| {
            content.validate(schema, error_details);
            Ok(())
        })
    }

}
