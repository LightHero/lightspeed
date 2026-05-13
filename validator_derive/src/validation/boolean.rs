//! Bool-specific code generation for `#[validate(isTrue)]` / `#[validate(isFalse)]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Type};

/// Returns true when `ty` is the bare `bool` type.
pub fn type_is_bool(ty: &Type) -> bool {
    let Type::Path(p) = ty else { return false };
    p.qself.is_none() && p.path.is_ident("bool")
}

/// Ensures a field annotated with an `isTrue` / `isFalse` validator is `bool`.
pub fn ensure_bool_field(field: &Field) -> syn::Result<()> {
    if !type_is_bool(&field.ty) {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "`isTrue` / `isFalse` validators require a `bool` field",
        ));
    }
    Ok(())
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustBeTrueValidator`.
pub fn must_be_true_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::boolean::MustBeTrueValidator)
    }
}

/// Tokens that construct a `Box<dyn FieldValidator<...>>` for `MustBeFalseValidator`.
pub fn must_be_false_validator_instance() -> TokenStream2 {
    quote! {
        ::std::boxed::Box::new(::lightspeed_validator::boolean::MustBeFalseValidator)
    }
}
