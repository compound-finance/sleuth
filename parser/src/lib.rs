#[macro_use]
extern crate pest_derive;

mod utils;
mod query;
mod parse;
mod resolve;
mod abi;
mod yul;
mod source;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(query: String) -> Result<String, String> { 
    let query_set = parse::parse_query_cls(&query)?;
    let resolutions = resolve::resolve(&query_set)?;
    let tuple = abi::get_tuple_abi(&resolutions);
    let yul = yul::derive_yul(resolutions)?;

    Ok(format!("{};{}", tuple, yul))
}
