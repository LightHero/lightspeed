use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct FieldsMustMatch {
    pub field_a: &'static str,
    pub field_b: &'static str,
}

impl Display for FieldsMustMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldsMustMatch [{}, {}]", self.field_a, self.field_b)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub struct MustMatchField {
    /// See [`FieldsMustMatch::field_a`].
    pub field: &'static str,
}

impl Display for MustMatchField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MustMatchField [{}]", self.field)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fields_must_match_fmt() {
        let fields_must_match = FieldsMustMatch { field_a: "a", field_b: "b" };
        assert_eq!("FieldsMustMatch [a, b]", format!("{}", fields_must_match));

        let must_match_field = MustMatchField { field: "a" };
        assert_eq!("MustMatchField [a]", format!("{}", must_match_field));
    }
}
