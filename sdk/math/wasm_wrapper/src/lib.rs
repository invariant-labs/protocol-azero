extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro::Group;
use proc_macro2::TokenTree;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, ItemFn, ReturnType, Type, Ident};

#[proc_macro_attribute]
pub fn wasm_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let original_function_name = &input.sig.ident;

    let generated_function_name = format!("wrapped_{}", original_function_name);
    let generated_function_ident =
        syn::Ident::new(&generated_function_name, original_function_name.span());
        
    let return_ty = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let mut idents: Vec<String> = Vec::new();
    
    for token in return_ty.clone().into_iter() {
        match token {
            TokenTree::Group(group) => {
                for inner_token in group.stream() {
                    match inner_token {
                        TokenTree::Ident(ident) => {
                            println!("Ident inside Group: {:?}", ident);
                            println!("Ident type: {:?}", ident.to_string());
                            idents.push(ident.to_string());
                        }
                        _ => { }
                    }
                }
            }
            _ => { }
        }
    }

    println!("idents: {:?}", idents);

    // TODO: Change tuple name to more generic depending on function name
    let tuple_struct_name = Ident::new("MyTupleStruct", proc_macro2::Span::call_site());
    let tuple_struct_fields: Vec<proc_macro2::TokenStream> = idents
        .iter()
        .map(|ident| {
            let field_ident = Ident::new(ident, proc_macro2::Span::call_site());
            quote::quote! { #field_ident }
        })
        .collect();


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

                    let (require_cast, intermediate_cast) = requires_special_casting(param_ty);
                    let intermediate_cast = if require_cast {
                        Some(intermediate_cast)
                    } else {
                        None
                    };
    
                    let conversion_code = if let Some(intermediate_cast) = intermediate_cast {
                        quote! {
                            let #param_name: #intermediate_cast = serde_wasm_bindgen::from_value(#js_param_name)?;
                            let #param_name: #param_ty = #param_name as #param_ty;
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


    let exported_function = if idents.len() > 0 {
        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Tsify)]
            #[tsify(into_wasm_abi, from_wasm_abi)]
            pub struct #tuple_struct_name (
                #(#tuple_struct_fields),*
            );

            #[wasm_bindgen(js_name = #camel_case_string)]
            pub fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
                #(#conversion_code)*
    
                let result = #original_function_name(#(#converted_params),*);
    
                match result {
                    Ok(tuple) => {
                        let tuple_struct_instance = #tuple_struct_name(tuple.0, tuple.1);
    
                        Ok(serde_wasm_bindgen::to_value(&tuple_struct_instance)?)
                    }
                    Err(error) => Err(JsValue::from_str(&error.to_string())),
                }
            }
        }
    } else {
        quote! {
            #[wasm_bindgen(js_name = #camel_case_string)]
            pub fn #generated_function_ident(#(#params),*) -> Result<JsValue, JsValue> {
                #(#conversion_code)*

                let result = #original_function_name(#(#converted_params),*);

                match result {
                    Ok(v) => Ok(serde_wasm_bindgen::to_value(&v)?),
                    Err(error) => Err(JsValue::from_str(&error.to_string())),
                }
            }
        }
    };

    let result = quote! {
        #input
        // #exported_struct
        #exported_function
    };

    result.into()
}

fn requires_special_casting(ty: &Type) -> (bool, syn::Ident) {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "i32" | "i16" | "i8" => return (true, syn::Ident::new("i64", segment.ident.span())),
                "u32" | "u16" | "u8" => return (true, syn::Ident::new("i64", segment.ident.span())),
                _ => return (false, segment.ident.clone()),
            };
        }
    }
    (false, syn::Ident::new("unknown", ty.span()))
}