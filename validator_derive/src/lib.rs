use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, FieldsNamed, Ident, ItemStruct, Type, parse_macro_input, parse_quote};

mod validation;

use validation::FieldValidator;

const VALIDATE_ATTR: &str = "validate";
const CONTEXT_KEYWORD: &str = "context";

/// Derive macro that generates a `<Name>Validable` companion type.
///
/// Applied to a named-field struct via `#[derive(Validable)]`, this emits a
/// sibling `<Name>Validable` whose fields are wrapped in
/// [`lightspeed_validator::ValidableType`]. The sibling exposes:
/// - `new(value: <Name>) -> Self` to wrap an instance with the validator list
///   declared by the field-level `#[validate(...)]` attributes;
/// - `validate(self) -> Result<<Name>, Self>` (or `validate(self, ctx: &Ctx)`
///   when a custom context is configured) which runs each field's validators
///   and returns the original struct when no field collected any errors,
///   otherwise returns the validable struct (with errors populated on each
///   field's `ValidableType`).
///
/// ## Struct-level attribute
/// - `#[validate(context = <Type>)]` — selects the validation context type
///   threaded to every validator's `validate(value, ctx)` call. Defaults to
///   `()` when absent. The generated `validate` method's signature mirrors
///   this: it takes no extra argument by default, and takes `ctx: &<Type>`
///   when set.
///
/// ## Field-level attribute
/// Field-level validators are declared via the helper attribute
/// `#[validate(<keyword>)]`. Supported keywords:
/// - `isTrue` — requires a `bool` field; the value must be `true`;
/// - `isFalse` — requires a `bool` field; the value must be `false`;
/// - `contains(pattern = "...", case_sensitive = <bool>)` — requires a
///   string-compatible field (`String`, `&str`, `Cow<'_, str>`, …); the
///   value must contain `pattern`. `case_sensitive` defaults to `true`;
/// - `not_contains(pattern = "...", case_sensitive = <bool>)` — same field
///   types as `contains`; the value must NOT contain `pattern`.
///   `case_sensitive` defaults to `true`.
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

    let context = match parse_struct_context(&input) {
        Ok(c) => c,
        Err(e) => return e.to_compile_error().into(),
    };

    let field_validators = match collect_field_validators(named_fields) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_struct = generate_validable_struct(vis, &validable_name, named_fields, &context);
    let new_fn = generate_new_fn(name, &validable_name, named_fields, &field_validators);
    let validate_fn = generate_validate_fn(name, &validable_name, named_fields, &context);

    let expanded = quote! {
        #validable_struct

        #new_fn

        #validate_fn
    };

    expanded.into()
}

/// Resolves the struct-level validation context.
///
/// Looks for `#[validate(context = <Type>)]` on the struct. When absent,
/// defaults to `()`. Errors on unknown keys or duplicate `context = ...`.
struct StructContext {
    ty: Type,
    is_explicit: bool,
}

fn parse_struct_context(input: &ItemStruct) -> syn::Result<StructContext> {
    let mut ty: Option<Type> = None;
    for attr in &input.attrs {
        if !attr.path().is_ident(VALIDATE_ATTR) {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(CONTEXT_KEYWORD) {
                if ty.is_some() {
                    return Err(meta.error("duplicate `context = ...`"));
                }
                let value = meta.value()?;
                ty = Some(value.parse()?);
                Ok(())
            } else {
                Err(meta.error("unknown struct-level `#[validate(...)]` option"))
            }
        })?;
    }
    Ok(match ty {
        Some(ty) => StructContext { ty, is_explicit: true },
        None => StructContext { ty: parse_quote!(()), is_explicit: false },
    })
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
/// fields with each type `T` wrapped as `ValidableType<T, Ctx>`.
fn generate_validable_struct(
    vis: &syn::Visibility,
    validable_name: &Ident,
    fields: &FieldsNamed,
    context: &StructContext,
) -> TokenStream2 {
    let ctx_ty = &context.ty;
    let validable_fields = fields.named.iter().map(|f| {
        let field_vis = &f.vis;
        let field_name = f.ident.as_ref().expect("named field has ident");
        let field_ty = &f.ty;
        quote! {
            #field_vis #field_name: ::lightspeed_validator::ValidableType<#field_ty, #ctx_ty>
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

/// Emits the `validate` method on `<Name>Validable`.
///
/// Runs `validate(ctx)` on each field's `ValidableType` (populating its
/// internal `errors`), then returns `Err(self)` if any field collected at
/// least one error. Otherwise moves each field's inner value into a fresh
/// instance of the original struct.
///
/// The method's signature is:
/// - `pub fn validate(self) -> Result<<Name>, Self>` when no context is set
///   (uses `&()` internally);
/// - `pub fn validate(self, ctx: &<Ctx>) -> Result<<Name>, Self>` when the
///   struct opted in to a custom context via `#[validate(context = <Ctx>)]`.
fn generate_validate_fn(
    name: &Ident,
    validable_name: &Ident,
    fields: &FieldsNamed,
    context: &StructContext,
) -> TokenStream2 {
    let field_idents: Vec<&Ident> = fields
        .named
        .iter()
        .map(|f: &Field| f.ident.as_ref().expect("named field has ident"))
        .collect();

    let ctx_ty = &context.ty;
    let (extra_param, ctx_expr) = if context.is_explicit {
        (quote! { , ctx: &#ctx_ty }, quote! { ctx })
    } else {
        (quote! {}, quote! { &() })
    };

    quote! {
        impl #validable_name {
            pub fn validate(mut self #extra_param) -> ::core::result::Result<#name, Self> {
                #( self.#field_idents.validate(#ctx_expr); )*
                let has_errors = false #( || !self.#field_idents.errors().is_empty() )*;
                if has_errors {
                    return ::core::result::Result::Err(self);
                }
                ::core::result::Result::Ok(#name {
                    #( #field_idents: self.#field_idents.into_value(), )*
                })
            }
        }
    }
}
