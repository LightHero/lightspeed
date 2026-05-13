//! Field-level validators parsed from `#[validate(...)]` attributes.
//!
//! To add a new variant:
//!  1. add it to [`FieldValidator`];
//!  2. recognize its keyword in [`parse_keyword`];
//!  3. declare its accepted field types in [`check_field_type`];
//!  4. emit its validator-instance tokens in [`generate_validator_instance`].

pub mod boolean;

use proc_macro2::TokenStream as TokenStream2;
use syn::{Field, Ident};

const VALIDATE_ATTR: &str = "validate";

/// A single validator configured on a field via `#[validate(<keyword>)]`.
pub enum FieldValidator {
    IsTrue,
    IsFalse,
}

/// Parses every `#[validate(...)]` attribute on `field`, returning all
/// validators in source order. Also enforces that each validator is
/// compatible with the field's declared type.
pub fn parse_field_validators(field: &Field) -> syn::Result<Vec<FieldValidator>> {
    let mut out = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident(VALIDATE_ATTR) {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let keyword = meta.path.require_ident()?;
            let validator = parse_keyword(keyword)?;
            check_field_type(field, &validator)?;
            out.push(validator);
            Ok(())
        })?;
    }
    Ok(out)
}

/// Emits the tokens that build a `Box<dyn FieldValidator<...>>` instance for
/// `validator`, suitable for inclusion in a `vec![...]` passed to
/// `ValidableType::new`.
pub fn generate_validator_instance(validator: &FieldValidator) -> TokenStream2 {
    match validator {
        FieldValidator::IsTrue => boolean::must_be_true_validator_instance(),
        FieldValidator::IsFalse => boolean::must_be_false_validator_instance(),
    }
}

/// Emits a `vec![...]` of validator instances for `validators`. An empty input
/// produces an empty vec literal.
pub fn generate_validators_vec(_field_ident: &Ident, validators: &[FieldValidator]) -> TokenStream2 {
    let items = validators.iter().map(generate_validator_instance);
    quote::quote! {
        ::std::vec![ #( #items ),* ]
    }
}

fn parse_keyword(keyword: &Ident) -> syn::Result<FieldValidator> {
    match keyword.to_string().as_str() {
        "isTrue" => Ok(FieldValidator::IsTrue),
        "isFalse" => Ok(FieldValidator::IsFalse),
        other => Err(syn::Error::new(keyword.span(), format!("unknown validator `{other}`"))),
    }
}

fn check_field_type(field: &Field, validator: &FieldValidator) -> syn::Result<()> {
    match validator {
        FieldValidator::IsTrue | FieldValidator::IsFalse => boolean::ensure_bool_field(field),
    }
}
