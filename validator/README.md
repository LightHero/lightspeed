# lightspeed_validator

A small, derive-driven validator for Rust structs. Annotate the fields and the
struct itself with `#[validate(...)]`, and the macro generates a companion
`<Name>Validable` type that runs the validators on demand.

Inspired by the [`validator`](https://github.com/Keats/validator) crate, but
with a different shape: validation produces a parallel `Validable` value that
either yields back the original struct on success or exposes per-field and
top-level error vecs on failure.

## Three error-type strategies, one switch

Field-level error types are controlled by the struct-level attribute
`#[validate(errors(<mode>))]`. Pick the trade-off that fits the consumer
of `field.errors()`:

| Mode | `errors(...)` | What `field.errors()` returns | When to use |
|---|---|---|---|
| **Shared** *(default)* | `errors(shared)` or omitted | `&[ValidationError]` on every field | Quick start, logging, JSON-style aggregation — one homogeneous error type across the whole struct. |
| **Tailored** | `errors(tailored)` | A per-field `<Struct><Field>FieldError` enum carrying only the variants the field can produce (`NoError` for no-validator fields) | UI / form renderers and any code that wants the compiler to keep `match`es in sync with the validation rules. |
| **Custom** | `errors(custom = <Type>)` | `&[<Type>]` on every field, where `<Type>` is your own enum | You already have an app-wide error type and want validation failures to fit into it. `<Type>` must `impl From<NarrowError>` for every narrow error your validators emit. |

### `errors(tailored)` — exhaustive per-field matching

The exhaustive-matching mode is what makes this crate different from the
usual one-big-enum design. **The macro generates a dedicated error enum
per field containing *only* the variants that field's validators can
actually produce** (duplicates collapsed). That enum becomes the field's
`E` in `ValidableType<T, E, Ctx>`, so a `match` on `field.errors()` is
exhaustively checked against that field's true error set — no `_ =>`
wildcards, no dead arms. Adding or removing a `#[validate(...)]` attribute
forces every consumer to acknowledge the shape change at compile time.

Fields with **no** validators get `NoError`, an uninhabited enum: their
error vec is always empty, and `match err {}` (no arms) is exhaustive.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
#[validate(errors(tailored))]
pub struct MatchOnValidator {
    pub zero_validators: String,

    #[validate(contains(pattern = "@"))]
    pub one_validator: String,

    #[validate(contains(pattern = "secret"))]
    #[validate(password)]
    #[validate(length(min = 3, max = 20))]
    pub three_validators: String,
}

let v = MatchOnValidatorValidable::new(MatchOnValidator {
    zero_validators: String::new(),
    one_validator: String::new(),
    three_validators: String::new(),
});

// No `#[validate(...)]` on the field → `ValidableType<String, NoError, _>`.
// `NoError` has no variants; the loop body is statically unreachable.
for _err in v.zero_validators.errors() {
    // unreachable
}

// One validator → the enum has exactly one variant.
for err in v.one_validator.errors() {
    match err {
        MatchOnValidatorOneValidatorFieldError::MustContain(_) => { /* … */ }
    }
}

// Three validators → exactly the three variants you wrote, no others.
// Add a fourth `#[validate(...)]` and this match stops compiling until
// you handle the new arm. Remove one and the dead arm errors out.
for err in v.three_validators.errors() {
    match err {
        MatchOnValidatorThreeValidatorsFieldError::MustContain(_) => { /* … */ }
        MatchOnValidatorThreeValidatorsFieldError::Password(_) => { /* … */ }
        MatchOnValidatorThreeValidatorsFieldError::Length(_) => { /* … */ }
    }
}
```

This is the core ergonomics win: **the compiler keeps your error-handling
code in sync with your validation rules.** UI layers, JSON-API responders,
form-error renderers — anything that consumes `field.errors()` — fail to
build the moment the validation rules drift out from under them, instead
of silently dropping new failure modes on the floor.

When you do need a single uniform error type (logging, persisting,
cross-field aggregation), each per-field enum gets a generated
`From<…FieldError> for ValidationError` impl, so `.into()` lifts back to
the wide type whenever you want it.

### `errors(custom = MyError)` — plug in your own error type

```rust,no_run
use lightspeed_validator::Validable;
use lightspeed_validator::contains::MustContainError;
use lightspeed_validator::length::LengthError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignupError {
    BadEmail(MustContainError),
    BadLength(LengthError),
}

impl From<MustContainError> for SignupError {
    fn from(e: MustContainError) -> Self { Self::BadEmail(e) }
}
impl From<LengthError> for SignupError {
    fn from(e: LengthError) -> Self { Self::BadLength(e) }
}

#[derive(Validable)]
#[validate(errors(custom = SignupError))]
pub struct Signup {
    #[validate(contains(pattern = "@"))]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
```

Every field's `ValidableType` is now `ValidableType<…, SignupError, …>`. If
you add a validator emitting a narrow error `MyError` doesn't `impl From`
for, you get a normal `From` trait error at compile time — pointing at the
exact validator construction site.

## Installation

```toml
[dependencies]
lightspeed_validator = "<latest_version>"
```

## A short example

```rust,no_run
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

```rust,no_run
use lightspeed_validator::Validable;

pub struct MyCtx;

#[derive(Validable)]
#[validate(context = MyCtx)]
struct Foo {
    // ... fields ...
}

let foo = Foo {};
let ctx = MyCtx;
let _result = FooValidable::new(foo).validate(&ctx);
```

The context type is forwarded to every validator's `validate(value, ctx)`
call, so you can write custom field validators that read it.

### Per-field error enums — generation rules

Only emitted when `#[validate(errors(tailored))]` is set on the struct.
The macro then emits, in addition to the usual `<Name>Validable`:

- one `pub enum <Struct><FieldPascalCase>FieldError` per field that has at
  least one `#[validate(...)]` attribute *or* is targeted by an
  `attach_to_fields = true` struct rule. Variants mirror the
  `ValidationError` variant names (`MustContain`, `Range`, `Length`, …),
  one per *unique* error type — duplicates from multiple validators on the
  same field (e.g. two `regex(...)` attributes) collapse into a single
  variant;
- `From<NarrowError> for <FieldError>` impls — one per variant — so each
  validator's narrow error lifts into the field's enum automatically;
- `From<<FieldError>> for ValidationError` — lift back to the wide type
  with `.into()` when you need a single uniform error type;
- for fields with no validators and no struct-rule targeting:
  `ValidableType<T, NoError, Ctx>`. `NoError` is uninhabited, so the
  errors vec is always empty and `match err {}` (no arms) is exhaustive.

## Validators

### isTrue / isFalse

For `bool` fields. Each requires the value be respectively `true` or `false`.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(isTrue)]
    accepted_tos: bool,

    #[validate(isFalse)]
    banned: bool,
}
```

Errors: `ValidationError::MustBeTrue(MustBeTrueError)` /
`ValidationError::MustBeFalse(MustBeFalseError)`.

### contains

Requires the field's value to contain a given substring. Works on any
string-compatible type — `String`, `&str`, `Cow<'_, str>`, `Box<str>`,
`Rc<str>`, `Arc<str>`.

