use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, FieldsNamed, Ident, ItemStruct, Type, parse_macro_input, parse_quote};

mod validation;

use validation::FieldValidator;
use validation::struct_fields_match::{self, FieldsMatchArgs};

const VALIDATE_ATTR: &str = "validate";
const CONTEXT_KEYWORD: &str = "context";
const FIELDS_MATCH_KEYWORD: &str = "fields_match";

/// Derive macro that generates a `<Name>Validable` companion type.
///
/// Applied to a named-field struct via `#[derive(Validable)]`, this emits a
/// sibling `<Name>Validable` whose fields are wrapped in
/// [`lightspeed_validator::ValidableType`]. The sibling exposes:
/// - `new(value: <Name>) -> Self` to wrap an instance with the validator list
///   declared by the field-level `#[validate(...)]` attributes;
/// - `validate(self) -> Result<<Name>, Self>` (or `validate(self, ctx: &Ctx)`
///   when a custom context is configured) which runs each field's validators
///   and any struct-level rules. Returns the original struct when no
///   field collected any errors and no struct-level rule produced any
///   top-level errors. Otherwise returns the validable (with errors populated
///   on each field's `ValidableType` and/or on the top-level `errors` vec);
/// - `top_level_errors(&self) -> &[ValidationError]` to read errors produced
///   by struct-level validators that were not attached to specific fields.
///
/// ## Struct-level attributes
/// - `#[validate(context = <Type>)]` — selects the validation context type
///   threaded to every validator's `validate(value, ctx)` call. Defaults to
///   `()` when absent.
/// - `#[validate(fields_match(<field_a>, <field_b>))]` — requires the two
///   named fields to compare equal (via `PartialEq`). Emits
///   `ValidationError::FieldsMustMatch { a, b }` on failure. By default the
///   error goes onto the top-level `errors` vec; pass `attach_to_fields = true`
///   to push a copy onto each named field's `errors` instead.
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
///   `case_sensitive` defaults to `true`;
/// - `ip` / `ipv4` / `ipv6` — requires a string-compatible field; the value
///   must parse as an IP address of the corresponding kind (any / v4 / v6).
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

    let struct_attrs = match parse_struct_attrs(&input) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    if let Err(e) = validate_struct_field_refs(named_fields, &struct_attrs.validators) {
        return e.to_compile_error().into();
    }

    let field_validators = match collect_field_validators(named_fields) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let vis = &input.vis;
    let name = &input.ident;
    let validable_name = format_ident!("{}Validable", name);

    let validable_struct = generate_validable_struct(vis, &validable_name, named_fields, &struct_attrs.context);
    let new_fn = generate_new_fn(name, &validable_name, named_fields, &field_validators);
    let validate_fn = generate_validate_fn(
        name,
        &validable_name,
        named_fields,
        &struct_attrs.context,
        &struct_attrs.validators,
    );
    let struct_validator_impls =
        generate_struct_validator_impls(&validable_name, &struct_attrs.context, &struct_attrs.validators);

    let expanded = quote! {
        #validable_struct

        #new_fn

        #validate_fn

        #struct_validator_impls
    };

    expanded.into()
}

/// Struct-level configuration parsed from `#[validate(...)]` attributes on the
/// item itself: the optional `context = <Type>` and the list of struct-level
/// rules (e.g. `fields_match(...)`).
struct StructAttrs {
    context: StructContext,
    validators: Vec<StructLevelValidator>,
}

struct StructContext {
    ty: Type,
    is_explicit: bool,
}

/// A single struct-level rule. Parsing, validation and code generation for
/// each variant live in `validation::<variant>` (e.g.
/// [`validation::struct_fields_match`]); this enum only routes between them.
enum StructLevelValidator {
    FieldsMatch(FieldsMatchArgs),
}

