use crate::alloc::string::ToString;
use serde::{Deserialize, Serialize};
use traceable_result::NotOwnedType;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct StructWithNotOwnedTypes {
    #[serde(with = "NotOwnedTypeDef")]
    #[tsify(type = "string")]
    pub not_owned_type: NotOwnedType,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "NotOwnedType")]
struct NotOwnedTypeDef {
    name: [u8; 32],
}
