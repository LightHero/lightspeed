use std::fmt::Display;

use crate::FieldValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeError {
    pub min: Option<String>,
    pub max: Option<String>,
    pub exclusive_min: Option<String>,
    pub exclusive_max: Option<String>,
}

impl Display for RangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Range [")?;
        let mut first = true;
        for (label, opt) in [
            ("min", &self.min),
            ("max", &self.max),
            ("exclusive_min", &self.exclusive_min),
            ("exclusive_max", &self.exclusive_max),
        ] {
            if let Some(v) = opt {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{label}: {v}")?;
                first = false;
            }
        }
        write!(f, "]")
    }
}

/// Validates that a value falls within a configurable numeric range. Any
/// combination of inclusive/exclusive lower and upper bounds is supported;
/// each bound is optional and unset bounds are not checked. Works on any
/// `T: PartialOrd + Display`, so it composes with all standard numeric
/// primitive types.
///
/// NaN handling: `PartialOrd` for `f32` / `f64` returns `false` for every
/// comparison involving `NaN`, so a `NaN` value silently passes every bound
/// check. If you need explicit NaN rejection, add a custom validator.
pub struct RangeValidator<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub exclusive_min: Option<T>,
    pub exclusive_max: Option<T>,
}

impl<T> Default for RangeValidator<T> {
    fn default() -> Self {
        Self { min: None, max: None, exclusive_min: None, exclusive_max: None }
    }
}

impl<T, E, Ctx> FieldValidator<T, E, Ctx> for RangeValidator<T>
where
    T: PartialOrd + Display,
    E: From<RangeError>,
{
    fn validate(&self, value: &T, _context: &Ctx) -> Result<(), E> {
        let out_of_range = self.min.as_ref().is_some_and(|m| value < m)
            || self.max.as_ref().is_some_and(|m| value > m)
            || self.exclusive_min.as_ref().is_some_and(|m| value <= m)
            || self.exclusive_max.as_ref().is_some_and(|m| value >= m);
        if out_of_range {
            Err(RangeError {
                min: self.min.as_ref().map(ToString::to_string),
                max: self.max.as_ref().map(ToString::to_string),
                exclusive_min: self.exclusive_min.as_ref().map(ToString::to_string),
                exclusive_max: self.exclusive_max.as_ref().map(ToString::to_string),
            }
            .into())
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

    fn err(
        min: Option<&str>,
        max: Option<&str>,
        excl_min: Option<&str>,
        excl_max: Option<&str>,
    ) -> Result<(), ValidationError> {
        Err(ValidationError::Range(RangeError {
            min: min.map(str::to_string),
            max: max.map(str::to_string),
            exclusive_min: excl_min.map(str::to_string),
            exclusive_max: excl_max.map(str::to_string),
        }))
    }

    #[test]
    fn inclusive_min_max_accepts_boundary_values() {
        let v = RangeValidator::<i32> { min: Some(0), max: Some(100), ..Default::default() };
        assert_eq!(v.validate(&0, &()), OK, "lower bound accepted");
        assert_eq!(v.validate(&50, &()), OK);
        assert_eq!(v.validate(&100, &()), OK, "upper bound accepted");
    }

    #[test]
    fn inclusive_min_max_rejects_out_of_range_values() {
        let v = RangeValidator::<i32> { min: Some(0), max: Some(100), ..Default::default() };
        assert_eq!(v.validate(&-1, &()), err(Some("0"), Some("100"), None, None));
        assert_eq!(v.validate(&101, &()), err(Some("0"), Some("100"), None, None));
    }

    #[test]
    fn exclusive_min_rejects_boundary() {
        let v = RangeValidator::<i32> { exclusive_min: Some(0), ..Default::default() };
        assert_eq!(v.validate(&0, &()), err(None, None, Some("0"), None));
        assert_eq!(v.validate(&1, &()), OK);
        assert_eq!(v.validate(&-1, &()), err(None, None, Some("0"), None));
    }

    #[test]
    fn exclusive_max_rejects_boundary() {
        let v = RangeValidator::<i32> { exclusive_max: Some(100), ..Default::default() };
        assert_eq!(v.validate(&100, &()), err(None, None, None, Some("100")));
        assert_eq!(v.validate(&99, &()), OK);
    }

    #[test]
    fn mixed_inclusive_and_exclusive_bounds_compose() {
        // [0, 100)
        let v = RangeValidator::<i32> {
            min: Some(0),
            exclusive_max: Some(100),
            ..Default::default()
        };
        assert_eq!(v.validate(&0, &()), OK);
        assert_eq!(v.validate(&99, &()), OK);
        assert_eq!(v.validate(&100, &()), err(Some("0"), None, None, Some("100")));
    }

    #[test]
    fn works_with_floats() {
        let v = RangeValidator::<f64> { min: Some(0.0), max: Some(1.0), ..Default::default() };
        assert_eq!(v.validate(&0.0, &()), OK);
        assert_eq!(v.validate(&0.5, &()), OK);
        assert_eq!(v.validate(&1.0, &()), OK);
        assert_eq!(v.validate(&1.5, &()), err(Some("0"), Some("1"), None, None));
    }

    #[test]
    fn works_with_unsigned_and_negative() {
        let v = RangeValidator::<u32> { min: Some(10), max: Some(20), ..Default::default() };
        assert_eq!(v.validate(&15u32, &()), OK);
        assert_eq!(v.validate(&5u32, &()), err(Some("10"), Some("20"), None, None));

        let v = RangeValidator::<i32> { min: Some(-100), max: Some(-10), ..Default::default() };
        assert_eq!(v.validate(&-50, &()), OK);
        assert_eq!(v.validate(&0, &()), err(Some("-100"), Some("-10"), None, None));
    }

    #[test]
    fn no_bounds_accepts_every_value() {
        let v = RangeValidator::<i32>::default();
        assert_eq!(v.validate(&0, &()), OK);
        assert_eq!(v.validate(&i32::MAX, &()), OK);
        assert_eq!(v.validate(&i32::MIN, &()), OK);
    }

    #[test]
    fn range_error_display_only_shows_set_bounds() {
        let e = RangeError {
            min: Some("0".to_string()),
            max: Some("100".to_string()),
            exclusive_min: None,
            exclusive_max: None,
        };
        assert_eq!(e.to_string(), "Range [min: 0, max: 100]");

        let e = RangeError {
            min: None,
            max: None,
            exclusive_min: Some("0".to_string()),
            exclusive_max: Some("10".to_string()),
        };
        assert_eq!(e.to_string(), "Range [exclusive_min: 0, exclusive_max: 10]");
    }
}
