use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, FieldsNamed, Ident, ItemStruct, parse_macro_input};

mod validators;

use validators::{FieldValidator, VALIDATE_ATTR};

/// Attribute macro applied to a named-field struct.
///
/// Emits the original struct unchanged (minus `#[validate(...)]` field
/// attributes, which are consumed here) plus a sibling `<Name>Validable`
/// whose fields are wrapped in [`lightspeed_validator::ValidableType`].
/// The sibling exposes `validate(self) -> Result<<Name>, Self>` which
/// runs each configured field validator and returns the original struct
/// when every field is valid, otherwise returns the validable struct
/// unchanged.
///
/// Field-level validators are declared via `#[validate(<keyword>)]`.
/// Supported keywords: `isTrue`, `isFalse` (both require `bool` fields).
#[proc_macro_attribute]
pub fn validable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let named_fields = match &input.fields {
        Fields::Named(named) => named,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[validable] only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_validators = match collect_field_validators(named_fields) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    // `#[validate(...)]` is consumed by this macro and is not a real
    // Rust attribute, so remove it before re-emitting the struct.
    strip_validate_attrs(&mut input);

    let named_fields = match &input.fields {
        Fields::Named(named) => named,
        _ => unreachable!("checked above"),
    };

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_struct = generate_validable_struct(vis, &validable_name, named_fields);
    let validate_fn = generate_validate_fn(name, &validable_name, named_fields, &field_validators);

    let expanded = quote! {
        #input

        #validable_struct

        #validate_fn
    };

    expanded.into()
}

/// Parses `#[validate(...)]` attributes from every field of the struct,
/// pairing each field's identifier with its (possibly empty) validator list.
fn collect_field_validators(fields: &FieldsNamed) -> syn::Result<Vec<(Ident, Vec<FieldValidator>)>> {
    fields
        .named
        .iter()
        .map(|f| {
            let ident = f.ident.clone().expect("named field has ident");
            let vs = validators::parse_field_validators(f)?;
            Ok((ident, vs))
        })
        .collect()
}

/// Removes `#[validate(...)]` from every field on the input struct so the
/// re-emitted struct compiles (rustc would otherwise reject the unknown attr).
fn strip_validate_attrs(input: &mut ItemStruct) {
    if let Fields::Named(named) = &mut input.fields {
        for field in named.named.iter_mut() {
            field.attrs.retain(|a| !a.path().is_ident(VALIDATE_ATTR));
        }
    }
}

/// Emits the `<Name>Validable` struct definition, mirroring the original
/// fields with each type `T` wrapped as `ValidableType<T>`.
fn generate_validable_struct(
    vis: &syn::Visibility,
    validable_name: &Ident,
    fields: &FieldsNamed,
) -> TokenStream2 {
    let validable_fields = fields.named.iter().map(|f| {
        let field_vis = &f.vis;
        let field_name = f.ident.as_ref().expect("named field has ident");
        let field_ty = &f.ty;
        quote! {
            #field_vis #field_name: ::lightspeed_validator::ValidableType<#field_ty>
        }
    });

    quote! {
        #vis struct #validable_name {
            #(#validable_fields,)*
        }
    }
}

/// Emits `impl <Name>Validable { pub fn validate(self) -> Result<<Name>, Self> }`.
/// Runs every configured field-level validator, then returns `Err(self)` if any
/// field reports invalid; otherwise moves each field's inner value into a fresh
/// instance of the original struct.
fn generate_validate_fn(
    name: &Ident,
    validable_name: &Ident,
    fields: &FieldsNamed,
    field_validators: &[(Ident, Vec<FieldValidator>)],
) -> TokenStream2 {
    let field_idents: Vec<&Ident> = fields
        .named
        .iter()
        .map(|f: &Field| f.ident.as_ref().expect("named field has ident"))
        .collect();

    let checks: Vec<TokenStream2> = field_validators
        .iter()
        .flat_map(|(field_ident, vs)| vs.iter().map(move |v| validators::generate_check(field_ident, v)))
        .collect();

    quote! {
        impl #validable_name {
            pub fn validate(mut self) -> ::core::result::Result<#name, Self> {
                #( #checks )*
                let all_valid = true #( && self.#field_idents.is_valid() )*;
                if !all_valid {
                    return ::core::result::Result::Err(self);
                }
                ::core::result::Result::Ok(#name {
                    #( #field_idents: self.#field_idents.into_value(), )*
                })
            }
        }
    }
}
