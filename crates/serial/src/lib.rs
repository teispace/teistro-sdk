//! One JSON document for a chart, and one way of writing it that two
//! bindings agree on byte for byte.
//!
//! Three parts:
//!
//! - the **seal** ([`seal`]), which pairs a value with its provenance
//!   and computes the one field a caller cannot fill by hand;
//! - the **canonical form** ([`canonical`]), whose bytes the content
//!   hash is taken over and which a consumer stores;
//! - the **document** ([`document`]), which holds everything the chart
//!   layer computes under one envelope.
//!
//! Three things the falsification pass found, and each shapes the module
//! (`03-design/serial-measured.md`, over
//! `03-design/serial-and-the-envelope.md`):
//!
//! - **The content hash was the hash of nothing.** `Provenance::new`
//!   sets it to `Hash::of(&[])` as a placeholder, and one producer of
//!   three ever replaced it — so a founded chart went out claiming a
//!   hash of the empty string. [`seal::Sealed::new`] is the only
//!   constructor and it computes the hash, so a stale one is not
//!   representable.
//! - **A number needs a stated format.** Rust's JSON layer writes
//!   `1e-6` where JavaScript's writes `0.000001`, so two bindings that
//!   agree about the number disagree about the bytes. The canonical
//!   grammar has no exponent at all.
//! - **The form itself is canonical**: keys in code-point order at every
//!   depth, the same bytes twice, and the same bytes however the value's
//!   own maps were ordered — measured over the corpus's 55 documents.
//!
//! And two forms rather than one: the **hash form** is fixed and
//! settings-independent, because a hash that moves with a display
//! setting is a worse cache key; the **rendered form** honours
//! `output.precision`, which is what that knob is for and which nothing
//! had read.
//!
//! ```
//! use teistro_core::envelope::{Hash, Provenance, Version};
//! use teistro_serial::canonical::to_hash_form;
//! use teistro_serial::seal::Sealed;
//!
//! // Never an exponent, so a binding needs no float printer of its own.
//! assert_eq!(to_hash_form(&1e-6_f64), "0.000001");
//!
//! // And a sealed value carries its own hash rather than a placeholder.
//! let provenance = Provenance::new(
//!     Version::new(0, 1, 0), 1, 1, "parashari-classical",
//!     Hash::of(b"settings"), Hash::of(b"input"),
//! );
//! let sealed = Sealed::new(vec![1.5_f64], provenance);
//! assert!(sealed.is_intact());
//! assert_ne!(sealed.content_hash(), Hash::of(&[]));
//! ```

pub mod canonical;
pub mod document;
pub mod seal;

pub use canonical::{hash_of, to_hash_form, to_rendered};
pub use document::Document;
pub use seal::Sealed;
