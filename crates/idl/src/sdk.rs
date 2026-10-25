//! The SDK's own description: which sources the boundary is read from,
//! the catalogue's kinds as enums, and one function that puts them
//! together, shared by `cargo xtask gen ffi` and the boundary crate's own
//! tests so the two can never describe different APIs.

use std::path::Path;

use serde::Deserialize;

use crate::extract::{ExtractError, Inputs, Source, extract};
use crate::model::{Api, BlobSchema, EnumDef, EnumValue, Scalar};
use crate::names::pascal;

/// The symbol prefix.
///
/// ```
/// use teistro_idl::names;
/// use teistro_idl::sdk::PREFIX;
///
/// assert_eq!(names::c_type_name(PREFIX, "TsContext"), "ts_context");
/// ```
pub const PREFIX: &str = "ts_";

/// The constant holding the ABI version.
///
/// ```
/// use teistro_idl::sdk::{ABI_VERSION_CONSTANT, PREFIX};
///
/// // The prefix, upper-cased, then the words.
/// assert!(ABI_VERSION_CONSTANT.starts_with(&PREFIX.to_ascii_uppercase()));
/// ```
pub const ABI_VERSION_CONSTANT: &str = "TS_ABI_VERSION";

/// The sources the boundary is read from, repository-relative, in the
/// order their items appear in the header: the status, the port's
/// bodies, scales and vtable, then the C ABI crate module by module.
///
/// ```
/// # use std::path::Path;
/// #
/// # let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
///
/// use teistro_idl::sdk::SOURCES;
///
/// // The status first, and every one a file in the checkout.
/// assert_eq!(SOURCES[0], "crates/core/src/error.rs");
/// assert!(SOURCES.iter().all(|relative| root.join(relative).is_file()));
/// ```
pub const SOURCES: [&str; 26] = [
    "crates/core/src/error.rs",
    "crates/port-ephemeris/src/body.rs",
    "crates/port-ephemeris/src/capabilities.rs",
    "crates/port-ephemeris/src/frame.rs",
    "crates/port-ephemeris/src/vtable.rs",
    "crates/ffi/src/lib.rs",
    "crates/ffi/src/string.rs",
    "crates/ffi/src/blob.rs",
    "crates/ffi/src/context.rs",
    "crates/ffi/src/key.rs",
    "crates/ffi/src/frame.rs",
    "crates/ffi/src/calendar.rs",
    "crates/ffi/src/day.rs",
    "crates/ffi/src/chart_request.rs",
    "crates/ffi/src/chart.rs",
    "crates/ffi/src/time.rs",
    "crates/ffi/src/intl.rs",
    "crates/ffi/src/positions.rs",
    "crates/ffi/src/panchanga.rs",
    "crates/ffi/src/naam.rs",
    "crates/ffi/src/numerology.rs",
    "crates/ffi/src/pakshi.rs",
    "crates/ffi/src/rashifal.rs",
    "crates/ffi/src/research.rs",
    "crates/ffi/src/ephemeris.rs",
    "crates/ffi/src/provider.rs",
];

/// The catalogue's generated JSON, relative to the repository root.
///
/// ```
/// # use std::path::Path;
/// #
/// # let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
///
/// use teistro_idl::sdk::{CATALOGUE_JSON, kinds_from_catalogue};
///
/// let kinds = kinds_from_catalogue(&std::fs::read_to_string(root.join(CATALOGUE_JSON))?)?;
/// assert_eq!(kinds[0].name, "Kind");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const CATALOGUE_JSON: &str = "catalogue/catalogue.json";

#[derive(Deserialize)]
struct CatalogueFile {
    kinds: Vec<KindRecord>,
}

#[derive(Deserialize)]
struct KindRecord {
    kind: String,
    number: u8,
    doc: String,
    #[serde(default)]
    members: Vec<MemberRecord>,
}

#[derive(Deserialize)]
struct MemberRecord {
    key: String,
    id: u16,
    doc: String,
    #[serde(default)]
    deprecated: bool,
}

