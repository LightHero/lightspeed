use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, FieldsNamed, Ident, ItemStruct, parse_macro_input};

mod validation;

use validation::FieldValidator;

/// Derive macro that generates a `<Name>Validable` companion type.
///
/// Applied to a named-field struct via `#[derive(Validable)]`, this emits a
/// sibling `<Name>Validable` whose fields are wrapped in
/// [`lightspeed_validator::ValidableType`]. The sibling exposes
/// `validate(self) -> Result<<Name>, Self>` which checks each field's
/// `is_valid()` flag and returns the original struct when every field is
/// valid, otherwise returns the validable struct unchanged.
///
/// Field-level validators are declared via the helper attribute
/// `#[validate(<keyword>)]`. Supported keywords:
/// - `isTrue` — requires a `bool` field; the value must be `true`;
/// - `isFalse` — requires a `bool` field; the value must be `false`.
#[proc_macro_derive(Validable, attributes(validate))]
pub fn derive_validable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let named_fields = match &input.fields {
        Fields::Named(named) => named,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[derive(Validable)] only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_validators = match collect_field_validators(named_fields) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_struct = generate_validable_struct(vis, &validable_name, named_fields);
    let new_fn = generate_new_fn(name, &validable_name, named_fields, &field_validators);
    let validate_fn = generate_validate_fn(name, &validable_name, named_fields);

    let expanded = quote! {
        #validable_struct

        #new_fn

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
            let vs = validation::parse_field_validators(f)?;
            Ok((ident, vs))
        })
        .collect()
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

/// Emits `impl <Name>Validable { pub fn new(value: <Name>) -> Self }`.
/// Each field is wrapped in a `ValidableType` whose validator list is built
/// from the `#[validate(...)]` attributes declared on that field.
fn generate_new_fn(
    name: &Ident,
    validable_name: &Ident,
    fields: &FieldsNamed,
    field_validators: &[(Ident, Vec<FieldValidator>)],
) -> TokenStream2 {
    let field_inits = fields.named.iter().zip(field_validators.iter()).map(|(f, (_, vs))| {
        let field_ident = f.ident.as_ref().expect("named field has ident");
        let validators_vec = validation::generate_validators_vec(field_ident, vs);
        quote! {
            #field_ident: ::lightspeed_validator::ValidableType::new(value.#field_ident, #validators_vec)
        }
    });

    quote! {
        impl #validable_name {
            pub fn new(value: #name) -> Self {
                Self {
                    #( #field_inits, )*
                }
            }
        }
    }
}

/// Emits `impl <Name>Validable { pub fn validate(self) -> Result<<Name>, Self> }`.
/// Iterates every field, returns `Err(self)` if any field reports invalid;
/// otherwise moves each field's inner value into a fresh instance of the
/// original struct. Field-level validation logic lives on each field's
/// `ValidableType` validator list (wired up by `new`), so this function does
/// not inspect field values directly.
fn generate_validate_fn(
    name: &Ident,
    validable_name: &Ident,
    fields: &FieldsNamed,
) -> TokenStream2 {
    let field_idents: Vec<&Ident> = fields
        .named
        .iter()
        .map(|f: &Field| f.ident.as_ref().expect("named field has ident"))
        .collect();

    quote! {
        impl #validable_name {
            pub fn validate(self) -> ::core::result::Result<#name, Self> {
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
