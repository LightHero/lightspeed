use lightspeed_validator::Validable;
use lightspeed_validator::range::RangeError;

const MIN_AGE: i32 = 18;

#[derive(Validable)]
pub struct AgeBounds {
    #[validate(range(min = 0, max = 120))]
    pub age: i32,
    pub untouched: i32,
}

#[derive(Validable)]
pub struct Probability {
    #[validate(range(min = 0.0, max = 1.0))]
    pub p: f64,
}

#[derive(Validable)]
pub struct PositiveUnsigned {
    #[validate(range(exclusive_min = 0))]
    pub count: u32,
}

#[derive(Validable)]
pub struct HalfOpen {
    // [0, 100)
    #[validate(range(min = 0, exclusive_max = 100))]
    pub bucket: i32,
}

#[derive(Validable)]
pub struct NegativeRange {
    #[validate(range(min = -100, max = -10))]
    pub value: i32,
}

#[derive(Validable)]
pub struct ConstBounded {
    #[validate(range(min = MIN_AGE, max = 99))]
    pub age: i32,
}

fn range_err<E: From<RangeError>>(
    min: Option<&str>,
    max: Option<&str>,
    excl_min: Option<&str>,
    excl_max: Option<&str>,
) -> E {
    RangeError {
        min: min.map(str::to_string),
        max: max.map(str::to_string),
        exclusive_min: excl_min.map(str::to_string),
        exclusive_max: excl_max.map(str::to_string),
    }
    .into()
}

#[test]
fn inclusive_min_max_accepts_boundaries() {
    for ok in [0, 60, 120] {
        let v = AgeBoundsValidable::new(AgeBounds { age: ok, untouched: 0 });
        assert!(v.validate().is_ok(), "expected {ok} to be accepted");
    }
}

#[test]
fn inclusive_min_max_rejects_out_of_range_with_bounds_in_error() {
    let v = AgeBoundsValidable::new(AgeBounds { age: 121, untouched: 0 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.age.errors(), &[range_err(Some("0"), Some("120"), None, None)]);
    assert!(returned.untouched.errors().is_empty());
}

#[test]
fn float_range_works() {
    let v = ProbabilityValidable::new(Probability { p: 0.5 });
    assert!(v.validate().is_ok());

    let v = ProbabilityValidable::new(Probability { p: 1.5 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.p.errors(), &[range_err(Some("0"), Some("1"), None, None)]);
}

#[test]
fn exclusive_min_rejects_boundary() {
    let v = PositiveUnsignedValidable::new(PositiveUnsigned { count: 0 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.count.errors(), &[range_err(None, None, Some("0"), None)]);

    let v = PositiveUnsignedValidable::new(PositiveUnsigned { count: 1 });
    assert!(v.validate().is_ok());
}

#[test]
fn half_open_range_excludes_upper_bound() {
    let v = HalfOpenValidable::new(HalfOpen { bucket: 99 });
    assert!(v.validate().is_ok());

    let v = HalfOpenValidable::new(HalfOpen { bucket: 100 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.bucket.errors(), &[range_err(Some("0"), None, None, Some("100"))]);
}

#[test]
fn negative_bounds_work() {
    let v = NegativeRangeValidable::new(NegativeRange { value: -50 });
    assert!(v.validate().is_ok());

    let v = NegativeRangeValidable::new(NegativeRange { value: 5 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    assert_eq!(returned.value.errors(), &[range_err(Some("-100"), Some("-10"), None, None)]);
}

#[test]
fn const_path_can_be_used_as_a_bound() {
    let v = ConstBoundedValidable::new(ConstBounded { age: 25 });
    assert!(v.validate().is_ok());

    let v = ConstBoundedValidable::new(ConstBounded { age: 17 });
    let returned = match v.validate() {
        Ok(_) => panic!("expected Err"),
        Err(v) => v,
    };
    // The error stringifies the actual runtime value of MIN_AGE (18), not the path.
    assert_eq!(returned.age.errors(), &[range_err(Some("18"), Some("99"), None, None)]);
}

#[test]
fn macro_attaches_one_validator_per_range_attribute() {
    let v = AgeBoundsValidable::new(AgeBounds { age: 30, untouched: 0 });
    assert_eq!(v.age.validators().len(), 1);
    assert_eq!(v.untouched.validators().len(), 0);
}
