use crate::model::schema::{
    LocalizableOptions, Schema, SchemaField, SchemaFieldArity, SchemaFieldType,
};
use lazy_static::*;
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};
use regex::Regex;
use std::collections::{BTreeMap, HashMap};

pub const SLUG_VALIDATION_REGEX: &str = r#"^[a-z0-9]+(?:-[a-z0-9]+)*$"#;

const VALUE_REQUIRED: &str = "VALUE_REQUIRED";
const MUST_BE_UNIQUE: &str = "MUST_BE_UNIQUE";
const UNKNOWN: &str = "UNKNOWN";
const MUST_BE_OF_TYPE_BOOLEAN: &str = "MUST_BE_OF_TYPE_BOOLEAN";
const MUST_BE_OF_TYPE_NUMBER: &str = "MUST_BE_OF_TYPE_NUMBER";
const MUST_BE_OF_TYPE_SLUG: &str = "MUST_BE_OF_TYPE_SLUG";
const MUST_BE_OF_TYPE_STRING: &str = "MUST_BE_OF_TYPE_STRING";
const SHOULD_HAVE_SINGLE_VALUE_ARITY: &str = "SHOULD_HAVE_SINGLE_VALUE_ARITY";
const SHOULD_HAVE_LOCALIZABLE_ARITY: &str = "SHOULD_HAVE_LOCALIZABLE_ARITY";
const NOT_VALID_SLUG: &str = "NOT_VALID_SLUG";

lazy_static! {
    static ref SLUG_REGEX: Regex =
        Regex::new(SLUG_VALIDATION_REGEX).expect("slug validation regex should be valid");
}

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
    Number(ContentFieldValueArity<Option<usize>>),
    Slug(String),
    String(ContentFieldValueArity<Option<String>>),
    Boolean(ContentFieldValueArity<Option<bool>>),
}

pub enum ContentFieldValueArity<T> {
    Single { value: T },
    Localizable { values: HashMap<String, T> },
}

