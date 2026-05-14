//! Runtime helper for the `#[validate(custom(function = "..."))]` macro.
//!
//! The macro emits a call to [`boxed_fn`] for each custom validator. Passing
//! the user function in argument position is what triggers the
//! function-item-to-fn-pointer coercion — the macro therefore never has to
//! emit an `as fn(...)` cast with `_` placeholders.
//!
//! The blanket `FieldValidator<T, E, CTX> for fn(&T, &CTX) -> Result<(), E>`
//! impl in [`crate::validation`] is what makes the boxed function pointer
//! usable as a runtime validator.

use crate::{FieldValidator, ValidatorRef};

/// Boxes `f` into a [`ValidatorRef::Boxed`]. The type parameters are
/// inferred from the function pointer at the call site, which means the
/// user function's signature has to line up with the field's `T`, the
/// struct's `Ctx`, and the field error type picked by the `errors(...)`
/// strategy — any mismatch surfaces as a normal type error pointing at the
/// user's function.
pub fn boxed_fn<T: 'static, E: 'static, CTX: 'static>(
    f: fn(&T, &CTX) -> Result<(), E>,
) -> ValidatorRef<T, E, CTX> {
    let boxed: Box<dyn FieldValidator<T, E, CTX>> = Box::new(f);
    ValidatorRef::Boxed(boxed)
}
