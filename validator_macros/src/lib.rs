use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn validable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

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

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_fields = named_fields.named.iter().map(|f| {
        let field_vis = &f.vis;
        let field_name = f.ident.as_ref().expect("named field has ident");
        let field_ty = &f.ty;
        quote! {
            #field_vis #field_name: ::lightspeed_validator::ValidableType<#field_ty>
        }
    });

    let expanded = quote! {
        #input

        #vis struct #validable_name {
            #(#validable_fields,)*
        }
    };

    expanded.into()
}
