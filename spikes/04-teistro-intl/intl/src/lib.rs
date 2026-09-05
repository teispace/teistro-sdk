//! Spike 4: Teistro Intl.
//!
//! The question: does one opinionated localisation standard (JSON sources
//! per locale per namespace, `MessageFormat 2` syntax with a fixed function
//! set bound to the SDK's types, a base locale as the source of truth,
//! validation, compiled packs and typed accessors generated for every
//! binding) hold up on real content in the product's two first languages,
//! and what does each piece cost?
//!
//! - [`mf2`]: the data model, the parser and its checks, serialisation;
//! - [`source`]: the `i18n/` conventions, `_meta.json`, namespaces, keys;
//! - [`render`]: evaluation with the SDK's functions, plural rules from
//!   ICU4X, numbering systems, fallback chains and provenance;
//! - [`validate`]: the gates a source tree passes before it is built;
//! - [`pack`]: the `.tpack` container, zero-copy reads with a checksum;
//! - [`generate`]: typed accessors for TypeScript and Dart;
//! - [`bench`]: the timing helper of the conformance kit (`crates/ephemeris-kit`), shared with spike 3.

pub mod analysis;
pub mod generate;
pub mod mf2;
pub mod pack;
pub mod render;
pub mod source;
pub mod validate;

pub use teistro_ephemeris_kit::bench;
