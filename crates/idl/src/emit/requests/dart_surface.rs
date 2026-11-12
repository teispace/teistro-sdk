//! Dart: the library's generated surface as data, for the tests that walk
//! all of it (`bindings/dart/test/surface/library.g.dart`).
//!
//! Dart has no reflection a test may lean on (`dart:mirrors` is deprecated
//! from Dart 3.14), so this lists what the tests used to find by
//! reflecting on the library: every enum `catalogue.dart` declares, with
//! its readers as tear-offs, and every type `requests.dart` declares, a
//! class by its constructor and its parameters, a union by its variants.
//! It is rendered from the same description as the library, so a member
//! added tomorrow is in the table the day it is generated, and `check-ffi`
//! holds the table to the generator. The shapes it is written in are the
//! test's own (`test/surface/shapes.dart`).

use std::fmt::Write;

use super::dart::{DART_SHADOWED, is_facade, quoted};
use super::{Types, key_part, variant_name};
use crate::emit::dart::{identifier, preamble};
use crate::model::Api;
use crate::names::binding_type_name;
use crate::records::{RecordField, RecordShape, RecordType};

/// What the table says it is, after the header.
const ABOUT: &str = "/// The library's generated surface as data, rendered from the description
/// the library is rendered from: every enum `catalogue.dart` declares, with
/// its readers, and every type `requests.dart` declares, a class by its
/// constructor and a union by its variants. The tests that walk a whole
/// surface read it here, so a member added tomorrow is walked the day it is
/// generated.
library;

// The catalogue's own library, prefixed: the layer declares a type of its
// own under one of its enums' names (`MoonEvent`), which `teistro.dart`
// then exports in the enum's place.
import 'package:teistro/src/catalogue.dart' as c;
import 'package:teistro/teistro.dart';

import 'shapes.dart';
";

impl Types<'_> {
    /// How a test reads a value of `ty` from its JSON.
    fn surface_shape(&self, ty: &RecordType) -> String {
        match ty {
            RecordType::Bool => "Scalar('bool')".into(),
            RecordType::Integer { .. } => "Scalar('int')".into(),
            RecordType::Number => "Scalar('double')".into(),
            RecordType::Text => "Scalar('String')".into(),
            RecordType::Json => "Scalar('Object')".into(),
            RecordType::Record { name } if DART_SHADOWED.contains(&name.as_str()) => {
                "Scalar('Object')".into()
            }
            RecordType::Record { name } => {
                let name = self
                    .api
                    .requests
                    .enums
                    .get(name)
                    .map_or_else(|| name.clone(), |boundary| binding_type_name(boundary));
                format!("Named({})", quoted(&name))
            }
            RecordType::Member { catalogue } => {
                format!("Named({})", quoted(&self.dart_kind(catalogue)))
            }
            RecordType::Keyed { .. } | RecordType::Pair { .. } => {
                format!("Unread({})", quoted(&self.dart(ty)))
            }
            RecordType::List { of, .. } => format!(
                "ListOf({}, (items) => List<{}>.from(items))",
                self.surface_shape(of),
                self.dart(of)
            ),
            RecordType::Map { of } => format!(
                "MapOf({}, (entries) => Map<String, {}>.from(entries))",
                self.surface_shape(of),
                self.dart(of)
            ),
        }
    }

    /// A class built by its constructor's named parameters, one a field.
    fn surface_class(
        &self,
        out: &mut String,
        name: &str,
        parent: Option<&str>,
        fields: &[RecordField],
    ) {
        let params: Vec<String> = fields
            .iter()
            .map(|field| {
                format!(
                    "Param({}, {})",
                    quoted(&identifier(&field.name)),
                    self.surface_shape(&field.ty)
                )
            })
            .collect();
        let read: Vec<String> = fields
            .iter()
            .map(|field| format!("v.{}", identifier(&field.name)))
            .collect();
        surface_built(out, name, parent, &params, &read);
    }

    /// A variant holding one value, its constructor's one positional
    /// parameter.
    fn surface_one(&self, out: &mut String, name: &str, parent: &str, ty: &RecordType) {
        let param = format!("Param('value', {}, named: false)", self.surface_shape(ty));
        surface_built(out, name, Some(parent), &[param], &["v.value".into()]);
    }
}

/// A class's entry: its name, its constructor as a tear-off, its
/// parameters, the union it is a variant of, and the fields its equality
/// compares.
fn surface_built(
    out: &mut String,
    name: &str,
    parent: Option<&str>,
    params: &[String],
    read: &[String],
) {
    let parent = parent.map_or_else(String::new, |parent| {
        format!(", parent: {}", quoted(parent))
    });
    let params = if params.is_empty() {
        "const []".to_owned()
    } else {
        format!("[\n    {},\n  ]", params.join(",\n    "))
    };
    let fields = if read.is_empty() {
        "(_) => const []".to_owned()
    } else {
        format!("(v) => [{}]", read.join(", "))
    };
    let _ = writeln!(
        out,
        "  Built<{name}>({}, {name}.new, {params}{parent}, fields: {fields}),",
        quoted(name)
    );
}

