use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// `#[requires_role("admin")]` — injects an RBAC guard at the top of an async axum handler.
///
/// The handler must have a parameter named `__auth_user: AuthUser` (from rbac extractor).
/// If the caller's role is insufficient, returns 403 immediately.
///
/// Role hierarchy: viewer < operator < admin
#[proc_macro_attribute]
pub fn requires_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let required = parse_macro_input!(attr as LitStr).value();
    let mut func = parse_macro_input!(item as ItemFn);

    let original_stmts = &func.block.stmts;

    let guarded = quote! {
        {
            use axum::http::StatusCode;
            use axum::response::IntoResponse;
            if !crate::server::rbac::role_allows(&__auth_user.role, #required) {
                tracing::warn!(
                    user = %__auth_user.name,
                    required = #required,
                    actual = %__auth_user.role,
                    "RBAC denied"
                );
                return (
                    StatusCode::FORBIDDEN,
                    axum::Json(serde_json::json!({
                        "error": "forbidden",
                        "required_role": #required
                    }))
                ).into_response();
            }
            tracing::info!(user = %__auth_user.name, role = %__auth_user.role, "RBAC allowed");
            #(#original_stmts)*
        }
    };

    func.block = syn::parse2(guarded).unwrap();
    quote!(#func).into()
}
