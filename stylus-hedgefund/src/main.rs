#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

#[cfg(not(any(feature = "export-abi", test)))]
#[no_mangle]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    stylus_hedgefund::print_abi("MIT OR Apache-2.0", "pragma solidity ^0.8.19;");
}