`case_sensitive` is optional; it defaults to `true`.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(contains(pattern = "@"))]
    email: String,

    #[validate(contains(pattern = "Hello", case_sensitive = false))]
    greeting: String,
}
```

Error: `ValidationError::MustContain(MustContainError { pattern, case_sensitive })`.

### not_contains

The complement of `contains`. Requires the value to NOT contain the pattern.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(not_contains(pattern = "spam"))]
    subject: String,

    #[validate(not_contains(pattern = "password", case_sensitive = false))]
    password: String,
}
```

Error: `ValidationError::MustNotContain(MustNotContainError { pattern, case_sensitive })`.

### ip / ipv4 / ipv6

Requires the field's value to parse as an IP address. Works on the same
string-compatible types as `contains`. All three keywords map to the same
`IpValidator`, distinguished only by the kind it carries:

- `ip` — any IP (v4 or v6);
- `ipv4` — must parse and be an IPv4 address;
- `ipv6` — must parse and be an IPv6 address.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(ip)]
    remote: String,

    #[validate(ipv4)]
    gateway: String,

    #[validate(ipv6)]
    link_local: String,
}
```

Error: `ValidationError::Ip(IpError { kind })`, where `kind` mirrors which
form was requested (`IpKind::Any` / `IpKind::V4` / `IpKind::V6`).

### url

Requires the field's value to parse as an absolute URL via the
[`url`](https://docs.rs/url) crate. Works on the same string-compatible types
as `contains`. Relative paths and missing-scheme inputs are rejected.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(url)]
    homepage: String,
}
```

Error: `ValidationError::Url(UrlError)` (unit-struct payload — failure means
the value did not parse as an absolute URL).

### email

