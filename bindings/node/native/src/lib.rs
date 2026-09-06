//! The Teistro SDK's Node addon: the mechanical layer between JavaScript
//! and the C ABI (`docs/03-design/ffi-abi-and-api-description.md`).
//!
//! Everything here is generated from `idl/api.json` by `cargo xtask gen
//! ffi`; this file exists to name the module and to say what is not
//! generated, which is nothing. The ergonomic layer that a consumer
//! actually uses is `bindings/node/lib/index.js`, hand-written and thin.

mod generated;

pub use generated::*;
