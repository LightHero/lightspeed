use std::collections::{BTreeMap, HashMap};

use lightspeed_validator::Validable;
use lightspeed_validator::length::LengthError;

const MAX_TAGS: usize = 5;

#[derive(Validable)]
pub struct Username {
    #[validate(length(min = 3, max = 20))]
    pub name: String,
    pub untouched: String,
}

#[derive(Validable)]
pub struct ExactCode {
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Validable)]
pub struct Tags {
    #[validate(length(min = 1, max = MAX_TAGS))]
    pub tags: Vec<String>,
}

#[derive(Validable)]
pub struct Settings {
    #[validate(length(min = 1))]
    pub options: HashMap<String, String>,
    #[validate(length(max = 3))]
    pub aliases: BTreeMap<String, String>,
}

fn length_err<E: From<LengthError>>(actual: usize, min: Option<usize>, max: Option<usize>, equal: Option<usize>) -> E {
    LengthError { min, max, equal, actual }.into()
}

#[test]
fn string_length_uses_unicode_scalar_count() {
    // "café" — 4 chars (é precomposed), 5 bytes. Should pass `max = 20`.
    let v = UsernameValidable::new(Username { name: "café".to_string(), untouched: String::new() });
    assert!(v.validate().is_ok());
}

#[test]
fn string_min_rejects_too_short_with_actual_length_in_error() {
    let v = UsernameValidable::new(Username { name: "ab".to_string(), untouched: String::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.name.errors(), &[length_err(2, Some(3), Some(20), None)]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn string_max_rejects_too_long_with_actual_length_in_error() {
    let too_long = "a".repeat(21);
    let v = UsernameValidable::new(Username { name: too_long, untouched: String::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.name.errors(), &[length_err(21, Some(3), Some(20), None)]);
}

#[test]
fn exact_length_requires_match() {
    let v = ExactCodeValidable::new(ExactCode { code: "ABC123".to_string() });
    assert!(v.validate().is_ok());

    let v = ExactCodeValidable::new(ExactCode { code: "AB".to_string() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.code.errors(), &[length_err(2, None, None, Some(6))]);
}

#[test]
fn works_on_vec_with_const_bound() {
    let v = TagsValidable::new(Tags { tags: vec!["rust".into(), "macros".into()] });
    assert!(v.validate().is_ok());

    let v = TagsValidable::new(Tags { tags: Vec::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.tags.errors(), &[length_err(0, Some(1), Some(MAX_TAGS), None)]);

    let v = TagsValidable::new(Tags { tags: (0..6).map(|i| format!("tag{i}")).collect() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.tags.errors(), &[length_err(6, Some(1), Some(MAX_TAGS), None)]);
}

#[test]
fn works_on_hashmap_and_btreemap() {
    let mut opts = HashMap::new();
    opts.insert("a".to_string(), "1".to_string());
    let aliases: BTreeMap<String, String> = [("k", "v")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    let v = SettingsValidable::new(Settings { options: opts, aliases });
    assert!(v.validate().is_ok());
}

#[test]
fn empty_hashmap_fails_min_length() {
    let v = SettingsValidable::new(Settings { options: HashMap::new(), aliases: BTreeMap::new() });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.options.errors(), &[length_err(0, Some(1), None, None)]);
    assert!(returned.aliases.errors().is_empty(), "aliases has no min bound so empty is fine");
}

#[test]
fn btreemap_max_rejects_when_too_large() {
    let aliases: BTreeMap<String, String> = (0..4).map(|i| (format!("k{i}"), format!("v{i}"))).collect();
    let mut opts = HashMap::new();
    opts.insert("a".to_string(), "b".to_string());
    let v = SettingsValidable::new(Settings { options: opts, aliases });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert!(returned.options.errors().is_empty());
    assert_eq!(returned.aliases.errors(), &[length_err(4, None, Some(3), None)]);
}

#[test]
fn macro_attaches_one_validator_per_length_attribute() {
    let v = UsernameValidable::new(Username { name: "abcde".to_string(), untouched: String::new() });
    assert_eq!(v.name.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