fn parse_struct_attrs(input: &ItemStruct) -> syn::Result<StructAttrs> {
    let mut ty: Option<Type> = None;
    let mut validators: Vec<StructLevelValidator> = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident(VALIDATE_ATTR) {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(CONTEXT_KEYWORD) {
                if ty.is_some() {
                    return Err(meta.error("duplicate `context = ...`"));
                }
                ty = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident(FIELDS_MATCH_KEYWORD) {
                validators.push(StructLevelValidator::FieldsMatch(
                    struct_fields_match::parse_fields_match(&meta)?,
                ));
                Ok(())
            } else {
                Err(meta.error("unknown struct-level `#[validate(...)]` option"))
            }
        })?;
    }

    let context = match ty {
        Some(ty) => StructContext { ty, is_explicit: true },
        None => StructContext { ty: parse_quote!(()), is_explicit: false },
    };
    Ok(StructAttrs { context, validators })
}

/// Ensures every field referenced by a struct-level rule exists on the struct.
/// Dispatches to the per-variant `ensure_*` helper.
fn validate_struct_field_refs(
    fields: &FieldsNamed,
    validators: &[StructLevelValidator],
) -> syn::Result<()> {
    for v in validators {
        match v {
            StructLevelValidator::FieldsMatch(args) => {
                struct_fields_match::ensure_fields_exist(fields, args)?;
            }
        }
    }
    Ok(())
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
/// fields with each type `T` wrapped as `ValidableType<T, Ctx>` and adding a
/// private `top_level_errors` vec used by struct-level validators.
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
            top_level_errors: ::std::vec::Vec<::lightspeed_validator::ValidationError>,
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
                    top_level_errors: ::std::vec::Vec::new(),
                }
            }
        }
    }
}

/// Emits the `validate` method on `<Name>Validable` plus the
/// `top_level_errors()` accessor.
///
/// `validate` first runs each field's `ValidableType::validate(ctx)`, then
/// invokes each struct-level validator (e.g. `fields_match`) and routes its
/// errors either to the field-level `errors` vecs or to the `top_level_errors`
/// vec depending on the rule's `attach_to_fields` flag. Returns `Err(self)` if
/// any field or the top-level vec collected at least one error; otherwise
/// moves each field's inner value into a fresh instance of the original struct.
fn generate_validate_fn(
    name: &Ident,
    validable_name: &Ident,
    fields: &FieldsNamed,
    context: &StructContext,
    struct_validators: &[StructLevelValidator],
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

    let struct_validator_calls = struct_validators.iter().enumerate().map(|(idx, v)| {
        let validator_ident = struct_validator_unit_ident(validable_name, idx);
        match v {
            StructLevelValidator::FieldsMatch(args) => struct_fields_match::generate_dispatch(
                &validator_ident,
                validable_name,
                ctx_ty,
                &ctx_expr,
                args,
            ),
        }
    });

    quote! {
        impl #validable_name {
            pub fn validate(mut self #extra_param) -> ::core::result::Result<#name, Self> {
                #( self.#field_idents.validate(#ctx_expr); )*
                #( #struct_validator_calls )*
                let has_errors = !self.top_level_errors.is_empty()
                    #( || !self.#field_idents.errors().is_empty() )*;
                if has_errors {
                    return ::core::result::Result::Err(self);
                }
                ::core::result::Result::Ok(#name {
                    #( #field_idents: self.#field_idents.into_value(), )*
                })
            }

            pub fn top_level_errors(&self) -> &[::lightspeed_validator::ValidationError] {
                &self.top_level_errors
            }
        }
    }
}

/// Emits one unit struct + `StructValidator` impl per struct-level rule.
/// Each unit struct is named `__<ValidableName>StructValidator<idx>` and lives
/// at the same module scope as the original struct.
fn generate_struct_validator_impls(
    validable_name: &Ident,
    context: &StructContext,
    struct_validators: &[StructLevelValidator],
) -> TokenStream2 {
    let ctx_ty = &context.ty;
    let items = struct_validators.iter().enumerate().map(|(idx, v)| {
        let validator_ident = struct_validator_unit_ident(validable_name, idx);
        match v {
            StructLevelValidator::FieldsMatch(args) => struct_fields_match::generate_validator_impl(
                validable_name,
                ctx_ty,
                &validator_ident,
                args,
            ),
        }
    });

    quote! { #( #items )* }
}

fn struct_validator_unit_ident(validable_name: &Ident, idx: usize) -> Ident {
    format_ident!("__{}StructValidator{}", validable_name, idx)
}
