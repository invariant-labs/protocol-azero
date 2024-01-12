extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};
use wasm_bindgen::prelude::*;

#[proc_macro_attribute]
pub fn wasm_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as ItemFn);

    let original_function_block = &input.block;
    let original_function_name = &input.sig.ident;
    let visibility = &input.vis;
    let generated_function_name = format!("{}_generated", original_function_name);
    let generated_function_ident =
        syn::Ident::new(&generated_function_name, original_function_name.span());
    let original_return_type = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let params = input.sig.inputs.iter().filter_map(|param| {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = &ident.ident;
                let param_ty = quote! { wasm_bindgen::JsValue };
                Some(quote! { #param_name: #param_ty })
            } else {
                None
            }
        } else {
            None
        }
    });

    // Generate the new function with the specified name
    let new_function = quote! {
        #[wasm_bindgen]
        #visibility fn #generated_function_ident(#(#params),*) {
            // #original_function_block
        }
    };

    // Combine the original function with the generated function
    let result = quote! {
        #input
        #new_function
    };

    result.into()
}
