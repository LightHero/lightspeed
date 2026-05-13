//! Field-level validators parsed from `#[validate(...)]` attributes.
//!
//! To add a new variant:
//!  1. add it to [`FieldValidator`];
//!  2. recognize its keyword in [`parse_keyword`];
//!  3. declare its accepted field types in [`check_field_type`];
//!  4. emit its runtime check in [`generate_check`].

pub mod boolean;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Ident, Type};

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

/// Emits the runtime check for `validator` against `self.<field_ident>`,
/// pushing the appropriate `ValidationError` on failure.
pub fn generate_check(field_ident: &Ident, validator: &FieldValidator) -> TokenStream2 {
    match validator {
        FieldValidator::IsTrue => quote! {
            if !*self.#field_ident.get() {
                self.#field_ident.push_error(
                    ::lightspeed_validator::ValidationError::MustBeTrue,
                );
            }
        },
        FieldValidator::IsFalse => quote! {
            if *self.#field_ident.get() {
                self.#field_ident.push_error(
                    ::lightspeed_validator::ValidationError::MustBeFalse,
                );
            }
        },
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
        FieldValidator::IsTrue | FieldValidator::IsFalse => {
            if !type_is_bool(&field.ty) {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`isTrue` / `isFalse` validators require a `bool` field",
                ));
            }
        }
    }
    Ok(())
}

fn type_is_bool(ty: &Type) -> bool {
    let Type::Path(p) = ty else { return false };
    p.qself.is_none() && p.path.is_ident("bool")
}
