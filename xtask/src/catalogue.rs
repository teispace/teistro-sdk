//! The catalogue generator and its gate: `catalogue/<kind>.yaml` in,
//! `crates/core/src/catalogue/generated/*.rs`, `catalogue/catalogue.json`
//! and `catalogue/entity-skeleton.json` out. `gen catalogue` writes them;
//! `check-catalogue` regenerates in memory and fails on any difference,
//! so the checked-in output can never drift from its sources.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{Map, Value};

/// A YAML mapping, read as JSON reads one: the catalogue's keys are all
/// strings.
type Mapping = Map<String, Value>;

/// An **open** kind: `catalogue/<kind>.yaml` with `open: true` and no
/// members, beside every closed kind's file.
///
/// They were a list inside this generator until a second one was needed,
/// which put a kind where nobody reading `catalogue/` would look
/// (`03-design/state-readings.md` §4).
struct OpenKind {
    name: String,
    number: u8,
    doc: String,
}

const RESERVED_FIELDS: [&str; 6] = ["type", "ref", "move", "self", "super", "crate"];

#[derive(Deserialize)]
struct KindFile {
    kind: String,
    number: u8,
    version: u16,
    doc: String,
    #[serde(default)]
    types: Mapping,
    #[serde(default)]
    attributes: Mapping,
    /// Whether the members arrive at runtime rather than from this file.
    /// An open kind carries the kind and nothing else; the catalogue
    /// cannot enumerate a consumer's rule keys, and would buy nothing by
    /// enumerating a graha and a bhava together
    /// (`03-design/state-readings.md` §4).
    #[serde(default)]
    open: bool,
    #[serde(default)]
    members: Vec<Member>,
}

