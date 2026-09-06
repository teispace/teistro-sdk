//! The API description of the Teistro SDK's C boundary and the toolchain
//! over it (`docs/03-design/ffi-abi-and-api-description.md`, ADR-0007,
//! ADR-0023).
//!
//! One description, extracted from the Rust source of the boundary crates,
//! drives every binding: the C header, the Node glue, the TypeScript
//! surface, the Dart layer and the result-blob decoders are all rendered
//! from the same [`model::Api`], so no two bindings can disagree about a
//! type, a unit or a field's documentation.
//!
//! - [`model`]: the description itself, serialised as `idl/api.json`;
//! - [`names`]: the one set of naming rules (Rust, C and binding names);
//! - [`layout`]: C layout of every boundary type, so the header can assert
//!   sizes and the tests can hold Rust and C to the same bytes;
//! - [`rules`]: how a parameter's role and a struct's kind are inferred
//!   from types and names, shared by every emitter;
//! - [`blob`]: the result blob (`TSRB`), a schema-driven encoder and
//!   decoder every binding's generated decoder reproduces;
//! - [`extract`] (feature `extract`): the extractor over Rust source, and
//!   [`sdk`], the SDK's own sources and catalogue kinds put through it;
//! - [`emit`]: the emitters, the C header first.
//!
//! ```
//! use teistro_idl::blob::{ColumnData, Reader, Writer};
//! use teistro_idl::model::{BlobSchema, ColumnDef, Scalar, SectionKind, SectionSchema};
//!
//! let schema = BlobSchema {
//!     name: "demo".into(),
//!     id: 7,
//!     doc: "a demonstration".into(),
//!     sections: vec![SectionSchema {
//!         id: 1,
//!         name: "rows".into(),
//!         doc: "two columns".into(),
//!         kind: SectionKind::Columns,
//!         fields: vec![ColumnDef::new("x", Scalar::F64, "abscissa"), ColumnDef::new("n", Scalar::U8, "count")],
//!     }],
//! };
//! let mut writer = Writer::new(&schema);
//! writer.columns("rows", 2, &[ColumnData::F64(&[1.5, 2.5]), ColumnData::U8(&[1, 2])]).expect("the schema's columns");
//! let bytes = writer.finish().expect("every section written");
//! let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");
//! assert_eq!(reader.count("rows"), Some(2));
//! assert_eq!(reader.column("rows", "x").expect("a column").iter().map(|v| v.as_f64()).collect::<Vec<_>>(), [1.5, 2.5]);
//! ```

pub mod blob;
pub mod emit;
#[cfg(feature = "extract")]
pub mod extract;
pub mod layout;
pub mod model;
pub mod names;
pub mod rules;
#[cfg(feature = "extract")]
pub mod sdk;

pub use model::{Api, BlobSchema, ColumnDef, Scalar, SectionKind, SectionSchema, TypeRef};
