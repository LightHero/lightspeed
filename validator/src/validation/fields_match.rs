use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldsMustMatch {
    pub field_a: String,
    pub field_b: String,
}

impl Display for FieldsMustMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldsMustMatch [{}, {}]", self.field_a, self.field_b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MustMatchField {
    pub field: String,
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
        let fields_must_match = FieldsMustMatch { field_a: "a".to_string(), field_b: "b".to_string() };
        assert_eq!("FieldsMustMatch [a, b]", format!("{}", fields_must_match));

        let must_match_field = MustMatchField { field: "a".to_string() };
        assert_eq!("MustMatchField [a]", format!("{}", must_match_field));
    }
}