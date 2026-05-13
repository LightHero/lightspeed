use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, FieldsNamed, Ident, ItemStruct, parse_macro_input};

/// Attribute macro applied to a named-field struct.
///
/// Emits the original struct unchanged plus a sibling `<Name>Validable`
/// whose fields are wrapped in [`lightspeed_validator::ValidableType`].
/// The sibling exposes `validate(self) -> Result<<Name>, Self>` which
/// returns the original struct when every field is valid, otherwise
/// returns the validable struct unchanged.
#[proc_macro_attribute]
pub fn validable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let named_fields = match &input.fields {
        syn::Fields::Named(named) => named,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[validable] only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_struct = generate_validable_struct(vis, &validable_name, named_fields);
    let validate_fn = generate_validate_fn(name, &validable_name, named_fields);

    let expanded = quote! {
        #input

        #validable_struct

        #validate_fn
    };

    expanded.into()
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
/// Returns `Err(self)` if any field reports invalid, otherwise moves each
/// field's inner value into a fresh instance of the original struct.
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
