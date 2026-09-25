//! The Teistro SDK's wasm module: the mechanical layer between JavaScript
//! and the C ABI, compiled to wasm32 (`docs/03-design/wasm-binding.md`).
//!
//! The glue is generated from `idl/api.json` by `cargo xtask gen ffi`, by
//! the same emitter as the Node addon's and with the same members, so the
//! hand-written `index.js` above either is one file. What is not generated
//! is the host provider's adapter, `provider.rs`, because each binding
//! wraps its own callback mechanism.

mod generated;
mod provider;

pub use generated::*;
pub use provider::ProviderInfo;