Requires the field's value to parse as an email address via the
[`email_address`](https://docs.rs/email_address) crate (RFC 5321 / 5322
shape). The check is **syntactic only** — no DNS lookup, no
mailbox-reachability ping, no accept-list. Works on the same
string-compatible types as `contains`.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(email)]
    contact: String,
}
```

Error: `ValidationError::Email(EmailError)` (unit-struct payload — failure
means the value did not parse as an email address).

### range

Checks that a numeric value falls within the configured bounds. All four
bounds — `min`, `max`, `exclusive_min`, `exclusive_max` — are optional, and
any combination may be supplied. At least one must be provided.

Works on any field whose type is `PartialOrd + Display`, which covers all
the integer (`i8`…`i128`, `u8`…`u128`, `isize`, `usize`) and float
(`f32`, `f64`) primitives. Bounds may be any Rust expression — literals,
named constants, etc. — and their types are checked against the field's
type by the compiler.

```rust,no_run
use lightspeed_validator::Validable;

const MIN_AGE: i32 = 18;

#[derive(Validable)]
struct Example {
    #[validate(range(min = 0, max = 120))]
    age: i32,

    #[validate(range(min = 0.0, max = 1.0))]
    probability: f64,

    #[validate(range(exclusive_min = 0))]
    positive_count: u32,

    // Half-open [0, 100)
    #[validate(range(min = 0, exclusive_max = 100))]
    bucket: i32,

    // Bound from a `const`
    #[validate(range(min = MIN_AGE, max = 99))]
    adult_age: i32,
}
```

Error: `ValidationError::Range(RangeError { min, max, exclusive_min, exclusive_max })`
where each field is `Option<String>` carrying the bound's `Display`-formatted
value (or `None` if that side wasn't configured). The `Display` impl on
`RangeError` only shows the bounds that were set.

NaN values silently pass every bound check, because Rust's `PartialOrd` for
floats returns `false` for any comparison involving `NaN`. Add a custom
validator if you need explicit NaN rejection.

### length

Checks the length / size of a string-like value or a collection. Options
(all `usize`-coerced expressions, all optional, at least one required):

- `min` — required minimum length (inclusive);
- `max` — required maximum length (inclusive);
- `equal` — required exact length. Mutually exclusive with `min` / `max`.

Works on any field whose type implements the runtime `HasLength` trait. The
crate provides impls for:

- string types: `String`, `&str`, `Cow<'_, str>`, `Box<str>`;
- collections: `Vec`, `VecDeque`, slice `[T]`, `HashMap`, `BTreeMap`,
  `HashSet`, `BTreeSet`;
- plus a blanket `impl<T: HasLength + ?Sized> HasLength for &T` so any
  reference to a length-having type works too.

Downstream crates can add impls for their own types.

```rust,no_run
use lightspeed_validator::Validable;

const MAX_TAGS: usize = 5;

#[derive(Validable)]
struct Example {
    #[validate(length(min = 3, max = 20))]
    username: String,

    #[validate(length(equal = 6))]
    otp_code: String,

    #[validate(length(min = 1, max = MAX_TAGS))]
    tags: Vec<String>,

    #[validate(length(max = 100))]
    settings: std::collections::HashMap<String, String>,
}
```

**String length is `chars().count()`** — the number of Unicode scalar
values, not the number of bytes and not the number of *visual characters*.
A grapheme cluster (e.g. base letter + combining accent, or a multi-codepoint
emoji sequence) can span more than one `char`, so an input the user perceives
as a single character may count as several. If you need grapheme-cluster
counting, use a crate like `unicode-segmentation` and add a custom
`HasLength` impl for your wrapper type.