impl Content {
    pub fn validate(&self, schema: &Schema, error_details: &ErrorDetails) {
        let mut schema_fields = BTreeMap::new();
        schema.fields.iter().for_each(|field| {
            schema_fields.insert(&field.name, field);
        });

        {
            let mut field_names = vec![];

            for (count, content_field) in (&self.fields).iter().enumerate() {
                let scoped_err = error_details.with_scope(format!("fields[{}]", count));

                if field_names.contains(&&content_field.name) {
                    scoped_err.add_detail("name", MUST_BE_UNIQUE);
                } else if let Some(schema_field) = schema_fields.remove(&content_field.name) {
                    content_field.validate(schema_field, &scoped_err);
                } else {
                    scoped_err.add_detail("name", UNKNOWN);
                }
                field_names.push(&content_field.name);
            }
        }

        {
            if !schema_fields.is_empty() {
                error_details.add_detail(
                    "fields",
                    (
                        VALUE_REQUIRED,
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
        match &schema_field.field_type {
            SchemaFieldType::Boolean {
                arity: schema_arity,
                default: _default,
            } => match &self.value {
                ContentFieldValue::Boolean(arity) => {
                    ContentField::validate_arity(
                        schema_field.required,
                        schema_arity,
                        arity,
                        error_details,
                        full_field_name,
                        |field_name, value| {
                            ContentField::validate_boolean(
                                schema_field.required,
                                field_name,
                                *value,
                                error_details,
                            )
                        },
                    );
                }
                _ => error_details.add_detail(full_field_name, MUST_BE_OF_TYPE_BOOLEAN),
            },
            SchemaFieldType::Number {
                min,
                max,
                arity: schema_arity,
                default: _default,
            } => match &self.value {
                ContentFieldValue::Number(arity) => {
                    ContentField::validate_arity(
                        schema_field.required,
                        schema_arity,
                        arity,
                        error_details,
                        full_field_name,
                        |field_name, value| {
                            ContentField::validate_number(
                                schema_field.required,
                                field_name,
                                value,
                                min,
                                max,
                                error_details,
                            )
                        },
                    );
                }
                _ => error_details.add_detail(full_field_name, MUST_BE_OF_TYPE_NUMBER),
            },
            SchemaFieldType::Slug => match &self.value {
                ContentFieldValue::Slug(value) => {
                    ContentField::validate_slug(full_field_name, value, error_details)
                }
                _ => error_details.add_detail(full_field_name, MUST_BE_OF_TYPE_SLUG),
            },
            SchemaFieldType::String {
                min_length,
                max_length,
                arity: schema_arity,
                default: _default,
            } => match &self.value {
                ContentFieldValue::String(arity) => {
                    ContentField::validate_arity(
                        schema_field.required,
                        schema_arity,
                        arity,
                        error_details,
                        full_field_name,
                        |field_name, value| {
                            ContentField::validate_string(
                                schema_field.required,
                                field_name,
                                value,
                                min_length,
                                max_length,
                                error_details,
                            )
                        },
                    );
                }
                _ => error_details.add_detail(full_field_name, MUST_BE_OF_TYPE_STRING),
            },
        }
    }

    fn validate_arity<T, F: Fn(&str, &Option<T>)>(
        required: bool,
        schema_arity: &SchemaFieldArity,
        arity: &ContentFieldValueArity<Option<T>>,
        error_details: &ErrorDetails,
        full_field_name: &str,
        value_validation: F,
    ) {
        match schema_arity {
            SchemaFieldArity::Single | SchemaFieldArity::Unique => match arity {
                ContentFieldValueArity::Single { value } => {
                    value_validation(full_field_name, value)
                }
                _ => error_details.add_detail(full_field_name, SHOULD_HAVE_SINGLE_VALUE_ARITY),
            },
            SchemaFieldArity::Localizable { options } => match arity {
                ContentFieldValueArity::Localizable { values } => {
                    match options {
                        LocalizableOptions::Languages { languages } => {
                            if required {
                                languages.iter().for_each(|language| {
                                    if !values.contains_key(language) {
                                        error_details.add_detail(
                                            format!("{}[{}]", full_field_name, language),
                                            VALUE_REQUIRED,
                                        )
                                    }
                                })
                            }
                        }
                    }
                    values.iter().for_each(|(key, value)| {
                        value_validation(&format!("{}[{}]", full_field_name, key), value)
                    })
                }
                _ => error_details.add_detail(full_field_name, SHOULD_HAVE_LOCALIZABLE_ARITY),
            },
        }
    }

    fn validate_boolean<S: Into<String>>(
        required: bool,
        full_field_name: S,
        value: Option<bool>,
        error_details: &ErrorDetails,
    ) {
        if value.is_none() && required {
            error_details.add_detail(full_field_name, VALUE_REQUIRED);
        }
    }

    fn validate_number<S: Into<String> + Clone>(
        required: bool,
        full_field_name: S,
        value: &Option<usize>,
        min: &Option<usize>,
        max: &Option<usize>,
        error_details: &ErrorDetails,
    ) {
        if let Some(value) = value {
            if let Some(min) = min {
                validate_number_ge(error_details, full_field_name.clone(), *min, *value)
            }
            if let Some(max) = max {
                validate_number_le(error_details, full_field_name, *max, *value)
            }
        } else if required {
            error_details.add_detail(full_field_name, VALUE_REQUIRED);
        }
    }

    fn validate_slug<S: Into<String>>(
        full_field_name: S,
        value: &str,
        error_details: &ErrorDetails,
    ) {
        //let reg: &Regex = &SLUG_REGEX;
        if !SLUG_REGEX.is_match(value) {
            error_details.add_detail(full_field_name, NOT_VALID_SLUG);
        }
    }

    fn validate_string<S: Into<String> + Clone>(
        required: bool,
        full_field_name: S,
        value: &Option<String>,
        min_length: &Option<usize>,
        max_length: &Option<usize>,
        error_details: &ErrorDetails,
    ) {
        if let Some(value) = value {
            if let Some(min_length) = min_length {
                validate_number_ge(
                    error_details,
                    full_field_name.clone(),
                    *min_length,
                    value.len(),
                )
            }
            if let Some(max_length) = max_length {
                validate_number_le(error_details, full_field_name, *max_length, value.len())
            }
        } else if required {
            error_details.add_detail(full_field_name, VALUE_REQUIRED);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::schema::{SchemaField, SchemaFieldArity, SchemaFieldType};
    use lightspeed_core::error::{ErrorDetail, LightSpeedError};
    use lightspeed_core::service::validator::number::{
        MUST_BE_GREATER_OR_EQUAL, MUST_BE_LESS_OR_EQUAL,
    };
    use lightspeed_core::service::validator::Validator;
    use maplit::*;

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
            value: ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }),
        });
        content.fields.push(ContentField {
            name: "one".to_owned(),
            value: ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }),
        });
        content.fields.push(ContentField {
            name: "two".to_owned(),
            value: ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }),
        });

        let schema = Schema {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                SchemaField {
                    name: "one".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        arity: SchemaFieldArity::Single,
                        default: None,
                    },
                    required: false,
                },
                SchemaField {
                    name: "two".to_owned(),
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        arity: SchemaFieldArity::Single,
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
                    Some(&vec![ErrorDetail::new(MUST_BE_UNIQUE, vec![])])
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
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Boolean {
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![
                ContentField {
                    name: "one".to_owned(),
                    value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                        value: Some(true),
                    }),
                },
                ContentField {
                    name: "two".to_owned(),
                    value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                        value: Some(true),
                    }),
                },
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[1].name"],
                vec![ErrorDetail::new(UNKNOWN, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_missing_fields() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![
                SchemaField {
                    name: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
                    },
                },
                SchemaField {
                    name: "two".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
                    },
                },
                SchemaField {
                    name: "three".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
                    },
                },
                SchemaField {
                    name: "four".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
                    },
                },
            ],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "two".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                    value: Some(true),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields"],
                vec![ErrorDetail::new(
                    VALUE_REQUIRED,
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
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Boolean {
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(MUST_BE_OF_TYPE_BOOLEAN, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_string_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::String {
                    min_length: None,
                    max_length: None,
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                    value: Some(false),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(MUST_BE_OF_TYPE_STRING, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_slug_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Slug,
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                    value: Some(false),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(MUST_BE_OF_TYPE_SLUG, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_fields_of_not_number_type() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Number {
                    min: None,
                    max: None,
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(MUST_BE_OF_TYPE_NUMBER, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_content_has_number_field_with_value_less_than_min() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Number {
                    min: Some(100),
                    max: None,
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Number(ContentFieldValueArity::Single {
                    value: Some(99),
                }),
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
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Number {
                    min: Some(100),
                    max: Some(1000),
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Number(ContentFieldValueArity::Single {
                    value: Some(1099),
                }),
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
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::String {
                    min_length: Some(1000),
                    max_length: None,
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }),
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
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::String {
                    min_length: Some(1),
                    max_length: Some(10),
                    default: None,
                    arity: SchemaFieldArity::Single,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::String(ContentFieldValueArity::Single {
                    value: Some("hello world?!?!?!?!?!".to_owned()),
                }),
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
            fields: vec![
                SchemaField {
                    name: "one".to_owned(),
                    required: true,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
                    },
                },
                SchemaField {
                    name: "two".to_owned(),
                    required: false,
                    description: "".to_owned(),
                    field_type: SchemaFieldType::Boolean {
                        default: None,
                        arity: SchemaFieldArity::Single,
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
                    value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                        value: None,
                    }),
                },
                ContentField {
                    name: "two".to_owned(),
                    value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                        value: None,
                    }),
                },
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(VALUE_REQUIRED, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_single_arity_is_not_matched() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: false,
                description: "".to_owned(),
                field_type: SchemaFieldType::Boolean {
                    default: None,
                    arity: SchemaFieldArity::Unique,
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Localizable {
                    values: HashMap::new(),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(SHOULD_HAVE_SINGLE_VALUE_ARITY, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_localizable_arity_is_not_matched() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Boolean {
                    default: None,
                    arity: SchemaFieldArity::Localizable {
                        options: LocalizableOptions::Languages { languages: vec![] },
                    },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Single {
                    value: Some(true),
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(SHOULD_HAVE_LOCALIZABLE_ARITY, vec![])]
            ),
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_fail_if_localizable_required_languages_missing() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "one".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Boolean {
                    default: None,
                    arity: SchemaFieldArity::Localizable {
                        options: LocalizableOptions::Languages {
                            languages: vec!["IT".to_owned(), "EN".to_owned(), "FR".to_owned()],
                        },
                    },
                },
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "one".to_owned(),
                value: ContentFieldValue::Boolean(ContentFieldValueArity::Localizable {
                    values: hashmap![
                        "IT".to_owned() => Some(true),
                        "EN".to_owned() => None,
                    ],
                }),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(
                    details.details().borrow()["fields[0].value[FR]"],
                    vec![ErrorDetail::new(VALUE_REQUIRED, vec![])]
                );
                assert_eq!(
                    details.details().borrow()["fields[0].value[EN]"],
                    vec![ErrorDetail::new(VALUE_REQUIRED, vec![])]
                );
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn validation_should_accept_valid_slug() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "slug".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Slug,
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "slug".to_owned(),
                value: ContentFieldValue::Slug("a-valid-slug".to_owned()),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_ok());
    }

    #[test]
    fn validation_should_fail_when_not_valid_slug() {
        let schema = Schema {
            created_ms: 0,
            updated_ms: 0,
            fields: vec![SchemaField {
                name: "slug".to_owned(),
                required: true,
                description: "".to_owned(),
                field_type: SchemaFieldType::Slug,
            }],
        };
        let content = Content {
            updated_ms: 0,
            created_ms: 0,
            fields: vec![ContentField {
                name: "slug".to_owned(),
                value: ContentFieldValue::Slug("a---notvalid-slug!".to_owned()),
            }],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[0].value"],
                vec![ErrorDetail::new(NOT_VALID_SLUG, vec![])]
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
