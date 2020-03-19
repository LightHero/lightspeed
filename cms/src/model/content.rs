use crate::model::schema::{
    LocalizableOptions, Schema, SchemaField, SchemaFieldArity, SchemaFieldType,
};
use c3p0::Model;
use lazy_static::*;
use lightspeed_core::error::ErrorDetails;
use lightspeed_core::service::validator::number::{validate_number_ge, validate_number_le};
use lightspeed_core::service::validator::{ERR_UNKNOWN_FIELD, ERR_VALUE_REQUIRED};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub const SLUG_VALIDATION_REGEX: &str = r#"^[a-z0-9]+(?:-[a-z0-9]+)*$"#;

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

pub type ContentModel = Model<ContentData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct ContentData {
    pub schema_id: i64,
    pub content: Content,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Content {
    pub fields: HashMap<String, ContentFieldValue>,
    pub created_ms: i64,
    pub updated_ms: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum ContentFieldValue {
    Number{value: ContentFieldValueArity<Option<usize>>},
    Slug{value: ContentFieldValueArity<Option<String>>},
    String{value: ContentFieldValueArity<Option<String>>},
    Boolean{value: ContentFieldValueArity<Option<bool>>},
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum ContentFieldValueArity<T> {
    Single { value: T },
    Localizable { values: HashMap<String, T> },
}

impl Content {
    pub fn validate(&self, schema: &Schema, error_details: &mut dyn ErrorDetails) {
        let mut schema_fields = BTreeMap::new();
        schema.fields.iter().for_each(|field| {
            schema_fields.insert(&field.name, field);
        });

        {
            for (content_field_name, content_field_value) in (&self.fields).iter() {
                let scoped_name = format!("fields[{}]", content_field_name);
                let mut scoped_err = error_details.with_scope(scoped_name.clone());

                if let Some(schema_field) = schema_fields.remove(content_field_name) {
                    validate_content_field(
                        content_field_name,
                        content_field_value,
                        schema_field,
                        &mut scoped_err,
                    );
                } else {
                    error_details.add_detail(scoped_name.into(), ERR_UNKNOWN_FIELD.into());
                }
            }
        }

        {
            if !schema_fields.is_empty() {
                error_details.add_detail(
                    "fields".into(),
                    (
                        ERR_VALUE_REQUIRED,
                        schema_fields
                            .iter()
                            .filter(|(_, value)| value.required)
                            .map(|(key, _)| key.to_string())
                            .collect(),
                    ).into(),
                );
            }
        }
    }
}

fn validate_content_field<E: ErrorDetails>(
    content_field_name: &str,
    content_field_value: &ContentFieldValue,
    schema_field: &SchemaField,
    error_details: &mut E,
) {
    validate_number_ge(error_details, "name", 1, content_field_name.len());

    let full_field_name = "value";
    match &schema_field.field_type {
        SchemaFieldType::Boolean {
            arity: schema_arity,
            default: _default,
        } => match content_field_value {
            ContentFieldValue::Boolean{value: arity} => {
                validate_arity(
                    schema_field.required,
                    schema_arity,
                    arity,
                    error_details,
                    full_field_name,
                    |field_name, value, error_details| {
                        validate_boolean(schema_field.required, field_name, *value, error_details)
                    },
                );
            }
            _ => error_details.add_detail(full_field_name.into(), MUST_BE_OF_TYPE_BOOLEAN.into()),
        },
        SchemaFieldType::Number {
            min,
            max,
            arity: schema_arity,
            default: _default,
        } => match content_field_value {
            ContentFieldValue::Number{value: arity} => {
                validate_arity(
                    schema_field.required,
                    schema_arity,
                    arity,
                    error_details,
                    full_field_name,
                    |field_name, value, error_details| {
                        validate_number(
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
            _ => error_details.add_detail(full_field_name.into(), MUST_BE_OF_TYPE_NUMBER.into()),
        },
        SchemaFieldType::Slug => match content_field_value {
            ContentFieldValue::Slug{value: arity} => {
                validate_arity(
                    schema_field.required,
                    schema_field.field_type.get_arity(),
                    arity,
                    error_details,
                    full_field_name,
                    |field_name, value, error_details| {
                        validate_slug(schema_field.required, field_name, value, error_details)
                    },
                );
            }
            _ => error_details.add_detail(full_field_name.into(), MUST_BE_OF_TYPE_SLUG.into()),
        },
        SchemaFieldType::String {
            min_length,
            max_length,
            arity: schema_arity,
            default: _default,
        } => match content_field_value {
            ContentFieldValue::String{value: arity} => {
                validate_arity(
                    schema_field.required,
                    schema_arity,
                    arity,
                    error_details,
                    full_field_name,
                    |field_name, value, error_details| {
                        validate_string(
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
            _ => error_details.add_detail(full_field_name.into(), MUST_BE_OF_TYPE_STRING.into()),
        },
    }
}

fn validate_arity<T, F: Fn(&str, &Option<T>, &mut E), E: ErrorDetails>(
    required: bool,
    schema_arity: &SchemaFieldArity,
    arity: &ContentFieldValueArity<Option<T>>,
    error_details: &mut E,
    full_field_name: &str,
    value_validation: F,
) {
    match schema_arity {
        SchemaFieldArity::Single | SchemaFieldArity::Unique => match arity {
            ContentFieldValueArity::Single { value } => value_validation(full_field_name, value, error_details),
            _ => error_details.add_detail(full_field_name.into(), SHOULD_HAVE_SINGLE_VALUE_ARITY.into()),
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
                                        ERR_VALUE_REQUIRED.into(),
                                    )
                                }
                            })
                        }
                    }
                }
                values.iter().for_each(|(key, value)| {
                    value_validation(&format!("{}[{}]", full_field_name, key), value, error_details)
                })
            }
            _ => error_details.add_detail(full_field_name.into(), SHOULD_HAVE_LOCALIZABLE_ARITY.into()),
        },
    }
}

fn validate_boolean<S: Into<String>, E: ErrorDetails>(
    required: bool,
    full_field_name: S,
    value: Option<bool>,
    error_details: &mut E,
) {
    if value.is_none() && required {
        error_details.add_detail(full_field_name.into(), ERR_VALUE_REQUIRED.into());
    }
}

fn validate_number<S: Into<String> + Clone, E: ErrorDetails>(
    required: bool,
    full_field_name: S,
    value: &Option<usize>,
    min: &Option<usize>,
    max: &Option<usize>,
    error_details: &mut E,
) {
    if let Some(value) = value {
        if let Some(min) = min {
            validate_number_ge(error_details, full_field_name.clone(), *min, *value)
        }
        if let Some(max) = max {
            validate_number_le(error_details, full_field_name, *max, *value)
        }
    } else if required {
        error_details.add_detail(full_field_name.into(), ERR_VALUE_REQUIRED.into());
    }
}

fn validate_slug<S: Into<String>, E: ErrorDetails>(
    required: bool,
    full_field_name: S,
    value: &Option<String>,
    error_details: &mut E,
) {
    if let Some(value) = value {
        //let reg: &Regex = &SLUG_REGEX;
        if !SLUG_REGEX.is_match(value) {
            error_details.add_detail(full_field_name.into(), NOT_VALID_SLUG.into());
        }
    } else if required {
        error_details.add_detail(full_field_name.into(), ERR_VALUE_REQUIRED.into());
    }
}

fn validate_string<S: Into<String> + Clone, E: ErrorDetails>(
    required: bool,
    full_field_name: S,
    value: &Option<String>,
    min_length: &Option<usize>,
    max_length: &Option<usize>,
    error_details: &mut E,
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
        error_details.add_detail(full_field_name.into(), ERR_VALUE_REQUIRED.into());
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

    /*
        #[test]
        fn content_validation_should_fail_if_fields_with_same_name() {
            // Arrange
            let mut content = Content {
                created_ms: 0,
                updated_ms: 0,
                fields: HashMap::new(),
            };
            content.fields.insert("one".to_owned(),
                ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }));
            content.fields.insert("one".to_owned(),
                                  ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }));
            content.fields.insert("two".to_owned(),
                                  ContentFieldValue::Boolean(ContentFieldValueArity::Single { value: None }));

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
            let result = Validator::validate(|error_details: &mut dyn ErrorDetails| {
                content.validate(&schema, error_details);
                Ok(())
            });

            match result {
                Err(LightSpeedError::ValidationError { details }) => {
                    assert_eq!(details.details().borrow().len(), 1);
                    assert_eq!(
                        details.details().borrow().get("fields[1].name"),
                        Some(&vec![ErrorDetail::new(ERR_NOT_UNIQUE, vec![])])
                    );
                }
                _ => assert!(false),
            }
        }
    */

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
            fields: HashMap::new(),
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
            fields: hashmap![
                "one".to_owned() =>
                    ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                        value: Some(true),
                    }},
                "two".to_owned() =>
                    ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                        value: Some(true),
                    }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                println!("details: {:#?}", details);
                assert_eq!(
                    details.details().borrow()["fields[two]"],
                    vec![ErrorDetail::new(ERR_UNKNOWN_FIELD, vec![])]
                )
            }
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
            fields: hashmap!["two".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                    value: Some(true),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields"],
                vec![ErrorDetail::new(
                    ERR_VALUE_REQUIRED,
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
            fields: hashmap!["one".to_owned() =>
                ContentFieldValue::String{value: ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                    value: Some(false),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                    value: Some(false),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::String{value: ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::Number{value: ContentFieldValueArity::Single {
                    value: Some(99),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::Number{value: ContentFieldValueArity::Single {
                    value: Some(1099),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::String{value: ContentFieldValueArity::Single {
                    value: Some("hello world".to_owned()),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::String{value: ContentFieldValueArity::Single {
                    value: Some("hello world?!?!?!?!?!".to_owned()),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                    ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                        value: None,
                    }},
                "two".to_owned() =>
                    ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                        value: None,
                    }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
                vec![ErrorDetail::new(ERR_VALUE_REQUIRED, vec![])]
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
            fields: hashmap!["one".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Localizable {
                    values: HashMap::new(),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![
                "one".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Single {
                    value: Some(true),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[one].value"],
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
            fields: hashmap![ "one".to_owned() =>
                ContentFieldValue::Boolean{value: ContentFieldValueArity::Localizable {
                    values: hashmap![
                        "IT".to_owned() => Some(true),
                        "EN".to_owned() => None,
                    ],
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => {
                assert_eq!(
                    details.details().borrow()["fields[one].value[FR]"],
                    vec![ErrorDetail::new(ERR_VALUE_REQUIRED, vec![])]
                );
                assert_eq!(
                    details.details().borrow()["fields[one].value[EN]"],
                    vec![ErrorDetail::new(ERR_VALUE_REQUIRED, vec![])]
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
            fields: hashmap![
                "slug".to_owned() =>
                ContentFieldValue::Slug{value: ContentFieldValueArity::Single { value: Some("a-valid-slug".to_owned()) }},
            ],
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
            fields: hashmap!["slug".to_owned()
                => ContentFieldValue::Slug{value: ContentFieldValueArity::Single { value: Some("a---notvalid-slug!".to_owned()) }} ,
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[slug].value"],
                vec![ErrorDetail::new(NOT_VALID_SLUG, vec![])]
            ),
            _ => assert!(false),
        };
    }
    #[test]
    fn validation_should_accept_valid_single_arity_slug() {
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
            fields: hashmap![
                "slug".to_owned() =>
                ContentFieldValue::Slug{value: ContentFieldValueArity::Localizable {
                    values: HashMap::new(),
                }},
            ],
        };

        let result = validate_content(&schema, &content);
        assert!(result.is_err());
        println!("{:?}", result);
        match result {
            Err(LightSpeedError::ValidationError { details }) => assert_eq!(
                details.details().borrow()["fields[slug].value"],
                vec![ErrorDetail::new(SHOULD_HAVE_SINGLE_VALUE_ARITY, vec![])]
            ),
            _ => assert!(false),
        };
    }

    pub fn validate_content(schema: &Schema, content: &Content) -> Result<(), LightSpeedError> {
        Validator::validate(|error_details: &mut dyn ErrorDetails| {
            content.validate(schema, error_details);
            Ok(())
        })
    }
}
