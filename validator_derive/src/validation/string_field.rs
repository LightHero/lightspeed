//! Shared helpers for "this validator only accepts string-compatible field
//! types" checks. Before this module each string-validator module
//! (`contains`, `ip`, `url`, `password`, `regex`, `email`, `credit_card`)
//! carried an identical copy of `is_string_like_type` plus a near-identical
//! `ensure_string_field` differing only in the error label — consolidated
//! here.

use syn::{Field, Type};

/// Returns true when `ty` looks like a string-compatible type (`String`,
/// `&str`, `Cow<_, str>`, `Box<str>`, `Rc<str>`, `Arc<str>`, etc.). The check
/// is intentionally syntactic and permissive: it accepts the shape of any
/// last-segment we recognise, leaving the actual `AsRef<str>` bound to be
/// enforced by rustc at the macro-emitted construction site.
pub fn is_string_like_type(ty: &Type) -> bool {
    match ty {
        Type::Path(p) => {
            if p.qself.is_some() {
                return false;
            }
            let Some(last) = p.path.segments.last() else { return false };
            matches!(last.ident.to_string().as_str(), "String" | "Cow" | "Box" | "Rc" | "Arc" | "str")
        }
        Type::Reference(r) => is_string_like_type(&r.elem),
        _ => false,
    }
}

/// Ensures `field`'s declared type looks string-compatible. `label` is the
/// validator name (e.g. `"url"`, or `"contains` / `not_contains"`) inserted
/// into the rejection message — pick whatever reads best when interpolated
/// into "`<label>` validator(s) require a string-compatible field".
pub fn ensure_string_field(field: &Field, label: &str) -> syn::Result<()> {
    if !is_string_like_type(&field.ty) {
        return Err(syn::Error::new_spanned(
            &field.ty,
            format!(
                "`{label}` validator requires a string-compatible field \
                 (e.g. `String`, `&str`, `Cow<'_, str>`)"
            ),
        ));
    }
    Ok(())
}