/// The catalogue's kinds as enums: `Kind` itself over the kind numbers,
/// then one `u16` enum per kind whose members are the catalogue's keys and
/// ids, each marked with its kind so the emitters know which union a
/// member belongs to (`docs/03-design/core-types-and-catalogue.md`, §3.6).
///
/// ```
/// use teistro_idl::Scalar;
/// use teistro_idl::sdk::kinds_from_catalogue;
///
/// let json = r#"{"kinds": [{"kind": "graha", "number": 1, "doc": "The grahas.", "members": [
///     {"key": "SUN", "id": 0, "doc": "The Sun"}
/// ]}]}"#;
/// let enums = kinds_from_catalogue(json)?;
/// // `Kind` over the kinds, then one enum per kind with members.
/// assert_eq!((enums[0].name.as_str(), enums[0].repr), ("Kind", Scalar::U8));
/// let graha = &enums[1];
/// assert_eq!((graha.name.as_str(), graha.values[0].name.as_str(), graha.values[0].key.as_str()), ("Graha", "Sun", "SUN"));
/// assert!(kinds_from_catalogue("[]").is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// JSON that is not the catalogue's.
pub fn kinds_from_catalogue(json: &str) -> Result<Vec<EnumDef>, ExtractError> {
    let file: CatalogueFile = serde_json::from_str(json).map_err(|e| ExtractError {
        where_: CATALOGUE_JSON.to_string(),
        detail: e.to_string(),
    })?;
    let mut enums = vec![EnumDef {
        name: String::from("Kind"),
        doc: String::from(
            "A kind: a family of entities sharing one key type. The number is the high half of every packed key id.",
        ),
        repr: Scalar::U8,
        kind: None,
        values: file
            .kinds
            .iter()
            .map(|k| EnumValue {
                name: pascal(&k.kind),
                value: i64::from(k.number),
                doc: k.doc.clone(),
                key: k.kind.clone(),
                deprecated: false,
            })
            .collect(),
        source: CATALOGUE_JSON.to_string(),
    }];
    enums.extend(file.kinds.iter().filter(|k| !k.members.is_empty()).map(|k| EnumDef {
        name: pascal(&k.kind),
        doc: format!(
            "{} Members are the catalogue's ids; the full key id is `(TS_KIND_{} << 16) | member`.",
            k.doc,
            k.kind.to_ascii_uppercase()
        ),
        repr: Scalar::U16,
        kind: Some(k.kind.clone()),
        values: k
            .members
            .iter()
            .map(|m| EnumValue {
                name: pascal(&m.key.to_ascii_lowercase()),
                value: i64::from(m.id),
                doc: m.doc.clone(),
                key: m.key.clone(),
                deprecated: m.deprecated,
            })
            .collect(),
        source: CATALOGUE_JSON.to_string(),
    }));
    Ok(enums)
}

/// The SDK's description from a checkout: the sources, the catalogue's
/// kinds, the blob schemas the boundary crate declares, and its version.
///
/// ```
/// # use std::path::Path;
/// #
/// # let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
///
/// use teistro_idl::Api;
/// use teistro_idl::sdk::describe;
///
/// // The checkout described again is the description it ships.
/// let shipped: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
/// let api = describe(root, shipped.blobs.clone(), &shipped.sdk_version)?;
/// assert_eq!(api.functions, shipped.functions);
/// assert_eq!(api.structs, shipped.structs);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A source that cannot be read or parsed, or any of the extractor's
/// refusals.
pub fn describe(
    root: &Path,
    blobs: Vec<BlobSchema>,
    sdk_version: &str,
) -> Result<Api, ExtractError> {
    let sources = SOURCES
        .iter()
        .map(|relative| Source::read(root, relative))
        .collect::<Result<Vec<_>, _>>()?;
    let catalogue =
        std::fs::read_to_string(root.join(CATALOGUE_JSON)).map_err(|e| ExtractError {
            where_: CATALOGUE_JSON.to_string(),
            detail: e.to_string(),
        })?;
    let inputs = Inputs {
        prefix: PREFIX.to_string(),
        sdk_version: sdk_version.to_string(),
        abi_version_constant: ABI_VERSION_CONSTANT.to_string(),
        extra_enums: kinds_from_catalogue(&catalogue)?,
        blobs,
    };
    extract(&sources, &inputs)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn the_catalogue_kinds_become_enums_with_keys() {
        let json = r#"{"schema": "teistro-catalogue/1", "kinds": [
            {"kind": "graha", "number": 1, "version": 1, "doc": "The grahas.", "members": [
                {"key": "SUN", "id": 0, "doc": "The Sun"},
                {"key": "PURVA_PHALGUNI", "id": 1, "doc": "x", "deprecated": true}
            ]},
            {"kind": "rule", "number": 32, "version": 1, "doc": "Rules at runtime.", "members": []}
        ]}"#;
        let enums = kinds_from_catalogue(json).unwrap();
        assert_eq!(enums.len(), 2);
        assert_eq!(enums[0].name, "Kind");
        assert_eq!(
            enums[0]
                .values
                .iter()
                .map(|v| (v.name.as_str(), v.value))
                .collect::<Vec<_>>(),
            [("Graha", 1), ("Rule", 32)]
        );
        let graha = &enums[1];
        assert_eq!(
            (graha.name.as_str(), graha.repr, graha.kind.as_deref()),
            ("Graha", Scalar::U16, Some("graha"))
        );
        assert_eq!(graha.values[1].name, "PurvaPhalguni");
        assert_eq!(graha.values[1].key, "PURVA_PHALGUNI");
        assert!(graha.values[1].deprecated);
        assert!(kinds_from_catalogue("{}").is_err());
    }
}
