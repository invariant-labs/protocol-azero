extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Type;
use syn::{parse_macro_input, FnArg, ItemFn};

#[proc_macro_attribute]
pub fn wasm_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let original_function_name = &input.sig.ident;

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

                    let conversion_code = if is_i32_type(param_ty) {
                        quote! {
                            let #param_name: i64 = serde_wasm_bindgen::from_value(#js_param_name)?;
                            let #param_name: i32 = #param_name as i32;
                        }
                    } else if is_u16_type(param_ty) {
                        quote! {
                            let #param_name: u64 = serde_wasm_bindgen::from_value(#js_param_name)?;
                            let #param_name: u16 = #param_name as u16;
                        }
                    } else {
                        quote! {
                            let #param_name: #param_ty = serde_wasm_bindgen::from_value(#js_param_name)?;
                        }
                    };
    
                    (
                        conversion_code,
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

    let exported_function = quote! {
        #[wasm_bindgen(js_name = #camel_case_string)]
        pub fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
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
        #exported_function
    };

    result.into()
}

fn is_i32_type(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return segment.ident.to_string() == "i32";
        }
    }
    false
}

fn is_u16_type(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return segment.ident.to_string() == "u16";
        }
    }
    false
}