/// A union's root, whose variants name it as their parent.
fn surface_sealed(out: &mut String, name: &str) {
    let _ = writeln!(out, "  Sealed({}),", quoted(name));
}

/// `test/surface/library.g.dart`: the catalogue's enums and the request
/// types, as the tests that walk them read them.
///
/// ```
/// # use teistro_idl::Api;
/// #
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
///
/// use teistro_idl::emit::requests::dart_surface;
///
/// let table = dart_surface(&api);
/// // A catalogue enum with its readers, open to a member registered at run time.
/// assert!(table.contains("Catalogued<c.Graha>('Graha', c.Graha.values, c.Graha.byId, c.Graha.byKey, (m) => m.id, (m) => m.key, c.Graha.registered),"));
/// // A closed set's, which no context registers into.
/// assert!(table.contains("Catalogued<c.Kind>('Kind', c.Kind.values, c.Kind.byId, c.Kind.byKey, (m) => m.id, (m) => m.key),"));
/// // `idl/api.json` carries no requests (`gen ffi` reads them from the
/// // façade), so this table lists none of them.
/// assert!(table.ends_with("final List<Surface> requests = [\n];\n"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn dart_surface(api: &Api) -> String {
    let types = Types::of(api);
    let mut out = preamble(api, "");
    out.push_str(ABOUT);
    out.push_str(
        "\n/// Every enum `catalogue.dart` declares, in its order: a catalogued kind's\n/// with its `registered`, a closed set's without.\nfinal List<Catalogued<Enum>> catalogue = [\n",
    );
    for e in &api.enums {
        let name = binding_type_name(&e.name);
        let registered = if e.kind.is_some() {
            format!(", c.{name}.registered")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  Catalogued<c.{name}>({}, c.{name}.values, c.{name}.byId, c.{name}.byKey, (m) => m.id, (m) => m.key{registered}),",
            quoted(&name)
        );
    }
    out.push_str(
        "];\n\n/// Every type `requests.dart` declares, in its order: a key set by its\n/// members, a class by its constructor, a union by its root and then its\n/// variants.\nfinal List<Surface> requests = [\n",
    );
    for record in &api.requests.records {
        let name = record.name.as_str();
        if api.requests.enums.contains_key(name) || is_facade(name) || DART_SHADOWED.contains(&name)
        {
            continue;
        }
        match &record.shape {
            RecordShape::Object { fields } => types.surface_class(&mut out, name, None, fields),
            RecordShape::Beside { fields, laid } => {
                let mut all: Vec<RecordField> = fields.iter().chain([laid]).cloned().collect();
                all.sort_by(|a, b| a.name.cmp(&b.name));
                types.surface_class(&mut out, name, None, &all);
            }
            RecordShape::Keys { .. } => {
                let _ = writeln!(
                    out,
                    "  KeySet<{name}>({}, {name}.values, (m) => m.key),",
                    quoted(name)
                );
            }
            RecordShape::Tagged { variants, .. } => {
                surface_sealed(&mut out, name);
                for variant in variants {
                    types.surface_class(
                        &mut out,
                        &variant_name(name, variant),
                        Some(name),
                        &variant.fields,
                    );
                }
            }
            RecordShape::External { variants } => {
                surface_sealed(&mut out, name);
                for variant in variants {
                    let class = format!("{name}{}", key_part(&variant.key));
                    match &variant.payload {
                        None => surface_built(&mut out, &class, Some(name), &[], &[]),
                        Some(payload) => types.surface_one(&mut out, &class, name, payload),
                    }
                }
            }
            RecordShape::OrNamed {
                names,
                values,
                member: kind,
            } => {
                surface_sealed(&mut out, name);
                for named in names {
                    let class = format!("{name}{}", key_part(named));
                    surface_built(&mut out, &class, Some(name), &[], &[]);
                }
                if let Some(kind) = kind {
                    let class = format!("{name}Member");
                    types.surface_one(
                        &mut out,
                        &class,
                        name,
                        &RecordType::Member {
                            catalogue: kind.clone(),
                        },
                    );
                }
                let several = values.len() > 1;
                for value in values {
                    let class = format!("{name}{}", value.class_suffix(several));
                    types.surface_one(&mut out, &class, name, &value.ty);
                }
            }
        }
    }
    out.push_str("];\n");
    out
}
