#![no_std]

extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use quote::{quote, ToTokens};
use syn::parse_macro_input;

mod base;
mod big_ops;
mod by_number;
mod checked_ops;
mod factories;
mod ops;
mod others;
mod structs;
mod uint_casts;
mod utils;

use structs::DecimalCharacteristics;

use crate::uint_casts::{Uint, UintsCastsInput};
use crate::utils::string_to_ident;
use quote::TokenStreamExt;
#[proc_macro]
pub fn impl_units_casts(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as UintsCastsInput);
    let mut uints: Vec<(syn::Ident, usize)> = alloc::vec![];
    let mut expanded = proc_macro2::TokenStream::new();
    input.uints.iter().for_each(|Uint(ident, size)| {
        let count: usize = size.base10_parse().expect("Failed to parse usize");
        expanded.append_all(uint_casts::validate_uint(ident.clone(), count));
        expanded.append_all(uint_casts::impl_uint_casts(
            uints.clone(),
            ident.clone(),
            count,
        ));
        expanded.append_all(uint_casts::impl_primitive_casts(ident.clone(), count));

        uints.push((ident.clone(), count))
    });
    expanded.into()
}

#[proc_macro_attribute]
pub fn decimal(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args_str = attr.to_string();
    let args: Vec<&str> = args_str.split(',').collect();

    let parsed_scale = match args[0].parse::<u8>() {
        Ok(scale) => scale,
        Err(_) => 0,
    };

    let big_type = match args.len() {
        1 => string_to_ident("", "U256"),
        2 => string_to_ident("", args[1].trim()),
        _ => panic!("decimal: invalid number of parameters"),
    };

    assert!(parsed_scale <= 38, "scale too big");

    let k = item.clone();
    let decimal_struct = parse_macro_input!(k as syn::ItemStruct);

    let fields = decimal_struct.fields;
    let first_field = fields.iter().next().unwrap();

    let underlying_type =
        string_to_ident("", first_field.ty.to_token_stream().to_string().as_str());

    let field_name = match first_field.ident.clone() {
        Some(ident) => quote! {#ident},
        None => quote! {0},
    };

    let struct_name = decimal_struct.ident;

    let characteristics = DecimalCharacteristics {
        struct_name: struct_name.clone(),
        field_name: field_name.clone(),
        underlying_type: underlying_type.clone(),
        big_type: big_type.clone(),
        scale: parsed_scale,
    };

    let mut result = proc_macro::TokenStream::from(quote! {});

    result.extend(item.clone());

    result.extend(base::generate_base(characteristics.clone()));
    result.extend(ops::generate_ops(characteristics.clone()));
    result.extend(big_ops::generate_big_ops(characteristics.clone()));
    result.extend(by_number::generate_by_number(characteristics.clone()));
    result.extend(others::generate_others(characteristics.clone()));
    result.extend(factories::generate_factories(characteristics.clone()));
    result.extend(checked_ops::generate_checked_ops(characteristics.clone()));

    result.extend(proc_macro::TokenStream::from(quote! {
        impl #struct_name {
            pub fn is_zero(self) -> bool {
                self.#field_name == #underlying_type::try_from(0).unwrap()
            }
        }
    }));

    result
}
