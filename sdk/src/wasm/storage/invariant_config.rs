use crate::alloc::string::ToString;
use crate::log;
use crate::types::percentage::Percentage;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::TrackableResult;
use tsify::Tsify;
use uint::construct_uint;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

#[derive(Debug, Clone, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct InvariantConfig {
    #[tsify(type = "string")]
    pub admin: String,
    pub protocol_fee: Percentage,
    pub poc_field: PocType,
}

construct_uint! {
    #[derive(Serialize, Deserialize, Tsify)]
    pub struct U256T(4);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PocType(pub U256T);

#[wasm_bindgen(js_name = "receiveBigType")]
pub fn receive_big_type(big: JsValue) -> Result<BigInt, JsValue> {
    log(format!("received: {:?}", big).as_str());
    let mut arr = [0u64; 4];
    let big_int = BigInt::new(&big)?;
    let x = BigInt::as_uint_n(256f64, &big_int);

    // cast to PocType
    for i in 0..4 {
        let shift_amount = BigInt::from(64 * i);
        let x_shifted = &big_int >> shift_amount;
        let mask = BigInt::from(0xFFFF_FFFF_FFFF_FFFFu64);
        let x_masked = x_shifted & &mask;

        let package: u64 = serde_wasm_bindgen::from_value(x_masked.into())?;
        arr[i] = package;
        log(format!("Package: {:?} index: {:?}", package, i).as_str());
    }

    let nested_type: U256T = U256T(arr);
    log(format!("nested_type: {:?}", nested_type).as_str());
    let big_type = PocType(nested_type);
    log(format!("big_type: {:?}", big_type).as_str());

    // cast back to BigInt
    let mut cast_back = BigInt::from(0u64);

    for (index, &value) in arr.iter().enumerate() {
        let value_bigint = BigInt::from(value);
        let shift_amount = BigInt::from(64 * index);
        let intermediate = value_bigint << shift_amount;
        cast_back = cast_back + intermediate;
    }

    Ok(cast_back)
}