Error: `ValidationError::Length(LengthError { min, max, equal, actual })`,
where each bound is `Option<usize>` carrying the value that was configured
(or `None` if that side wasn't set) and `actual` is the measured length.

### regex

Checks a string-compatible field against a regular expression via
[`Regex::is_match`](https://docs.rs/regex). The validator holds a
`&'static Regex`, so the regex is compiled once and reused across every
`validate` call.

Two forms are supported, exactly one of which must be provided:

- **`path = <expr>`** — the expression should evaluate to
  `&'static ::regex::Regex`. The caller controls how the regex is held —
  typically with `LazyLock<Regex>`, `OnceLock<Regex>`, or `lazy_static!`:

  ```rust,no_run
  use std::sync::LazyLock;
  use regex::Regex;
  use lightspeed_validator::Validable;

  static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap()
  });

  #[derive(Validable)]
  struct Form {
      #[validate(regex(path = &EMAIL_RE))]
      email: String,
  }
  ```

- **`pattern = "..."`** — a string literal. The macro lifts the pattern
  into a per-call-site `static OnceLock<Regex>` and initializes it on the
  first `validate` call (subsequent calls reuse the cached regex). The
  `OnceLock` is scoped to the generated `Box::new(...)` block so multiple
  fields don't collide:

  ```rust,no_run
  use lightspeed_validator::Validable;

  #[derive(Validable)]
  struct Form {
      #[validate(regex(pattern = r"^\d{3}-\d{4}$"))]
      phone_local: String,
  }
  ```

  An invalid pattern panics at first validation with
  `"invalid regex pattern: <pattern>"`.

Error: `ValidationError::Regex(RegexError { pattern: String })`. The
`pattern` field carries the regex source text (`Regex::as_str()`).

### password

Checks a string against a configurable password policy. Works on the same
string-compatible types as `contains`. All the options are optional and have
sensible defaults — bare `#[validate(password)]` gives an OWASP-style policy.

Options (all may be omitted):

- `upper` (`bool`, default `true`) — require at least one ASCII uppercase letter.
- `lower` (`bool`, default `true`) — require at least one ASCII lowercase letter.
- `number` (`bool`, default `true`) — require at least one ASCII digit.
- `special_char` (`bool` *or* string literal, default `true`) — controls
  the special-character requirement:
  - `true`: require one char from the recommended default list
    (`DEFAULT_SPECIAL_CHARS` — the printable-ASCII non-alphanumeric,
    non-space set);
  - `false`: disable the special-character check entirely;
  - `"..."`: require one char from the *provided* set (each char in the
    string literal is an allowed special char).
- `trailing_whitespaces` (`bool`, default `false`) — when `false` the
  password must not end in a whitespace character. Set to `true` to allow
  trailing whitespace.

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    // Default OWASP-style policy
    #[validate(password)]
    strong: String,

    // Relax the rules
    #[validate(password(upper = false, number = false, special_char = false))]
    loose: String,

    // Custom allowed special chars
    #[validate(password(special_char = "*$"))]
    star_or_dollar: String,
}
```

All policy violations from a single value are aggregated into one error:
`ValidationError::Password(PasswordError { violations: Vec<PasswordViolation> })`
where `PasswordViolation` is `MissingUppercase` / `MissingLowercase` /
`MissingNumber` / `MissingSpecialChar` / `HasTrailingWhitespace`.

### credit_card

*Requires the `credit_card` feature (off by default).*

Requires the field's value to be recognized as a credit card number. Spaces
and dashes are stripped, then the cleaned input is passed to the
[`card-validate`](https://docs.rs/card-validate) crate, which runs:

- Luhn checksum,
- brand-specific length checks (e.g. Amex must be 15 digits, MasterCard 16),
- IIN range matching for the major issuers (Visa, MasterCard, Amex,
  Discover, Diners Club, JCB, UnionPay, MIR, Maestro, Dankort, Visa Electron,
  Forbrugsforeningen).

Numbers that pass Luhn but don't match any known issuer prefix are rejected.

```rust,no_run
use lightspeed_validator::Validable;

#[cfg(feature = "credit_card")]
#[derive(Validable)]
struct Example {
    #[validate(credit_card)]
    card_number: String,
}
```

Error: `ValidationError::CreditCard(CreditCardError)` (unit-struct payload).

To use this validator, enable the feature:

```toml
[dependencies]
lightspeed_validator = { version = "0.66", features = ["credit_card"] }
```

### custom

Plugs a user-supplied function into the validator pipeline. Use it when
the rule doesn't fit any of the built-in validators — domain-specific
shape checks, lookups that read the validation context, anything you'd
otherwise hand-roll.

The function must match

```text
fn(value: &FieldTy, ctx: &CtxTy) -> Result<(), FieldErrTy>
```

where `FieldTy` is the field's type, `CtxTy` is the struct's
`#[validate(context = ...)]` type (or `()` if absent), and `FieldErrTy`
is the field's error type as picked by the struct-level `errors(...)`
strategy — `ValidationError` under `errors(shared)` (the default), your
`<Type>` under `errors(custom = <Type>)`, and the generated per-field
enum under `errors(tailored)`. The accepted field type is delegated to
the function's signature: a `&String` parameter on a field typed `i32`
produces a compile error pointing at the macro-generated wrapper, not at
a hand-written `ensure_*` check.

The function name is given as either a string literal — matching the
[`validator`](https://github.com/Keats/validator) crate's convention —
or a bare path:

```rust,no_run
use std::collections::HashMap;
use lightspeed_validator::{Validable, ValidationError};

fn not_reserved(value: &String, _ctx: &()) -> Result<(), ValidationError> {
    if value == "root" || value == "admin" {
        Err(ValidationError::Custom {
            code: "reserved".to_string(),
            message: "this name is reserved".to_string(),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

#[derive(Validable)]
struct Signup {
    // String literal form
    #[validate(custom(function = "not_reserved"))]
    username: String,

    // Bare path form
    #[validate(custom(function = not_reserved))]
    handle: String,
}
```

Reading the validation context works exactly as for the built-in
validators — the second parameter is `&CtxTy`:

```rust,no_run
use lightspeed_validator::{Validable, ValidationError, length::LengthError};

pub struct MinLenContext { pub min: usize }

fn at_least_min(value: &String, ctx: &MinLenContext) -> Result<(), ValidationError> {
    let actual = value.chars().count();
    if actual >= ctx.min {
        Ok(())
    } else {
        Err(ValidationError::Length(LengthError {
            min: Some(ctx.min),
            max: None,
            equal: None,
            actual,
        }))
    }
}

#[derive(Validable)]
#[validate(context = MinLenContext)]
struct PolicyUser {
    #[validate(custom(function = "at_least_min"))]
    name: String,
}

let v = PolicyUserValidable::new(PolicyUser { name: "ab".to_string() });
assert!(v.validate(&MinLenContext { min: 3 }).is_err());
```

Under `errors(tailored)` the macro-generated per-field enum gains a
`Custom(ValidationError)` variant plus the usual `From<ValidationError>`
impl — so a function that returns the wide `ValidationError` is lifted
into the narrow enum with `.into()`:

```rust,no_run
use std::collections::HashMap;
use lightspeed_validator::{Validable, ValidationError};

fn ban_root(value: &String, _ctx: &()) -> Result<(), TailoredUserNameFieldError> {
    if value == "root" {
        Err(ValidationError::Custom {
            code: "banned".to_string(),
            message: "name is reserved".to_string(),
            params: HashMap::new(),
        }
        .into())
    } else {
        Ok(())
    }
}

#[derive(Validable)]
#[validate(errors(tailored))]
struct TailoredUser {
    #[validate(custom(function = "ban_root"))]
    name: String,
}
```

Custom validators compose with the built-in keywords on the same field
and run in declaration order. The function pointer is boxed once inside
`<Name>Validable::new` and reused on every `validate` call — no
per-validation allocation.

For *stateful* validators — anything that needs to carry data alongside
the function — write a type implementing `FieldValidator<T, E, Ctx>` and
plug it in manually; see [Writing a custom field
validator](#writing-a-custom-field-validator).

### Multiple validators on the same field

Field attributes are additive — you can either repeat the attribute or
combine them in a single one:

```rust,no_run
use lightspeed_validator::Validable;

#[derive(Validable)]
struct Example {
    #[validate(contains(pattern = "@"))]
    #[validate(not_contains(pattern = " "))]
    email: String,

    #[validate(contains(pattern = "@"), not_contains(pattern = " "))]
    email_short_form: String,
}
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

```rust,no_run
use lightspeed_validator::Validable;

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

## Writing a custom field validator

For a plain function with no captured state, the
[`custom(function = ...)`](#custom) field attribute already plugs it into
the derive-generated pipeline. This section is for *stateful* validators
— types that carry data alongside the validation function.

Anything implementing `FieldValidator<T, ValidationError, Ctx>` can be used
manually, without the derive macro:

```rust,no_run
use lightspeed_validator::range::RangeError;
use lightspeed_validator::{FieldValidator, ValidableType, ValidationError};

struct MustBeGreaterValidator { min: usize }

impl FieldValidator<usize, ValidationError, ()> for MustBeGreaterValidator {
    fn validate(&self, value: &usize, _ctx: &()) -> Result<(), ValidationError> {
        if *value > self.min {
            Ok(())
        } else {
            // Re-use the shipped `Range` error variant — `exclusive_min`
            // expresses "value must be strictly greater than this bound".
            Err(ValidationError::Range(RangeError {
                min: None,
                max: None,
                exclusive_min: Some(self.min.to_string()),
                exclusive_max: None,
            }))
        }
    }
}

let mut field: ValidableType<usize> =
    ValidableType::new(3, vec![Box::new(MustBeGreaterValidator { min: 5 }).into()]);
field.validate(&());
assert!(!field.errors().is_empty());
```

The `StructValidator<T, ValidationError, Ctx>` trait plays the same role for
rules that need access to the whole validable struct after the field pass.

## Cargo features

- `credit_card` — pulls in the [`card-validate`](https://docs.rs/card-validate)
  crate and enables the `#[validate(credit_card)]` field validator together
  with `lightspeed_validator::credit_card` and
  `ValidationError::CreditCard(...)`. Off by default to keep the dependency
  footprint small for users that don't need card validation.