#[derive(Deserialize)]
struct Member {
    key: String,
    id: u16,
    doc: String,
    #[serde(default)]
    glyph: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    deprecated: bool,
    #[serde(default)]
    attributes: Option<Mapping>,
    sources: Vec<Source>,
    mark: String,
    #[serde(default)]
    unverified: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct Source {
    text: String,
    #[serde(rename = "ref")]
    reference: String,
}

/// The attribute type language.
#[derive(Clone, Debug, PartialEq)]
enum Ty {
    U8,
    U16,
    I32,
    F64,
    Bool,
    Str,
    Key(String),
    Option(Box<Ty>),
    List(Box<Ty>),
    Array(Box<Ty>, usize),
    Composite(String),
}

fn parse_ty(s: &str) -> Result<Ty, String> {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("option<").and_then(|r| r.strip_suffix('>')) {
        return Ok(Ty::Option(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = s.strip_prefix("list<").and_then(|r| r.strip_suffix('>')) {
        return Ok(Ty::List(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = s.strip_prefix("array<").and_then(|r| r.strip_suffix('>')) {
        let (ty, n) = inner
            .rsplit_once(',')
            .ok_or_else(|| format!("array type `{s}` needs a length"))?;
        let n: usize = n
            .trim()
            .parse()
            .map_err(|_| format!("array length in `{s}`"))?;
        return Ok(Ty::Array(Box::new(parse_ty(ty)?), n));
    }
    if let Some(kind) = s.strip_prefix("key:") {
        return Ok(Ty::Key(kind.to_string()));
    }
    Ok(match s {
        "u8" => Ty::U8,
        "u16" => Ty::U16,
        "i32" => Ty::I32,
        "f64" => Ty::F64,
        "bool" => Ty::Bool,
        "str" => Ty::Str,
        other if other.chars().next().is_some_and(char::is_uppercase) => {
            Ty::Composite(other.to_string())
        }
        other => return Err(format!("unknown type `{other}`")),
    })
}

fn rust_ty(ty: &Ty) -> String {
    match ty {
        Ty::U8 => "u8".into(),
        Ty::U16 => "u16".into(),
        Ty::I32 => "i32".into(),
        Ty::F64 => "f64".into(),
        Ty::Bool => "bool".into(),
        Ty::Str => "&'static str".into(),
        Ty::Key(kind) => pascal(kind),
        Ty::Option(inner) => format!("Option<{}>", rust_ty(inner)),
        Ty::List(inner) => format!("&'static [{}]", rust_ty(inner)),
        Ty::Array(inner, n) => format!("[{}; {n}]", rust_ty(inner)),
        Ty::Composite(name) => name.clone(),
    }
}

/// `PURVA_PHALGUNI` to `PurvaPhalguni`, `house_system` to `HouseSystem`.
pub(crate) fn pascal(name: &str) -> String {
    name.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

fn field_name(name: &str) -> String {
    if RESERVED_FIELDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

fn is_key_name(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_uppercase())
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

fn is_kind_name(s: &str) -> bool {
    s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

struct Kind {
    file: KindFile,
    schema: Vec<(String, Ty)>,
    types: Vec<(String, Vec<(String, Ty)>)>,
    members: BTreeSet<String>,
}

/// Everything loaded and validated.
pub(crate) struct Catalogue {
    kinds: Vec<Kind>,
    open: Vec<OpenKind>,
}

fn mapping_str(m: &Mapping, what: &str) -> Result<Vec<(String, String)>, String> {
    m.iter()
        .map(|(k, v)| match v.as_str() {
            Some(v) => Ok((k.clone(), v.to_string())),
            None => Err(format!("{what}: entries must be `name: type`")),
        })
        .collect()
}

impl Catalogue {
    /// Loads `catalogue/*.yaml` and validates the whole.
    pub(crate) fn load(root: &Path) -> Result<Catalogue, Vec<String>> {
        let dir = root.join("catalogue");
        let mut errors = Vec::new();
        let mut kinds: Vec<Kind> = Vec::new();
        let mut open: Vec<OpenKind> = Vec::new();
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map_err(|e| vec![format!("{}: {e}", dir.display())])?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
            .collect();
        files.sort();
        for path in &files {
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let file: KindFile = match serde_saphyr::from_str(&text) {
                Ok(f) => f,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            if file.kind != name {
                errors.push(format!(
                    "{}: `kind` is {} but the file is {name}.yaml",
                    path.display(),
                    file.kind
                ));
            }
            if file.open {
                if !file.members.is_empty() {
                    errors.push(format!(
                        "{}: an open kind carries no members; its members arrive at runtime",
                        path.display()
                    ));
                }
                open.push(OpenKind {
                    name: file.kind,
                    number: file.number,
                    doc: file.doc,
                });
                continue;
            }
            match Kind::new(file) {
                Ok(kind) => kinds.push(kind),
                Err(mut e) => errors.append(&mut e),
            }
        }
        kinds.sort_by_key(|k| k.file.number);
        open.sort_by_key(|k| k.number);
        let catalogue = Catalogue { kinds, open };
        errors.extend(catalogue.validate_whole());
        if errors.is_empty() {
            Ok(catalogue)
        } else {
            Err(errors)
        }
    }

    fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.iter().find(|k| k.file.kind == name)
    }

    fn validate_whole(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut numbers = BTreeMap::new();
        for open in &self.open {
            if let Some(other) = numbers.insert(open.number, open.name.clone()) {
                errors.push(format!(
                    "kind number {} is used by {other} and {}",
                    open.number, open.name
                ));
            }
        }
        for kind in &self.kinds {
            if let Some(other) = numbers.insert(kind.file.number, kind.file.kind.clone()) {
                errors.push(format!(
                    "kind number {} is used by {other} and {}",
                    kind.file.number, kind.file.kind
                ));
            }
            for (name, ty) in kind
                .schema
                .iter()
                .chain(kind.types.iter().flat_map(|(_, f)| f.iter()))
            {
                errors.extend(self.check_ty_refs(ty, &kind.file.kind, name, kind));
            }
            for member in &kind.file.members {
                let prefix = format!("{}.{}", kind.file.kind, member.key);
                match (&member.attributes, kind.schema.is_empty()) {
                    (Some(attrs), false) => {
                        for (name, ty) in &kind.schema {
                            match attrs.get(name.as_str()) {
                                Some(value) => errors.extend(self.check_value(
                                    value,
                                    ty,
                                    kind,
                                    &format!("{prefix}.{name}"),
                                )),
                                None => {
                                    errors.push(format!("{prefix}: attribute `{name}` is missing"));
                                }
                            }
                        }
                        for (k, _) in attrs {
                            let k = k.as_str();
                            if !kind.schema.iter().any(|(n, _)| n == k) {
                                errors.push(format!(
                                    "{prefix}: attribute `{k}` is not in the schema"
                                ));
                            }
                        }
                    }
                    (None, false) => errors.push(format!("{prefix}: attributes are missing")),
                    (Some(_), true) => {
                        errors.push(format!("{prefix}: the kind declares no attributes"));
                    }
                    (None, true) => {}
                }
                for name in &member.unverified {
                    if !kind.schema.iter().any(|(n, _)| n == name) {
                        errors.push(format!(
                            "{prefix}: unverified names `{name}`, which is not an attribute"
                        ));
                    }
                }
            }
        }
        errors
    }

    fn check_ty_refs(&self, ty: &Ty, kind_name: &str, field: &str, kind: &Kind) -> Vec<String> {
        match ty {
            Ty::Key(target) => {
                if self.kind(target).is_none() {
                    vec![format!(
                        "{kind_name}: `{field}` refers to unknown kind `{target}`"
                    )]
                } else {
                    Vec::new()
                }
            }
            Ty::Option(inner) | Ty::List(inner) | Ty::Array(inner, _) => {
                self.check_ty_refs(inner, kind_name, field, kind)
            }
            Ty::Composite(name) => {
                if kind.types.iter().any(|(n, _)| n == name) {
                    Vec::new()
                } else {
                    vec![format!(
                        "{kind_name}: `{field}` uses undeclared composite type `{name}`"
                    )]
                }
            }
            _ => Vec::new(),
        }
    }

    fn check_value(&self, value: &Value, ty: &Ty, kind: &Kind, at: &str) -> Vec<String> {
        let bad = |what: &str| vec![format!("{at}: expected {what}, found {value:?}")];
        match ty {
            Ty::U8 => value
                .as_u64()
                .filter(|v| u8::try_from(*v).is_ok())
                .map_or_else(|| bad("a u8"), |_| Vec::new()),
            Ty::U16 => value
                .as_u64()
                .filter(|v| u16::try_from(*v).is_ok())
                .map_or_else(|| bad("a u16"), |_| Vec::new()),
            Ty::I32 => value
                .as_i64()
                .filter(|v| i32::try_from(*v).is_ok())
                .map_or_else(|| bad("an i32"), |_| Vec::new()),
            Ty::F64 => value
                .as_f64()
                .map_or_else(|| bad("a number"), |_| Vec::new()),
            Ty::Bool => value
                .as_bool()
                .map_or_else(|| bad("a bool"), |_| Vec::new()),
            Ty::Str => value
                .as_str()
                .map_or_else(|| bad("a string"), |_| Vec::new()),
            Ty::Key(target) => match value.as_str() {
                Some(key) => {
                    let known = self.kind(target).is_some_and(|k| k.members.contains(key));
                    if known {
                        Vec::new()
                    } else {
                        vec![format!("{at}: `{key}` is not a member of `{target}`")]
                    }
                }
                None => bad(&format!("a key of `{target}`")),
            },
            Ty::Option(inner) => {
                if value.is_null() {
                    Vec::new()
                } else {
                    self.check_value(value, inner, kind, at)
                }
            }
            Ty::List(inner) => match value.as_array() {
                Some(items) => items
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| self.check_value(v, inner, kind, &format!("{at}[{i}]")))
                    .collect(),
                None => bad("a list"),
            },
            Ty::Array(inner, n) => match value.as_array() {
                Some(items) if items.len() == *n => items
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| self.check_value(v, inner, kind, &format!("{at}[{i}]")))
                    .collect(),
                Some(items) => vec![format!("{at}: expected {n} items, found {}", items.len())],
                None => bad("an array"),
            },
            Ty::Composite(name) => {
                let Some((_, fields)) = kind.types.iter().find(|(n, _)| n == name) else {
                    return vec![format!("{at}: unknown composite `{name}`")];
                };
                let Some(map) = value.as_object() else {
                    return bad(&format!("a `{name}` object"));
                };
                let mut errors = Vec::new();
                for (field, fty) in fields {
                    match map.get(field.as_str()) {
                        Some(v) => {
                            errors.extend(self.check_value(v, fty, kind, &format!("{at}.{field}")));
                        }
                        None => errors.push(format!("{at}: `{name}` needs `{field}`")),
                    }
                }
                errors
            }
        }
    }

    /// Every output, path relative to the root, with its content.
    pub(crate) fn render(&self) -> BTreeMap<PathBuf, String> {
        let mut out = BTreeMap::new();
        let generated = Path::new("crates/core/src/catalogue/generated");
        for kind in &self.kinds {
            out.insert(
                generated.join(format!("{}.rs", kind.file.kind)),
                Self::render_kind(kind),
            );
        }
        out.insert(generated.join("kinds.rs"), self.render_kinds());
        out.insert(generated.join("mod.rs"), self.render_mod());
        out.insert(
            PathBuf::from("catalogue/catalogue.json"),
            self.render_json(),
        );
        out.insert(
            PathBuf::from("catalogue/entity-skeleton.json"),
            self.render_skeleton(),
        );
        out
    }

    fn render_mod(&self) -> String {
        let mut s = String::from(
            "//! Generated by `cargo xtask gen catalogue`; do not edit. One module per kind,\n//! the `Kind` enumeration, and the resolvers over every kind.\n\n",
        );
        s.push_str("mod kinds;\n");
        for kind in &self.kinds {
            let _ = writeln!(s, "mod {};", kind.file.kind);
        }
        s.push_str("\npub use kinds::Kind;\n");
        for kind in &self.kinds {
            let _ = writeln!(s, "pub use {}::*;", kind.file.kind);
        }
        s.push_str("\nuse crate::key::KeyId;\n\n");
        s.push_str("/// Resolves a key inside a kind to its id, over every catalogued kind.\n");
        s.push_str(&self.resolver_example("resolve"));
        s.push_str("#[must_use]\npub fn resolve(kind: Kind, key: &str) -> Option<KeyId> {\n    match kind {\n");
        for kind in &self.kinds {
            let _ = writeln!(
                s,
                "        Kind::{p} => {p}::from_key(key).map({p}::key_id),",
                p = pascal(&kind.file.kind)
            );
        }
        s.push_str("        _ => None,\n    }\n}\n\n");
        s.push_str("/// The key of a catalogued id, over every catalogued kind.\n");
        s.push_str(&self.resolver_example("key_of"));
        s.push_str("#[must_use]\npub fn key_of(id: KeyId) -> Option<&'static str> {\n    match id.kind()? {\n");
        for kind in &self.kinds {
            let _ = writeln!(
                s,
                "        Kind::{p} => {p}::from_id(id.id()).map({p}::key),",
                p = pascal(&kind.file.kind)
            );
        }
        s.push_str("        _ => None,\n    }\n}\n");
        s
    }

    /// The example on `resolve` or `key_of`: the first catalogued kind's
    /// first member, and an open kind, which resolves nothing.
    fn resolver_example(&self, function: &str) -> String {
        let Some((kind, first)) = self
            .kinds
            .iter()
            .find_map(|kind| kind.file.members.first().map(|first| (kind, first)))
        else {
            return String::new();
        };
        let (name, key) = (pascal(&kind.file.kind), &first.key);
        let at = format!("{name}::{}", pascal(key));
        let mut code = format!("use teistro_core::catalogue::{{{function}, Kind, {name}}};\n\n");
        if function == "resolve" {
            let _ = write!(
                code,
                "assert_eq!(resolve(Kind::{name}, \"{key}\"), Some({at}.key_id()));\nassert_eq!(resolve(Kind::{name}, \"NOT_A_KEY\"), None);"
            );
            if let Some(open) = self.open.first() {
                let _ = write!(
                    code,
                    "\n// An open kind's members arrive at runtime, so none resolves here.\nassert_eq!(resolve(Kind::{}, \"{key}\"), None);",
                    pascal(&open.name)
                );
            }
        } else {
            let _ = write!(code, "assert_eq!(key_of({at}.key_id()), Some(\"{key}\"));");
        }
        doc_example("", &code)
    }

    fn render_kinds(&self) -> String {
        let mut all: Vec<(String, u8, String, usize)> = self
            .kinds
            .iter()
            .map(|k| {
                (
                    k.file.kind.clone(),
                    k.file.number,
                    k.file.doc.clone(),
                    k.file.members.len(),
                )
            })
            .collect();
        for open in &self.open {
            all.push((open.name.clone(), open.number, open.doc.clone(), 0));
        }
        all.sort_by_key(|k| k.1);
        let mut s = String::from(
            "//! Generated by `cargo xtask gen catalogue`; do not edit.\n\n#![allow(clippy::match_same_arms, reason = \"generated tables\")]\n\n",
        );
        s.push_str("/// A kind: a family of entities sharing one key type. The number is part of\n/// the C boundary and is never reused.\n");
        s.push_str(&kind_example(&all, &self.open, ""));
        s.push_str("#[repr(u8)]\n#[non_exhaustive]\n#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum Kind {\n");
        for (name, number, doc, _) in &all {
            let _ = writeln!(s, "    /// {doc}\n    {} = {number},", pascal(name));
        }
        s.push_str("}\n\n");
        let mut sorted: Vec<&(String, u8, String, usize)> = all.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let _ = writeln!(s, "const BY_NAME: [(&str, Kind); {}] = [", all.len());
        for (name, _, _, _) in &sorted {
            let _ = writeln!(s, "    (\"{name}\", Kind::{}),", pascal(name));
        }
        s.push_str("];\n\nimpl Kind {\n");
        let _ = writeln!(
            s,
            "    /// Every kind, by number.\n    pub const ALL: [Kind; {}] = [",
            all.len()
        );
        for (name, _, _, _) in &all {
            let _ = writeln!(s, "        Kind::{},", pascal(name));
        }
        s.push_str(
            "    ];\n\n    /// The kind's name, the first segment of its members' full keys.\n",
        );
        s.push_str(&kind_example(&all, &self.open, "name"));
        s.push_str("    #[must_use]\n    pub const fn name(self) -> &'static str {\n        match self {\n");
        for (name, _, _, _) in &all {
            let _ = writeln!(s, "            Kind::{} => \"{name}\",", pascal(name));
        }
        s.push_str("        }\n    }\n\n    /// The kind's number at the C boundary.\n");
        s.push_str(&kind_example(&all, &self.open, "number"));
        s.push_str("    #[must_use]\n    pub const fn number(self) -> u8 {\n        self as u8\n    }\n\n    /// The number of catalogued members; zero for an open kind.\n");
        s.push_str(&kind_example(&all, &self.open, "count"));
        s.push_str(
            "    #[must_use]\n    pub const fn count(self) -> usize {\n        match self {\n",
        );
        for (name, _, _, count) in &all {
            let _ = writeln!(s, "            Kind::{} => {count},", pascal(name));
        }
        s.push_str("        }\n    }\n\n    /// Whether members arrive at runtime rather than from the catalogue.\n");
        s.push_str(&kind_example(&all, &self.open, "is_open"));
        s.push_str("    #[must_use]\n    pub const fn is_open(self) -> bool {\n        self.count() == 0\n    }\n\n    /// The kind with a number.\n");
        s.push_str(&kind_example(&all, &self.open, "from_number"));
        s.push_str("    #[must_use]\n    pub const fn from_number(number: u8) -> Option<Kind> {\n        match number {\n");
        for (name, number, _, _) in &all {
            let _ = writeln!(s, "            {number} => Some(Kind::{}),", pascal(name));
        }
        s.push_str("            _ => None,\n        }\n    }\n\n    /// The kind with a name.\n");
        s.push_str(&kind_example(&all, &self.open, "from_name"));
        s.push_str("    #[must_use]\n    pub fn from_name(name: &str) -> Option<Kind> {\n        crate::catalogue::lookup(&BY_NAME, name)\n    }\n}\n\nimpl core::fmt::Display for Kind {\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {\n        f.write_str(self.name())\n    }\n}\n");
        s.push_str("\nimpl serde::Serialize for Kind {\n    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {\n        serializer.serialize_str(self.name())\n    }\n}\n\nimpl<'de> serde::Deserialize<'de> for Kind {\n    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {\n        let name = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;\n        Kind::from_name(&name).ok_or_else(|| serde::de::Error::custom(crate::catalogue::UnknownKey::kind_name(&name)))\n    }\n}\n\n/// The kind names the reader above accepts, and no others.\n#[cfg(feature = \"schema\")]\nimpl schemars::JsonSchema for Kind {\n    fn schema_name() -> std::borrow::Cow<'static, str> {\n        std::borrow::Cow::Borrowed(\"Kind\")\n    }\n\n    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {\n        crate::catalogue::names_schema(\n            \"A catalogue kind, by name.\",\n            Kind::ALL.iter().map(|kind| kind.name()),\n        )\n    }\n}\n");
        s
    }

    #[allow(clippy::too_many_lines, reason = "one linear emitter per kind file")]
    fn render_kind(kind: &Kind) -> String {
        let file = &kind.file;
        let name = pascal(&file.kind);
        let n = file.members.len();
        let mut s = String::new();
        let _ = writeln!(
            s,
            "//! Generated by `cargo xtask gen catalogue` from `catalogue/{}.yaml` (version {}); do not edit.\n\n#![allow(clippy::match_same_arms, clippy::unreadable_literal, clippy::too_many_lines, reason = \"generated tables\")]\n",
            file.kind, file.version
        );
        s.push_str("use crate::catalogue::{Catalogued, Mark, Source, UnknownKey};\nuse crate::key::{Kind, KeyId};\n");
        let mut referenced: BTreeSet<String> = BTreeSet::new();
        for (_, ty) in kind
            .schema
            .iter()
            .chain(kind.types.iter().flat_map(|(_, f)| f.iter()))
        {
            collect_keys(ty, &mut referenced);
        }
        referenced.remove(&file.kind);
        if !referenced.is_empty() {
            let list: Vec<String> = referenced.iter().map(|k| pascal(k)).collect();
            let _ = writeln!(s, "use super::{{{}}};", list.join(", "));
        }
        let _ = writeln!(
            s,
            "\n/// {}\n///\n/// Members are appended only; the discriminants are the catalogue ids.\n{}#[repr(u16)]\n#[non_exhaustive]\n#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum {name} {{",
            file.doc,
            member_example(&name, file, "")
        );
        for m in &file.members {
            let _ = writeln!(
                s,
                "    /// {}{}\n    {} = {},",
                m.doc,
                if m.deprecated { " (deprecated)" } else { "" },
                pascal(&m.key),
                m.id
            );
        }
        s.push_str("}\n");
        for (tname, fields) in &kind.types {
            let _ = write!(
                s,
                "\n/// A value type of the `{}` attributes.\n{}#[derive(Clone, Copy, Debug, PartialEq)]\npub struct {tname} {{\n",
                file.kind,
                value_type_example(&name, kind, tname)
            );
            for (fname, fty) in fields {
                let _ = writeln!(
                    s,
                    "    /// `{fname}`.\n    pub {}: {},",
                    field_name(fname),
                    rust_ty(fty)
                );
            }
            s.push_str("}\n");
        }
        if !kind.schema.is_empty() {
            let _ = write!(
                s,
                "\n/// The attributes of a `{}` member.\n{}#[derive(Clone, Copy, Debug, PartialEq)]\npub struct {name}Attributes {{\n",
                file.kind,
                attributes_example(&name, kind, "")
            );
            for (fname, fty) in &kind.schema {
                let _ = writeln!(
                    s,
                    "    /// `{fname}`.\n    pub {}: {},",
                    field_name(fname),
                    rust_ty(fty)
                );
            }
            s.push_str("}\n");
            let _ = writeln!(s, "\nstatic ATTRIBUTES: [{name}Attributes; {n}] = [");
            for m in &file.members {
                let attrs = m
                    .attributes
                    .as_ref()
                    .map_or_else(Mapping::new, Clone::clone);
                let _ = write!(s, "    {name}Attributes {{");
                for (fname, fty) in &kind.schema {
                    let value = attrs.get(fname.as_str()).cloned().unwrap_or(Value::Null);
                    let _ = write!(
                        s,
                        " {}: {},",
                        field_name(fname),
                        Self::render_value(&value, fty, kind)
                    );
                }
                s.push_str(" },\n");
            }
            s.push_str("];\n");
        }
        let mut sorted: Vec<&Member> = file.members.iter().collect();
        sorted.sort_by(|a, b| a.key.cmp(&b.key));
        let _ = writeln!(s, "\nconst BY_KEY: [(&str, {name}); {n}] = [");
        for m in &sorted {
            let _ = writeln!(s, "    (\"{}\", {name}::{}),", m.key, pascal(&m.key));
        }
        s.push_str("];\n");
        let mut aliases: Vec<(&str, &Member)> = file
            .members
            .iter()
            .flat_map(|m| m.aliases.iter().map(move |a| (a.as_str(), m)))
            .collect();
        aliases.sort_by(|a, b| a.0.cmp(b.0));
        let _ = writeln!(
            s,
            "\nconst ALIASES: [(&str, {name}); {}] = [",
            aliases.len()
        );
        for (alias, m) in &aliases {
            let _ = writeln!(s, "    (\"{alias}\", {name}::{}),", pascal(&m.key));
        }
        s.push_str("];\n");
        let _ = write!(
            s,
            "\nimpl {name} {{\n    /// The kind.\n    pub const KIND: Kind = Kind::{name};\n\n    /// Every member, in id order.\n    pub const ALL: [{name}; {n}] = [\n"
        );
        for m in &file.members {
            let _ = writeln!(s, "        {name}::{},", pascal(&m.key));
        }
        s.push_str("    ];\n\n");
        Self::render_match(
            &mut s,
            &name,
            file,
            "The key inside the kind (`SUN`).",
            "key",
            "&'static str",
            |m| format!("\"{}\"", m.key),
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "The full key (`graha.SUN`).",
            "full_key",
            "&'static str",
            |m| format!("\"{}.{}\"", file.kind, m.key),
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "The member's documentation.",
            "doc",
            "&'static str",
            |m| format!("{:?}", m.doc),
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "The glyph, when the member has one.",
            "glyph",
            "Option<&'static str>",
            |m| {
                m.glyph
                    .as_ref()
                    .map_or_else(|| "None".into(), |g| format!("Some({g:?})"))
            },
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "The confidence mark (ADR-0018).",
            "mark",
            "Mark",
            |m| format!("Mark::{}", mark_name(&m.mark)),
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "Whether the member is deprecated; it still resolves.",
            "deprecated",
            "bool",
            |m| m.deprecated.to_string(),
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "The sources the member cites.",
            "sources",
            "&'static [Source]",
            |m| {
                let items: Vec<String> = m
                    .sources
                    .iter()
                    .map(|src| {
                        format!(
                            "Source {{ text: {:?}, reference: {:?} }}",
                            src.text, src.reference
                        )
                    })
                    .collect();
                format!("&[{}]", items.join(", "))
            },
        );
        Self::render_match(
            &mut s,
            &name,
            file,
            "Attributes whose values are traditional inside a verified row.",
            "unverified",
            "&'static [&'static str]",
            |m| {
                let items: Vec<String> = m.unverified.iter().map(|u| format!("{u:?}")).collect();
                format!("&[{}]", items.join(", "))
            },
        );
        let (id, key_id, from_key, from_id) = (
            member_example(&name, file, "id"),
            member_example(&name, file, "key_id"),
            member_example(&name, file, "from_key"),
            member_example(&name, file, "from_id"),
        );
        let _ = write!(
            s,
            "    /// The catalogue id: the discriminant.\n{id}    #[must_use]\n    pub const fn id(self) -> u16 {{\n        self as u16\n    }}\n\n    /// The packed key id for the C boundary.\n{key_id}    #[must_use]\n    pub const fn key_id(self) -> KeyId {{\n        KeyId::new(Kind::{name}, self as u16)\n    }}\n\n    /// The member with a key or a former key.\n{from_key}    #[must_use]\n    pub fn from_key(key: &str) -> Option<{name}> {{\n        crate::catalogue::lookup(&BY_KEY, key).or_else(|| crate::catalogue::lookup(&ALIASES, key))\n    }}\n\n    /// The member with an id.\n{from_id}    #[must_use]\n    pub const fn from_id(id: u16) -> Option<{name}> {{\n        match id {{\n"
        );
        for m in &file.members {
            let _ = writeln!(
                s,
                "            {} => Some({name}::{}),",
                m.id,
                pascal(&m.key)
            );
        }
        s.push_str("            _ => None,\n        }\n    }\n");
        if !kind.schema.is_empty() {
            let _ = write!(
                s,
                "\n    /// The member's attributes.\n{}    #[must_use]\n    #[allow(clippy::indexing_slicing, reason = \"the discriminants are dense from zero\")]\n    pub fn attributes(self) -> &'static {name}Attributes {{\n        &ATTRIBUTES[self as usize]\n    }}\n",
                attributes_example(&name, kind, "    ")
            );
        }
        s.push_str("}\n");
        let _ = write!(
            s,
            "\nimpl Catalogued for {name} {{\n    const KIND: Kind = Kind::{name};\n\n    fn all() -> &'static [Self] {{\n        &Self::ALL\n    }}\n\n    fn key(self) -> &'static str {{\n        self.key()\n    }}\n\n    fn id(self) -> u16 {{\n        self.id()\n    }}\n\n    fn mark(self) -> Mark {{\n        self.mark()\n    }}\n\n    fn from_key(key: &str) -> Option<Self> {{\n        Self::from_key(key)\n    }}\n\n    fn from_id(id: u16) -> Option<Self> {{\n        Self::from_id(id)\n    }}\n}}\n\nimpl core::fmt::Display for {name} {{\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{\n        f.write_str(self.key())\n    }}\n}}\n\nimpl core::str::FromStr for {name} {{\n    type Err = UnknownKey;\n\n    fn from_str(key: &str) -> Result<Self, UnknownKey> {{\n        <Self as Catalogued>::from_either_key(key)\n    }}\n}}\n\nimpl From<{name}> for KeyId {{\n    fn from(member: {name}) -> KeyId {{\n        member.key_id()\n    }}\n}}\n\nimpl TryFrom<KeyId> for {name} {{\n    type Error = UnknownKey;\n\n    fn try_from(id: KeyId) -> Result<Self, UnknownKey> {{\n        match id.kind() {{\n            Some(Kind::{name}) => Self::from_id(id.id()).ok_or_else(|| UnknownKey::id(id)),\n            _ => Err(UnknownKey::id(id)),\n        }}\n    }}\n}}\n\nimpl serde::Serialize for {name} {{\n    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {{\n        serializer.serialize_str(self.key())\n    }}\n}}\n\nimpl<'de> serde::Deserialize<'de> for {name} {{\n    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {{\n        let key = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;\n        <Self as Catalogued>::from_either_key(&key).map_err(serde::de::Error::custom)\n    }}\n}}\n\n/// The keys the reader above accepts, and no others.\n#[cfg(feature = \"schema\")]\nimpl schemars::JsonSchema for {name} {{\n    fn schema_name() -> std::borrow::Cow<'static, str> {{\n        std::borrow::Cow::Borrowed(\"{name}\")\n    }}\n\n    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {{\n        crate::catalogue::key_schema::<{name}>(&ALIASES)\n    }}\n}}\n"
        );
        // Every member through every accessor generated above and back,
        // so a kind's tables are read whole by a test rather than only
        // where some caller happens to look.
        let attributes = if kind.schema.is_empty() {
            ""
        } else {
            "\n            assert_ne!(format!(\"{:?}\", member.attributes()), \"\");"
        };
        let _ = write!(
            s,
            "\n#[cfg(test)]\nmod every_member {{\n    #![allow(clippy::unwrap_used, reason = \"a test fails by panicking\")]\n\n    use super::*;\n\n    #[test]\n    fn every_member_reads_whole_and_comes_back() {{\n        for member in {name}::ALL {{\n            assert_eq!({name}::from_key(member.key()), Some(member));\n            assert_eq!({name}::from_id(member.id()), Some(member));\n            assert_eq!({name}::try_from(member.key_id()), Ok(member));\n            assert_eq!(member.full_key(), format!(\"{{}}.{{}}\", Kind::{name}.name(), member.key()));\n            assert_eq!(member.full_key().parse::<{name}>(), Ok(member));\n            assert_eq!(member.to_string().parse::<{name}>(), Ok(member));\n            let json = serde_json::to_string(&member).unwrap();\n            assert_eq!(serde_json::from_str::<{name}>(&json).unwrap(), member);\n            assert_eq!(<{name} as Catalogued>::mark(member), member.mark());\n            assert_ne!(member.doc(), \"\");\n            assert_ne!(member.glyph(), Some(\"\"));\n            let _ = (member.deprecated(), member.sources(), member.unverified());{attributes}\n        }}\n        assert_eq!({name}::ALL.len(), Kind::{name}.count());\n    }}\n}}\n"
        );
        s
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one emitter for every per-member match"
    )]
    fn render_match(
        s: &mut String,
        name: &str,
        file: &KindFile,
        doc: &str,
        fn_name: &str,
        ret: &str,
        value: impl Fn(&Member) -> String,
    ) {
        let example = member_example(name, file, fn_name);
        let _ = write!(
            s,
            "    /// {doc}\n{example}    #[must_use]\n    pub const fn {fn_name}(self) -> {ret} {{\n        match self {{\n"
        );
        for m in &file.members {
            let _ = writeln!(s, "            {name}::{} => {},", pascal(&m.key), value(m));
        }
        s.push_str("        }\n    }\n\n");
    }

    fn render_value(value: &Value, ty: &Ty, kind: &Kind) -> String {
        match ty {
            Ty::U8 | Ty::U16 | Ty::I32 => {
                value.as_i64().map_or_else(|| "0".into(), |v| v.to_string())
            }
            Ty::F64 => {
                let v = value.as_f64().unwrap_or(0.0);
                if v.fract() == 0.0 {
                    format!("{v:.1}")
                } else {
                    format!("{v:?}")
                }
            }
            Ty::Bool => value.as_bool().unwrap_or(false).to_string(),
            Ty::Str => format!("{:?}", value.as_str().unwrap_or_default()),
            Ty::Key(target) => format!(
                "{}::{}",
                pascal(target),
                pascal(value.as_str().unwrap_or_default())
            ),
            Ty::Option(inner) => {
                if value.is_null() {
                    "None".into()
                } else {
                    format!("Some({})", Self::render_value(value, inner, kind))
                }
            }
            Ty::List(inner) | Ty::Array(inner, _) => {
                let items: Vec<String> = value
                    .as_array()
                    .map(|seq| {
                        seq.iter()
                            .map(|v| Self::render_value(v, inner, kind))
                            .collect()
                    })
                    .unwrap_or_default();
                if matches!(ty, Ty::List(_)) {
                    format!("&[{}]", items.join(", "))
                } else {
                    format!("[{}]", items.join(", "))
                }
            }
            Ty::Composite(name) => {
                let fields = kind
                    .types
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, f)| f.as_slice())
                    .unwrap_or_default();
                let map = value.as_object();
                let parts: Vec<String> = fields
                    .iter()
                    .map(|(fname, fty)| {
                        let v = map
                            .and_then(|m| m.get(fname.as_str()))
                            .cloned()
                            .unwrap_or(Value::Null);
                        format!(
                            "{}: {}",
                            field_name(fname),
                            Self::render_value(&v, fty, kind)
                        )
                    })
                    .collect();
                format!("{name} {{ {} }}", parts.join(", "))
            }
        }
    }

    fn render_json(&self) -> String {
        let kinds: Vec<serde_json::Value> = self
            .kinds
            .iter()
            .map(|k| {
                serde_json::json!({
                    "kind": k.file.kind,
                    "number": k.file.number,
                    "version": k.file.version,
                    "doc": k.file.doc,
                    "types": k.file.types,
                    "attributes": k.file.attributes,
                    "members": k.file.members.iter().map(|m| serde_json::json!({
                        "key": m.key, "id": m.id, "doc": m.doc, "glyph": m.glyph, "aliases": m.aliases,
                        "deprecated": m.deprecated,
                        "attributes": m.attributes,
                        "sources": m.sources.iter().map(|s| serde_json::json!({"text": s.text, "ref": s.reference})).collect::<Vec<_>>(),
                        "mark": m.mark, "unverified": m.unverified,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let open: Vec<serde_json::Value> = self
            .open
            .iter()
            .map(|k| {
                serde_json::json!({"kind": k.name, "number": k.number, "doc": k.doc, "open": true})
            })
            .collect();
        let doc = serde_json::json!({ "schema": "teistro-catalogue/1", "kinds": kinds, "open_kinds": open });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&doc).unwrap_or_default()
        )
    }

    fn render_skeleton(&self) -> String {
        let mut root = serde_json::Map::new();
        for k in &self.kinds {
            let mut members = serde_json::Map::new();
            for m in &k.file.members {
                let mut record = serde_json::Map::new();
                if let Some(g) = &m.glyph {
                    record.insert("glyph".into(), serde_json::Value::String(g.clone()));
                }
                let gender = m.attributes.as_ref().and_then(|a| {
                    a.get("descriptors")
                        .and_then(|d| d.as_object())
                        .and_then(|d| d.get("gender"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| {
                            a.get("parity").and_then(Value::as_str).map(|p| {
                                if p == "ODD" {
                                    "MALE".into()
                                } else {
                                    "FEMALE".into()
                                }
                            })
                        })
                });
                if let Some(g) = gender {
                    let short = match g.as_str() {
                        "MALE" => "m",
                        "FEMALE" => "f",
                        _ => "n",
                    };
                    record.insert("gender".into(), serde_json::Value::String(short.into()));
                }
                members.insert(m.key.clone(), serde_json::Value::Object(record));
            }
            root.insert(k.file.kind.clone(), serde_json::Value::Object(members));
        }
        format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap_or_default()
        )
    }
}

/// A rustdoc example as doc-comment lines at `indent`, opened by a blank
/// doc line so that it follows the item's prose.
fn doc_example(indent: &str, code: &str) -> String {
    let mut out = format!("{indent}///\n{indent}/// ```\n");
    for line in code.lines() {
        if line.is_empty() {
            let _ = writeln!(out, "{indent}///");
        } else {
            let _ = writeln!(out, "{indent}/// {line}");
        }
    }
    let _ = writeln!(out, "{indent}/// ```");
    out
}

/// The variant a catalogue mark letter names.
fn mark_name(mark: &str) -> &'static str {
    match mark {
        "V" => "Verified",
        "T" => "Traditional",
        _ => "Shape",
    }
}

/// The example on a kind's enum (`method` empty) or on one of its
/// per-member methods, asserting the catalogue's own values for the
/// member that shows the method best: the first, or the first with a
/// glyph, an alias, a deprecation or an unverified attribute where the
/// method is about one.
fn member_example(name: &str, file: &KindFile, method: &str) -> String {
    let Some(first) = file.members.first() else {
        return String::new();
    };
    let find =
        |wanted: fn(&Member) -> bool| file.members.iter().find(|m| wanted(m)).unwrap_or(first);
    let path = |m: &Member| format!("{name}::{}", pascal(&m.key));
    let (at, kind, key) = (path(first), &file.kind, &first.key);
    let import = format!("use teistro_core::catalogue::{name};\n\n");
    let code = match method {
        "" => format!(
            "{import}let member: {name} = \"{key}\".parse()?;\nassert_eq!(member, {at});\n// The full key reads too; a member prints as its key.\nassert_eq!(\"{kind}.{key}\".parse::<{name}>(), Ok({at}));\nassert_eq!(member.to_string(), \"{key}\");\n# Ok::<(), teistro_core::catalogue::UnknownKey>(())"
        ),
        "key" => format!("{import}assert_eq!({at}.key(), \"{key}\");"),
        "full_key" => format!("{import}assert_eq!({at}.full_key(), \"{kind}.{key}\");"),
        "doc" => format!("{import}assert_eq!({at}.doc(), {:?});", first.doc),
        "glyph" => {
            let m = find(|m| m.glyph.is_some());
            let glyph = m
                .glyph
                .as_ref()
                .map_or_else(|| String::from("None"), |g| format!("Some({g:?})"));
            format!("{import}assert_eq!({}.glyph(), {glyph});", path(m))
        }
        "mark" => format!(
            "use teistro_core::catalogue::{{Mark, {name}}};\n\nassert_eq!({at}.mark(), Mark::{});",
            mark_name(&first.mark)
        ),
        "deprecated" => {
            let m = find(|m| m.deprecated);
            if m.deprecated {
                format!(
                    "{import}assert!({p}.deprecated());\n// Deprecated, and still read.\nassert_eq!({name}::from_key(\"{}\"), Some({p}));",
                    m.key,
                    p = path(m)
                )
            } else {
                format!("{import}assert!(!{at}.deprecated());")
            }
        }
        "sources" => match first.sources.first() {
            Some(source) => format!(
                "{import}let first = &{at}.sources()[0];\nassert_eq!((first.text, first.reference), ({:?}, {:?}));",
                source.text, source.reference
            ),
            None => format!("{import}assert!({at}.sources().is_empty());"),
        },
        "unverified" => {
            let m = find(|m| !m.unverified.is_empty());
            if m.unverified.is_empty() {
                format!("{import}assert!({at}.unverified().is_empty());")
            } else {
                let items: Vec<String> = m.unverified.iter().map(|u| format!("{u:?}")).collect();
                format!(
                    "{import}assert_eq!({}.unverified(), [{}]);",
                    path(m),
                    items.join(", ")
                )
            }
        }
        "id" => format!("{import}assert_eq!({at}.id(), {});", first.id),
        "key_id" => format!(
            "use teistro_core::catalogue::{{Kind, {name}}};\n\nlet packed = {at}.key_id();\nassert_eq!((packed.kind(), packed.id()), (Some(Kind::{name}), {}));",
            first.id
        ),
        "from_key" => {
            let mut code = format!(
                "{import}assert_eq!({name}::from_key(\"{key}\"), Some({at}));\n// The key inside the kind; `parse` reads the full key as well.\nassert_eq!({name}::from_key(\"{kind}.{key}\"), None);"
            );
            let aliased = file
                .members
                .iter()
                .find_map(|m| m.aliases.first().map(|alias| (m, alias)));
            if let Some((m, alias)) = aliased {
                let _ = write!(
                    code,
                    "\n// A former key still reads.\nassert_eq!({name}::from_key(\"{alias}\"), Some({}));",
                    path(m)
                );
            }
            code
        }
        "from_id" => {
            let past = file
                .members
                .iter()
                .map(|m| m.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            format!(
                "{import}assert_eq!({name}::from_id({}), Some({at}));\nassert_eq!({name}::from_id({past}), None);",
                first.id
            )
        }
        _ => return String::new(),
    };
    doc_example(if method.is_empty() { "" } else { "    " }, &code)
}

/// The example on a kind's attributes, or on its attribute struct: the
/// first member's whole row.
fn attributes_example(name: &str, kind: &Kind, indent: &str) -> String {
    let Some(first) = kind.file.members.first() else {
        return String::new();
    };
    let row = first.attributes.clone().unwrap_or_default();
    let fields: Vec<String> = kind
        .schema
        .iter()
        .map(|(field, ty)| {
            let value = row.get(field.as_str()).cloned().unwrap_or(Value::Null);
            format!(
                "        {}: {},",
                field_name(field),
                Catalogue::render_value(&value, ty, kind)
            )
        })
        .collect();
    let code = format!(
        "use teistro_core::catalogue::*;\n\nassert_eq!(\n    {name}::{}.attributes(),\n    &{name}Attributes {{\n{}\n    }}\n);",
        pascal(&first.key),
        fields.join("\n")
    );
    doc_example(indent, &code)
}

/// The example on one of a kind's value types: the first member holding
/// one, directly, as an option, or as an array's first element.
fn value_type_example(name: &str, kind: &Kind, type_name: &str) -> String {
    let wanted = Ty::Composite(type_name.to_owned());
    for member in &kind.file.members {
        let Some(row) = &member.attributes else {
            continue;
        };
        for (field, ty) in &kind.schema {
            let Some(value) = row.get(field.as_str()) else {
                continue;
            };
            let shown = match ty {
                Ty::Option(inner) if **inner == wanted && !value.is_null() => {
                    Some((String::new(), Catalogue::render_value(value, ty, kind)))
                }
                Ty::List(inner) | Ty::Array(inner, _) if **inner == wanted => value
                    .as_array()
                    .and_then(|items| items.first())
                    .map(|item| {
                        (
                            String::from("[0]"),
                            Catalogue::render_value(item, inner, kind),
                        )
                    }),
                direct if *direct == wanted => {
                    Some((String::new(), Catalogue::render_value(value, ty, kind)))
                }
                _ => None,
            };
            if let Some((index, rendered)) = shown {
                let code = format!(
                    "use teistro_core::catalogue::*;\n\nassert_eq!({name}::{}.attributes().{}{index}, {rendered});",
                    pascal(&member.key),
                    field_name(field)
                );
                return doc_example("", &code);
            }
        }
    }
    String::new()
}

/// The example on the `Kind` enum (`method` empty) or one of its methods:
/// the first catalogued kind, and the first open one where the method is
/// about openness.
fn kind_example(all: &[(String, u8, String, usize)], open: &[OpenKind], method: &str) -> String {
    let Some((kind, number, _, count)) = all.iter().find(|k| k.3 > 0) else {
        return String::new();
    };
    let name = pascal(kind);
    let open = open.first().map(|o| pascal(&o.name));
    let import = "use teistro_core::catalogue::Kind;\n\n";
    let code = match method {
        "" => format!(
            "{import}let kind: Kind = Kind::from_name(\"{kind}\").unwrap();\nassert_eq!(kind, Kind::{name});\nassert_eq!(kind.to_string(), \"{kind}\");"
        ),
        "name" => format!("{import}assert_eq!(Kind::{name}.name(), \"{kind}\");"),
        "number" => format!("{import}assert_eq!(Kind::{name}.number(), {number});"),
        "count" => {
            let mut code = format!("{import}assert_eq!(Kind::{name}.count(), {count});");
            if let Some(open) = &open {
                let _ = write!(code, "\nassert_eq!(Kind::{open}.count(), 0);");
            }
            code
        }
        "is_open" => {
            let mut code = format!("{import}assert!(!Kind::{name}.is_open());");
            if let Some(open) = &open {
                let _ = write!(code, "\nassert!(Kind::{open}.is_open());");
            }
            code
        }
        "from_number" => {
            let unused = (0..=u8::MAX).find(|n| all.iter().all(|k| k.1 != *n));
            let mut code =
                format!("{import}assert_eq!(Kind::from_number({number}), Some(Kind::{name}));");
            if let Some(unused) = unused {
                let _ = write!(code, "\nassert_eq!(Kind::from_number({unused}), None);");
            }
            code
        }
        "from_name" => format!(
            "{import}assert_eq!(Kind::from_name(\"{kind}\"), Some(Kind::{name}));\nassert_eq!(Kind::from_name(\"not_a_kind\"), None);"
        ),
        _ => return String::new(),
    };
    doc_example(if method.is_empty() { "" } else { "    " }, &code)
}

fn collect_keys(ty: &Ty, out: &mut BTreeSet<String>) {
    match ty {
        Ty::Key(k) => {
            out.insert(k.clone());
        }
        Ty::Option(inner) | Ty::List(inner) | Ty::Array(inner, _) => collect_keys(inner, out),
        _ => {}
    }
}

impl Kind {
    fn new(file: KindFile) -> Result<Kind, Vec<String>> {
        let mut errors = Vec::new();
        let name = file.kind.clone();
        if !is_kind_name(&name) {
            errors.push(format!("{name}: a kind name is lowercase snake case"));
        }
        let mut types = Vec::new();
        for (tname, fields) in &file.types {
            let tname = tname.clone();
            match fields
                .as_object()
                .ok_or_else(|| format!("{name}: type `{tname}` must be a mapping"))
                .and_then(|m| mapping_str(m, &format!("{name}.types.{tname}")))
            {
                Ok(pairs) => {
                    let mut parsed = Vec::new();
                    for (f, t) in pairs {
                        match parse_ty(&t) {
                            Ok(ty) => parsed.push((f, ty)),
                            Err(e) => errors.push(format!("{name}.types.{tname}.{f}: {e}")),
                        }
                    }
                    types.push((tname, parsed));
                }
                Err(e) => errors.push(e),
            }
        }
        let mut schema = Vec::new();
        match mapping_str(&file.attributes, &format!("{name}.attributes")) {
            Ok(pairs) => {
                for (f, t) in pairs {
                    match parse_ty(&t) {
                        Ok(ty) => schema.push((f, ty)),
                        Err(e) => errors.push(format!("{name}.attributes.{f}: {e}")),
                    }
                }
            }
            Err(e) => errors.push(e),
        }
        let mut members = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for (index, m) in file.members.iter().enumerate() {
            if !is_key_name(&m.key) {
                errors.push(format!("{name}.{}: not a key name", m.key));
            }
            if !members.insert(m.key.clone()) {
                errors.push(format!("{name}.{}: duplicate key", m.key));
            }
            if usize::from(m.id) != index || !ids.insert(m.id) {
                errors.push(format!(
                    "{name}.{}: id {} is not dense in file order",
                    m.key, m.id
                ));
            }
            if !matches!(m.mark.as_str(), "V" | "T" | "S") {
                errors.push(format!("{name}.{}: mark must be V, T or S", m.key));
            }
            if m.sources.is_empty() {
                errors.push(format!("{name}.{}: at least one source", m.key));
            }
            for alias in &m.aliases {
                if !is_key_name(alias) {
                    errors.push(format!(
                        "{name}.{}: alias `{alias}` is not a key name",
                        m.key
                    ));
                }
            }
        }
        for m in &file.members {
            for alias in &m.aliases {
                if members.contains(alias) {
                    errors.push(format!(
                        "{name}.{}: alias `{alias}` is also a member",
                        m.key
                    ));
                }
            }
        }
        if errors.is_empty() {
            Ok(Kind {
                file,
                schema,
                types,
                members,
            })
        } else {
            Err(errors)
        }
    }
}

/// `cargo xtask gen catalogue`.
pub(crate) fn generate(root: &Path) -> i32 {
    let catalogue = match Catalogue::load(root) {
        Ok(c) => c,
        Err(errors) => return report(&errors),
    };
    for (path, content) in catalogue.render() {
        let full = root.join(&path);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&full, content) {
            eprintln!("{}: {e}", full.display());
            return 1;
        }
        println!("written {}", path.display());
    }
    0
}

/// `cargo xtask check-catalogue`.
pub(crate) fn check(root: &Path) -> i32 {
    let catalogue = match Catalogue::load(root) {
        Ok(c) => c,
        Err(errors) => return report(&errors),
    };
    let mut failures = Vec::new();
    let rendered = catalogue.render();
    for (path, content) in &rendered {
        match std::fs::read_to_string(root.join(path)) {
            Ok(on_disk) if on_disk == *content => {}
            Ok(_) => failures.push(format!(
                "{}: differs from its sources; run `cargo xtask gen catalogue`",
                path.display()
            )),
            Err(_) => failures.push(format!(
                "{}: missing; run `cargo xtask gen catalogue`",
                path.display()
            )),
        }
    }
    if failures.is_empty() {
        println!("checked {} generated files: 0 failure(s)", rendered.len());
        0
    } else {
        report(&failures)
    }
}

fn report(errors: &[String]) -> i32 {
    for e in errors {
        eprintln!("{e}");
    }
    eprintln!("{} failure(s)", errors.len());
    1
}
