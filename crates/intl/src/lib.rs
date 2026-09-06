//! Teistro Intl: one localisation standard for every string the SDK
//! renders and for the applications that adopt it
//! (`docs/03-design/intl-engine-and-packs.md`, ADR-0010).
//!
//! - [`mf2`]: the stable `MessageFormat 2` data model, parser, checks and
//!   serialiser;
//! - [`source`]: the `i18n/` conventions, `_meta.json`, namespaces, keys,
//!   entity records;
//! - [`analysis`]: the signature a message declares (its parameters with
//!   the types its functions imply, its selectors, its links);
//! - [`render`]: evaluation with the SDK's functions, plural rules from
//!   ICU4X, numbering systems, fallback chains and provenance;
//! - [`validate`]: the gates a source tree passes before it is built, the
//!   SDK's catalogue being the authority for every entity key;
//! - [`pack`]: the `.tpack` container and the locale bundle, zero-copy
//!   reads with a checksum;
//! - [`generate`]: typed accessors for TypeScript, Dart and Rust, keys and
//!   parameter shapes only, never text;
//! - [`messages`]: the SDK's own namespaces as typed Rust accessors,
//!   generated from `i18n/en-Latn` by `cargo xtask gen intl`;
//! - [`cli`]: the `teistro-intl` command line as library functions.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_intl::{Intl, Value, params, sdk_root};
//! use teistro_intl::source::Tree;
//!
//! let tree = Tree::load(&sdk_root()).expect("the SDK's sources");
//! let mut intl = Intl::from_tree(&tree).expect("plural rules for every locale");
//! intl.set_locale("ne-Deva-NP").expect("a shipped locale");
//! let rendered = intl.render(
//!     "sdk.reason.grahaInBhava",
//!     &params([("graha", Value::catalogued(Graha::Jupiter)), ("bhava", Value::Int(7))]),
//! );
//! assert!(rendered.warnings.is_empty());
//! assert_eq!(rendered.resolved_from.as_deref(), Some("ne-Deva-NP"));
//! assert!(rendered.text.contains('७'));
//! ```

pub mod analysis;
#[cfg(feature = "cli")]
pub mod cli;
pub mod generate;
pub mod mf2;
pub mod pack;
pub mod render;
pub mod source;
pub mod validate;

/// The SDK's own namespaces as typed accessors, generated from
/// `i18n/en-Latn` by `cargo xtask gen intl` and held by `check-intl`.
#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic, reason = "generated")]
pub mod messages;

pub use render::{Intl, OutPart, Params, Rendered, TypedMessage, Value, params};
pub use source::{BASE_LOCALE, ENTITY_NAMESPACE, sdk_root};
