use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Ident, Type};

pub fn process_params(input: &syn::ItemFn) -> Vec<TokenStream> {
    input
        .sig
        .inputs
        .iter()
        .filter_map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let param_name = &ident.ident;
                    let js_param_name =
                        syn::Ident::new(&format!("js_{}", param_name), ident.span());
                    let param_ty = quote! { wasm_bindgen::JsValue };
                    Some(quote! { #js_param_name: #param_ty })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn process_return_type(
    return_ty: proc_macro2::TokenStream,
    camel_case_string: String,
) -> (Ident, Vec<proc_macro2::TokenStream>) {
    let mut idents: Vec<String> = Vec::new();

    for token in return_ty.clone().into_iter() {
        match token {
            TokenTree::Group(group) => {
                for inner_token in group.stream() {
                    match inner_token {
                        TokenTree::Ident(ident) => {
                            idents.push(ident.to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let tuple_struct_name = Ident::new(
        &format!("{}{}", camel_case_string, "Result"),
        proc_macro2::Span::call_site(),
    );
    let tuple_struct_fields: Vec<proc_macro2::TokenStream> = idents
        .iter()
        .map(|ident| {
            let field_ident = Ident::new(ident, proc_macro2::Span::call_site());
            quote::quote! { #field_ident }
        })
        .collect();

    (tuple_struct_name, tuple_struct_fields)
}

pub fn requires_special_casting(ty: &Type) -> (bool, syn::Ident) {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "i32" | "i16" | "i8" => {
                    return (true, syn::Ident::new("i64", segment.ident.span()))
                }
                "u32" | "u16" | "u8" => {
                    return (true, syn::Ident::new("u64", segment.ident.span()))
                }
                _ => return (false, segment.ident.clone()),
            };
        }
    }
    (false, syn::Ident::new("unknown", ty.span()))
}

pub fn construct_camel_case(args: Vec<&str>, original_function_name: String) -> String {
    let camel_case_string = if args.len() == 1 && args[0] != "" {
        let trimmed_string = args[0].trim_matches(|c| c == '"' || c == '\\');
        trimmed_string.to_string()
    } else {
        let mut camel_case = String::new();
        let mut capitalize_next = false;
        for c in original_function_name.chars() {
            if c == '_' {
                capitalize_next = true;
            } else {
                if capitalize_next {
                    camel_case.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    camel_case.push(c);
                }
            }
        }
        camel_case
    };
    camel_case_string
}
