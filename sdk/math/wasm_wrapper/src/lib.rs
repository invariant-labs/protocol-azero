extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, ItemFn};

#[proc_macro_attribute]
pub fn wasm_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let original_function_name = &input.sig.ident;
    let visibility = &input.vis;

    let generated_function_name = format!("wrapped_{}", original_function_name);
    let generated_function_ident =
        syn::Ident::new(&generated_function_name, original_function_name.span());

    let camel_case_string = {
        let mut camel_case = String::new();
        let mut capitalize_next = false;
        for c in original_function_name.to_string().chars() {
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

    let params = input.sig.inputs.iter().filter_map(|param| {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = &ident.ident;
                let js_param_name =
                    syn::Ident::new(&format!("js_{}", param_name), ident.ident.span());
                let param_ty = quote! { wasm_bindgen::JsValue };
                Some(quote! { #js_param_name: #param_ty })
            } else {
                None
            }
        } else {
            None
        }
    });

    let (conversion_code, converted_params): (Vec<_>, Vec<_>) = input
        .sig
        .inputs
        .iter()
        .map(|param| {
            if let FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let param_name = &ident.ident;
                    let js_param_name =
                        syn::Ident::new(&format!("js_{}", param_name), ident.span());
                    let param_ty = &pat_type.ty;
                    (
                        quote! { let #param_name: #param_ty = serde_wasm_bindgen::from_value(#js_param_name).unwrap(); },
                        quote! { #param_name },
                    )
                } else {
                    (quote! {}, quote! {})
                }
            } else {
                (quote! {}, quote! {})
            }
        })
        .unzip();

    let new_function = quote! {
        #[wasm_bindgen(js_name = #camel_case_string)]
        #visibility fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
            #(#conversion_code)*

            let result = #original_function_name(#(#converted_params),*);

            match result {
                Ok(v) => Ok(serde_wasm_bindgen::to_value(&v)?),
                Err(error) => Err(JsValue::from_str(&error.to_string())),
            }
        }
    };

    let result = quote! {
        #input
        #new_function
    };

    result.into()
}
