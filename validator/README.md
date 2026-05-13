# lightspeed_validator

A small, derive-driven validator for Rust structs. Annotate the fields and the
struct itself with `#[validate(...)]`, and the macro generates a companion
`<Name>Validable` type that runs the validators on demand.

Inspired by the [`validator`](https://github.com/Keats/validator) crate, but
with a different shape: validation produces a parallel `Validable` value that
either yields back the original struct on success or exposes per-field and
top-level error vecs on failure.

## Installation

```toml
[dependencies]
lightspeed_validator = "<latest_version>"
```

## A short example

```rust,ignore
use lightspeed_validator::{Validable, ValidationError};

#[derive(Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
struct Signup {
    #[validate(contains(pattern = "@"))]
    email: String,
    #[validate(not_contains(pattern = "password", case_sensitive = false))]
    password: String,
    password_confirm: String,
    #[validate(isTrue)]
    accepted_tos: bool,
}

let signup = Signup {
    email: "user@example.com".to_string(),
    password: "hunter2".to_string(),
    password_confirm: "hunter2".to_string(),
    accepted_tos: true,
};

match SignupValidable::new(signup).validate() {
    Ok(signup) => { /* validated, original value handed back */ }
    Err(validable) => {
        // Inspect what went wrong:
        let _email_errors = validable.email.errors();
        let _password_errors = validable.password.errors();
        let _struct_errors = validable.top_level_errors();
    }
}
```

## How it works

`#[derive(Validable)]` on a struct `Foo` emits a sibling `FooValidable` whose
fields are wrapped in `ValidableType<T, Ctx>`. The companion type exposes:

- `FooValidable::new(value: Foo) -> Self` — wraps an instance together with
  the validator list declared by the field-level attributes.
- `FooValidable::validate(self) -> Result<Foo, Self>` — runs every field's
  validators and every struct-level rule. On success the original `Foo` is
  returned; on failure the validable is returned with errors populated on the
  relevant fields and/or on its top-level `errors` vec.
- `FooValidable::top_level_errors(&self) -> &[ValidationError]` — errors
  produced by struct-level rules that were not attached to a specific field.
- Per-field accessors: `validable.field.get() -> &T`, `validable.field.set(v)`,
  `validable.field.errors() -> &[ValidationError]`.

Validation never panics and never mutates anything visible until you call
`validate`. The validator list is built once in `new` and reused on every
subsequent `validate` call.

### Custom validation context

By default every validator receives `&()` as its context. To thread your own
context through, declare it on the struct and pass it to `validate`:

```rust,ignore
#[derive(Validable)]
#[validate(context = MyCtx)]
struct Foo { /* ... */ }

let ctx = MyCtx { /* ... */ };
let result = FooValidable::new(foo).validate(&ctx);
```

The context type is forwarded to every validator's `validate(value, ctx)`
call, so you can write custom field validators that read it.

## Validators

### isTrue / isFalse

For `bool` fields. Each requires the value be respectively `true` or `false`.

```rust,ignore
#[validate(isTrue)]
accepted_tos: bool,

#[validate(isFalse)]
banned: bool,
```

Errors: `ValidationError::MustBeTrue(MustBeTrueError)` /
`ValidationError::MustBeFalse(MustBeFalseError)`.

### contains

Requires the field's value to contain a given substring. Works on any
string-compatible type — `String`, `&str`, `Cow<'_, str>`, `Box<str>`,
`Rc<str>`, `Arc<str>`.

`case_sensitive` is optional; it defaults to `true`.

```rust,ignore
#[validate(contains(pattern = "@"))]
email: String,

#[validate(contains(pattern = "Hello", case_sensitive = false))]
greeting: String,
```

Error: `ValidationError::MustContain(MustContainError { pattern, case_sensitive })`.

### not_contains

The complement of `contains`. Requires the value to NOT contain the pattern.

```rust,ignore
#[validate(not_contains(pattern = "spam"))]
subject: String,

#[validate(not_contains(pattern = "password", case_sensitive = false))]
password: String,
```

Error: `ValidationError::MustNotContain(MustNotContainError { pattern, case_sensitive })`.

### ip / ipv4 / ipv6

Requires the field's value to parse as an IP address. Works on the same
string-compatible types as `contains`. All three keywords map to the same
`IpValidator`, distinguished only by the kind it carries:

- `ip` — any IP (v4 or v6);
- `ipv4` — must parse and be an IPv4 address;
- `ipv6` — must parse and be an IPv6 address.

```rust,ignore
#[validate(ip)]
remote: String,

#[validate(ipv4)]
gateway: String,

#[validate(ipv6)]
link_local: String,
```

Error: `ValidationError::Ip(IpError { kind })`, where `kind` mirrors which
form was requested (`IpKind::Any` / `IpKind::V4` / `IpKind::V6`).

### url

Requires the field's value to parse as an absolute URL via the
[`url`](https://docs.rs/url) crate. Works on the same string-compatible types
as `contains`. Relative paths and missing-scheme inputs are rejected.

```rust,ignore
#[validate(url)]
homepage: String,
```

Error: `ValidationError::Url(UrlError)` (unit-struct payload — failure means
the value did not parse as an absolute URL).

### Multiple validators on the same field

Field attributes are additive — you can either repeat the attribute or
combine them in a single one:

```rust,ignore
#[validate(contains(pattern = "@"))]
#[validate(not_contains(pattern = " "))]
email: String,

#[validate(contains(pattern = "@"), not_contains(pattern = " "))]
email_short_form: String,
```

Each validator runs in declaration order; errors from every failing validator
are collected onto the field's `errors()` vec.

## Struct-level validation

Some rules cross field boundaries. They are declared on the struct with the
same `#[validate(...)]` attribute syntax.

### fields_match

Requires two fields to compare equal (via `PartialEq`). The error routing is
controlled by the optional `attach_to_fields` flag:

- `attach_to_fields = false` (default): a single
  `ValidationError::FieldsMustMatch(FieldsMustMatch { field_a, field_b })`
  is pushed onto the validable's top-level `errors` vec.
- `attach_to_fields = true`: a `ValidationError::MustMatchField(MustMatchField { field })`
  is pushed onto each of the two fields' `errors`. Each field's error names
  the *other* field — so `field_a` gets `MustMatchField { field: "field_b" }`
  and vice-versa.

```rust,ignore
// Top-level routing (default)
#[derive(Validable)]
#[validate(fields_match(password, password_confirm))]
struct Signup {
    password: String,
    password_confirm: String,
}

// Per-field routing
#[derive(Validable)]
#[validate(fields_match(password, password_confirm, attach_to_fields = true))]
struct SignupAttach {
    password: String,
    password_confirm: String,
}
```

Both named fields must exist on the struct; unknown names produce a
compile-time error pointing at the bad identifier. Multiple `fields_match`
rules can be declared on the same struct.

## Error types

All errors flow through the `ValidationError` enum. The current variants are:

```rust,ignore
pub enum ValidationError {
    MustBeTrue(MustBeTrueError),
    MustBeFalse(MustBeFalseError),
    MustContain(MustContainError),
    MustNotContain(MustNotContainError),
    MustBeGreater { min: usize },
    FieldsMustMatch(FieldsMustMatch),
    MustMatchField(MustMatchField),
    Ip(IpError),
    Url(UrlError),
}
```

`ValidationError`, every inner `*Error` struct, and the `FieldsMustMatch` /
`MustMatchField` payload structs all derive `Debug`, `Clone`, `PartialEq`,
`Eq`, and `Display`.

## Writing a custom field validator

Anything implementing `FieldValidator<T, ValidationError, Ctx>` can be used
manually, without the derive macro:

```rust,ignore
use lightspeed_validator::{FieldValidator, ValidableType, ValidationError};

struct MustBeGreaterValidator { min: usize }

impl FieldValidator<usize, ValidationError, ()> for MustBeGreaterValidator {
    fn validate(&self, value: &usize, _ctx: &()) -> Result<(), ValidationError> {
        if *value > self.min {
            Ok(())
        } else {
            Err(ValidationError::MustBeGreater { min: self.min })
        }
    }
}

let mut field: ValidableType<usize> =
    ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 })]);
field.validate(&());
assert!(!field.errors().is_empty());
```

The `StructValidator<T, ValidationError, Ctx>` trait plays the same role for
rules that need access to the whole validable struct after the field pass.
