#[macro_use]
extern crate pest_derive;

mod utils;
mod query;
mod parse;
mod resolve;
mod yul;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(query: String) -> Result<String, String> { 
    let query = parse::parse_query(&query)?;
    // format!("{:?}", query)
    let yul = yul::derive_yul(query)?;

    return Ok(yul);
}
