use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Display;

use thiserror::Error;

use crate::FieldValidator;

/// Implemented for every type that the [`LengthValidator`] knows how to
/// measure. The crate provides impls for the standard string-like types
/// (`str`, `String`, `Cow<'_, str>`, `Box<str>`) and the standard
/// collections (`Vec`, `VecDeque`, slice, `HashMap`, `BTreeMap`, `HashSet`,
/// `BTreeSet`), plus a blanket `&T: HasLength` impl. Downstream crates can
/// add impls for their own types.
///
/// ## What "length" means for strings
///
/// For string-like types this is `chars().count()` — the number of Unicode
/// scalar values, **not** the number of bytes and **not** the number of
/// "visual characters".
pub trait HasLength {
    fn length(&self) -> usize;
}

impl HasLength for str {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl HasLength for String {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl HasLength for Cow<'_, str> {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl HasLength for Box<str> {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl<T: HasLength + ?Sized> HasLength for &T {
    fn length(&self) -> usize {
        (**self).length()
    }
}

impl<T> HasLength for [T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for VecDeque<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<K, V, S> HasLength for HashMap<K, V, S> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLength for BTreeMap<K, V> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, S> HasLength for HashSet<T, S> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for BTreeSet<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct LengthError {
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub equal: Option<usize>,
    pub actual: usize,
}

impl Display for LengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Length [actual: {}", self.actual)?;
        if let Some(m) = self.min {
            write!(f, ", min: {m}")?;
        }
        if let Some(m) = self.max {
            write!(f, ", max: {m}")?;
        }
        if let Some(m) = self.equal {
            write!(f, ", equal: {m}")?;
        }
        write!(f, "]")
    }
}

/// Validates that a value's "length" (via [`HasLength`]) falls within the
/// configured bounds. `min` / `max` may be combined; `equal` should be used on
/// its own. At least one bound must be set (enforced by the macro; manual
/// construction of `LengthValidator::default()` simply accepts every value).
#[derive(Default)]
pub struct LengthValidator {
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub equal: Option<usize>,
}

impl<T, E, Ctx> FieldValidator<T, E, Ctx> for LengthValidator
where
    T: HasLength,
    E: From<LengthError>,
{
    fn validate(&self, value: &T, _context: &Ctx) -> Result<(), E> {
        let actual = value.length();
        let out_of_range = self.min.is_some_and(|m| actual < m)
            || self.max.is_some_and(|m| actual > m)
            || self.equal.is_some_and(|m| actual != m);
        if out_of_range {
            Err(LengthError { min: self.min, max: self.max, equal: self.equal, actual }.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ValidationError;

    const OK: Result<(), ValidationError> = Ok(());

    fn err(actual: usize, min: Option<usize>, max: Option<usize>, equal: Option<usize>) -> Result<(), ValidationError> {
        Err(ValidationError::Length(LengthError { min, max, equal, actual }))
    }

    #[test]
    fn min_accepts_at_or_above_bound() {
        let v = LengthValidator { min: Some(3), ..Default::default() };
        assert_eq!(v.validate(&"abc", &()), OK);
        assert_eq!(v.validate(&"abcd", &()), OK);
        assert_eq!(v.validate(&"ab", &()), err(2, Some(3), None, None));
    }

    #[test]
    fn max_accepts_at_or_below_bound() {
        let v = LengthValidator { max: Some(3), ..Default::default() };
        assert_eq!(v.validate(&"", &()), OK);
        assert_eq!(v.validate(&"abc", &()), OK);
        assert_eq!(v.validate(&"abcd", &()), err(4, None, Some(3), None));
    }

    #[test]
    fn equal_requires_exact_length() {
        let v = LengthValidator { equal: Some(3), ..Default::default() };
        assert_eq!(v.validate(&"abc", &()), OK);
        assert_eq!(v.validate(&"ab", &()), err(2, None, None, Some(3)));
        assert_eq!(v.validate(&"abcd", &()), err(4, None, None, Some(3)));
    }

    #[test]
    fn min_and_max_compose() {
        let v = LengthValidator { min: Some(3), max: Some(5), ..Default::default() };
        assert_eq!(v.validate(&"abc", &()), OK);
        assert_eq!(v.validate(&"abcd", &()), OK);
        assert_eq!(v.validate(&"abcde", &()), OK);
        assert_eq!(v.validate(&"ab", &()), err(2, Some(3), Some(5), None));
        assert_eq!(v.validate(&"abcdef", &()), err(6, Some(3), Some(5), None));
    }

    #[test]
    fn string_length_counts_unicode_scalar_values_not_bytes() {
        // "café" is 4 chars but 5 bytes (é is 2 bytes in UTF-8).
        assert_eq!("café".len(), 5);
        let v = LengthValidator { equal: Some(4), ..Default::default() };
        assert_eq!(v.validate(&"café".to_string(), &()), OK);
    }

    #[test]
    fn string_length_differs_from_visual_chars_for_combining_sequences() {
        // "é" written as base + combining acute (U+0065 U+0301) is two chars
        // even though it looks like one visual character.
        let combining = "e\u{0301}".to_string();
        assert_eq!(combining.chars().count(), 2);
        let v = LengthValidator { equal: Some(2), ..Default::default() };
        assert_eq!(v.validate(&combining, &()), OK);
    }

    #[test]
    fn works_on_vec() {
        let v = LengthValidator { min: Some(1), max: Some(3), ..Default::default() };
        assert_eq!(v.validate(&vec![1, 2], &()), OK);
        assert_eq!(v.validate(&Vec::<i32>::new(), &()), err(0, Some(1), Some(3), None));
        assert_eq!(v.validate(&vec![1, 2, 3, 4], &()), err(4, Some(1), Some(3), None));
    }

    #[test]
    fn works_on_hashmap_and_btreemap() {
        let v = LengthValidator { min: Some(2), ..Default::default() };

        let mut hm: HashMap<&str, i32> = HashMap::new();
        hm.insert("a", 1);
        assert_eq!(v.validate(&hm, &()), err(1, Some(2), None, None));
        hm.insert("b", 2);
        assert_eq!(v.validate(&hm, &()), OK);

        let bm: BTreeMap<i32, i32> = [(1, 1), (2, 2), (3, 3)].into_iter().collect();
        assert_eq!(v.validate(&bm, &()), OK);
    }

    #[test]
    fn works_on_sets() {
        let v = LengthValidator { equal: Some(3), ..Default::default() };
        let hs: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert_eq!(v.validate(&hs, &()), OK);
        let bs: BTreeSet<i32> = [1, 2].into_iter().collect();
        assert_eq!(v.validate(&bs, &()), err(2, None, None, Some(3)));
    }

    #[test]
    fn works_on_cow_str() {
        let v = LengthValidator { min: Some(2), max: Some(5), ..Default::default() };
        let cow: Cow<'_, str> = Cow::Borrowed("abc");
        assert_eq!(v.validate(&cow, &()), OK);
    }

    #[test]
    fn length_error_display_only_shows_set_bounds() {
        let e = LengthError { min: Some(2), max: Some(10), equal: None, actual: 1 };
        assert_eq!(e.to_string(), "Length [actual: 1, min: 2, max: 10]");

        let e = LengthError { min: None, max: None, equal: Some(4), actual: 7 };
        assert_eq!(e.to_string(), "Length [actual: 7, equal: 4]");
    }
}
