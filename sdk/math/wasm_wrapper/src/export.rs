use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate_exported_function(
    tuple_struct_name: &Ident,
    tuple_struct_fields: Vec<TokenStream>,
    camel_case_string: &str,
    generated_function_ident: &Ident,
    params: Vec<TokenStream>,
    conversion_code: Vec<TokenStream>,
    converted_params: Vec<TokenStream>,
    original_function_name: &Ident,
    result_not_wrapped: bool,
) -> TokenStream {
    if tuple_struct_fields.len() > 0 {
        tuple_exported_function(
            &tuple_struct_name,
            tuple_struct_fields,
            &camel_case_string,
            &generated_function_ident,
            params.clone(),
            conversion_code.clone(),
            converted_params.clone(),
            &original_function_name,
        )
    } else if result_not_wrapped {
        value_exproted_function(
            &camel_case_string,
            &generated_function_ident,
            params.clone(),
            conversion_code.clone(),
            converted_params.clone(),
            &original_function_name,
        )
    } else {
        struct_exported_function(
            &camel_case_string,
            &generated_function_ident,
            params.clone(),
            conversion_code.clone(),
            converted_params.clone(),
            &original_function_name,
        )
    }
}

pub fn value_exproted_function(
    camel_case_string: &str,
    generated_function_ident: &Ident,
    params: Vec<TokenStream>,
    conversion_code: Vec<TokenStream>,
    converted_params: Vec<TokenStream>,
    original_function_name: &Ident,
) -> TokenStream {
    quote! {
        #[wasm_bindgen(js_name = #camel_case_string)]
        pub fn #generated_function_ident(#(#params),*) -> Result<JsValue,JsValue> {
            #(#conversion_code)*

            let result = #original_function_name(#(#converted_params),*);
            // TODO - add parsing to BigInt when the value is < 2^53 - 1
            Ok(serde_wasm_bindgen::to_value(&result)?)
        }
    }
}

pub fn tuple_exported_function(
    tuple_struct_name: &Ident,
    tuple_struct_fields: Vec<TokenStream>,
    camel_case_string: &str,
    generated_function_ident: &Ident,
    params: Vec<TokenStream>,
    conversion_code: Vec<TokenStream>,
    converted_params: Vec<TokenStream>,
    original_function_name: &Ident,
) -> TokenStream {
    let tuple_struct_instance = match tuple_struct_fields.len() {
        1 => {
            quote! { #tuple_struct_name(tuple.0) }
        }
        2 => {
            quote! { #tuple_struct_name(tuple.0, tuple.1) }
        }
        3 => {
            quote! { #tuple_struct_name(tuple.0, tuple.1, tuple.2) }
        }
        4 => {
            quote! { #tuple_struct_name(tuple.0, tuple.1, tuple.2, tuple.3) }
        }
        5 => {
            quote! { #tuple_struct_name(tuple.0, tuple.1, tuple.2, tuple.3, tuple.4) }
        }
        _ => {
            panic!(
                "Unsupported number of tuple fields: {}",
                tuple_struct_fields.len()
            );
        }
    };

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
                    let mut tuple_struct_instance = #tuple_struct_instance;

                    Ok(serde_wasm_bindgen::to_value(&tuple_struct_instance)?)
                }
                Err(error) => Err(JsValue::from_str(&error.to_string())),
            }
        }
    }
}

pub fn struct_exported_function(
    camel_case_string: &str,
    generated_function_ident: &Ident,
    params: Vec<TokenStream>,
    conversion_code: Vec<TokenStream>,
    converted_params: Vec<TokenStream>,
    original_function_name: &Ident,
) -> TokenStream {
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
}
