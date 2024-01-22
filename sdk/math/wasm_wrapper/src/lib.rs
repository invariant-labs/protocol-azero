extern crate proc_macro;

mod conversion;
mod export;
mod helpers;
use crate::conversion::convert_params;
use crate::export::generate_exported_function;
use crate::helpers::{construct_camel_case, process_params, process_return_type};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn wasm_wrapper(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let original_function_name = &input.sig.ident;

    let generated_function_name = format!("wrapped_{}", original_function_name);
    let generated_function_ident =
        syn::Ident::new(&generated_function_name, original_function_name.span());

    let return_ty = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    println!(
        "return_ty: {:?}",
        return_ty.clone().into_iter().next().unwrap().to_string()
    );

    let args_str = attr.to_string();
    let args: Vec<&str> = args_str.split(',').collect();

    let camel_case_string = construct_camel_case(args.clone(), original_function_name.to_string());

    let (tuple_struct_name, tuple_struct_fields, result_not_wrapped) =
        process_return_type(return_ty.clone(), camel_case_string.clone());

    let params: Vec<_> = process_params(&input);

    let (conversion_code, converted_params): (Vec<_>, Vec<_>) = convert_params(&input);

    let exported_function = generate_exported_function(
        &tuple_struct_name,
        tuple_struct_fields,
        &camel_case_string,
        &generated_function_ident,
        params.clone(),
        conversion_code.clone(),
        converted_params.clone(),
        &original_function_name,
        result_not_wrapped,
    );

    let result = quote! {
        #input
        #exported_function
    };

    result.into()
}
