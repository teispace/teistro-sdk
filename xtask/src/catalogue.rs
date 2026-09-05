//! The catalogue generator and its gate: `catalogue/<kind>.yaml` in,
//! `crates/core/src/catalogue/generated/*.rs`, `catalogue/catalogue.json`
//! and `catalogue/entity-skeleton.json` out. `gen catalogue` writes them;
//! `check-catalogue` regenerates in memory and fails on any difference,
//! so the checked-in output can never drift from its sources.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_yaml_ng::{Mapping, Value};

/// Kinds that have no source file because their members arrive at runtime.
const OPEN_KINDS: [(&str, u8, &str); 1] = [(
    "rule",
    32,
    "Rule keys, which rule packs register at runtime; the catalogue holds the kind only.",
)];

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
}

fn mapping_str(m: &Mapping, what: &str) -> Result<Vec<(String, String)>, String> {
    m.iter()
        .map(|(k, v)| match (k.as_str(), v.as_str()) {
            (Some(k), Some(v)) => Ok((k.to_string(), v.to_string())),
            _ => Err(format!("{what}: entries must be `name: type`")),
        })
        .collect()
}

impl Catalogue {
    /// Loads `catalogue/*.yaml` and validates the whole.
    pub(crate) fn load(root: &Path) -> Result<Catalogue, Vec<String>> {
        let dir = root.join("catalogue");
        let mut errors = Vec::new();
        let mut kinds: Vec<Kind> = Vec::new();
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
            let file: KindFile = match serde_yaml_ng::from_str(&text) {
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
            match Kind::new(file) {
                Ok(kind) => kinds.push(kind),
                Err(mut e) => errors.append(&mut e),
            }
        }
        for (name, number, _) in OPEN_KINDS {
            if kinds
                .iter()
                .any(|k| k.file.kind == name || k.file.number == number)
            {
                errors.push(format!(
                    "open kind `{name}` ({number}) collides with a file"
                ));
            }
        }
        kinds.sort_by_key(|k| k.file.number);
        let catalogue = Catalogue { kinds };
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
                            let k = k.as_str().unwrap_or_default();
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
            Ty::List(inner) => match value.as_sequence() {
                Some(items) => items
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| self.check_value(v, inner, kind, &format!("{at}[{i}]")))
                    .collect(),
                None => bad("a list"),
            },
            Ty::Array(inner, n) => match value.as_sequence() {
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
                let Some(map) = value.as_mapping() else {
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
        s.push_str("/// Resolves a key inside a kind to its id, over every catalogued kind.\n#[must_use]\npub fn resolve(kind: Kind, key: &str) -> Option<KeyId> {\n    match kind {\n");
        for kind in &self.kinds {
            let _ = writeln!(
                s,
                "        Kind::{p} => {p}::from_key(key).map({p}::key_id),",
                p = pascal(&kind.file.kind)
            );
        }
        s.push_str("        _ => None,\n    }\n}\n\n");
        s.push_str("/// The key of a catalogued id, over every catalogued kind.\n#[must_use]\npub fn key_of(id: KeyId) -> Option<&'static str> {\n    match id.kind()? {\n");
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
        for (name, number, doc) in OPEN_KINDS {
            all.push((name.to_string(), number, doc.to_string(), 0));
        }
        all.sort_by_key(|k| k.1);
        let mut s = String::from(
            "//! Generated by `cargo xtask gen catalogue`; do not edit.\n\n#![allow(clippy::match_same_arms, reason = \"generated tables\")]\n\n",
        );
        s.push_str("/// A kind: a family of entities sharing one key type. The number is part of\n/// the C boundary and is never reused.\n#[repr(u8)]\n#[non_exhaustive]\n#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum Kind {\n");
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
        s.push_str("    ];\n\n    /// The kind's name, the first segment of its members' full keys.\n    #[must_use]\n    pub const fn name(self) -> &'static str {\n        match self {\n");
        for (name, _, _, _) in &all {
            let _ = writeln!(s, "            Kind::{} => \"{name}\",", pascal(name));
        }
        s.push_str("        }\n    }\n\n    /// The kind's number at the C boundary.\n    #[must_use]\n    pub const fn number(self) -> u8 {\n        self as u8\n    }\n\n    /// The number of catalogued members; zero for an open kind.\n    #[must_use]\n    pub const fn count(self) -> usize {\n        match self {\n");
        for (name, _, _, count) in &all {
            let _ = writeln!(s, "            Kind::{} => {count},", pascal(name));
        }
        s.push_str("        }\n    }\n\n    /// Whether members arrive at runtime rather than from the catalogue.\n    #[must_use]\n    pub const fn is_open(self) -> bool {\n        self.count() == 0\n    }\n\n    /// The kind with a number.\n    #[must_use]\n    pub const fn from_number(number: u8) -> Option<Kind> {\n        match number {\n");
        for (name, number, _, _) in &all {
            let _ = writeln!(s, "            {number} => Some(Kind::{}),", pascal(name));
        }
        s.push_str("            _ => None,\n        }\n    }\n\n    /// The kind with a name.\n    #[must_use]\n    pub fn from_name(name: &str) -> Option<Kind> {\n        crate::catalogue::lookup(&BY_NAME, name)\n    }\n}\n\nimpl core::fmt::Display for Kind {\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {\n        f.write_str(self.name())\n    }\n}\n");
        s.push_str("\nimpl serde::Serialize for Kind {\n    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {\n        serializer.serialize_str(self.name())\n    }\n}\n\nimpl<'de> serde::Deserialize<'de> for Kind {\n    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {\n        let name = <&str>::deserialize(deserializer)?;\n        Kind::from_name(name).ok_or_else(|| serde::de::Error::custom(crate::catalogue::UnknownKey::kind_name(name)))\n    }\n}\n");
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
            "\n/// {}\n///\n/// Members are appended only; the discriminants are the catalogue ids.\n#[repr(u16)]\n#[non_exhaustive]\n#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum {name} {{",
            file.doc
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
                "\n/// A value type of the `{}` attributes.\n#[derive(Clone, Copy, Debug, PartialEq)]\npub struct {tname} {{\n",
                file.kind
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
                "\n/// The attributes of a `{}` member.\n#[derive(Clone, Copy, Debug, PartialEq)]\npub struct {name}Attributes {{\n",
                file.kind
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
            |m| {
                format!(
                    "Mark::{}",
                    match m.mark.as_str() {
                        "V" => "Verified",
                        "T" => "Traditional",
                        _ => "Shape",
                    }
                )
            },
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
        let _ = write!(
            s,
            "    /// The catalogue id: the discriminant.\n    #[must_use]\n    pub const fn id(self) -> u16 {{\n        self as u16\n    }}\n\n    /// The packed key id for the C boundary.\n    #[must_use]\n    pub const fn key_id(self) -> KeyId {{\n        KeyId::new(Kind::{name}, self as u16)\n    }}\n\n    /// The member with a key or a former key.\n    #[must_use]\n    pub fn from_key(key: &str) -> Option<{name}> {{\n        crate::catalogue::lookup(&BY_KEY, key).or_else(|| crate::catalogue::lookup(&ALIASES, key))\n    }}\n\n    /// The member with an id.\n    #[must_use]\n    pub const fn from_id(id: u16) -> Option<{name}> {{\n        match id {{\n"
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
                "\n    /// The member's attributes.\n    #[must_use]\n    #[allow(clippy::indexing_slicing, reason = \"the discriminants are dense from zero\")]\n    pub fn attributes(self) -> &'static {name}Attributes {{\n        &ATTRIBUTES[self as usize]\n    }}\n"
            );
        }
        s.push_str("}\n");
        let _ = write!(
            s,
            "\nimpl Catalogued for {name} {{\n    const KIND: Kind = Kind::{name};\n\n    fn all() -> &'static [Self] {{\n        &Self::ALL\n    }}\n\n    fn key(self) -> &'static str {{\n        self.key()\n    }}\n\n    fn id(self) -> u16 {{\n        self.id()\n    }}\n\n    fn mark(self) -> Mark {{\n        self.mark()\n    }}\n\n    fn from_key(key: &str) -> Option<Self> {{\n        Self::from_key(key)\n    }}\n\n    fn from_id(id: u16) -> Option<Self> {{\n        Self::from_id(id)\n    }}\n}}\n\nimpl core::fmt::Display for {name} {{\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{\n        f.write_str(self.key())\n    }}\n}}\n\nimpl core::str::FromStr for {name} {{\n    type Err = UnknownKey;\n\n    fn from_str(key: &str) -> Result<Self, UnknownKey> {{\n        Self::from_key(key).ok_or_else(|| UnknownKey::in_kind::<{name}>(key))\n    }}\n}}\n\nimpl From<{name}> for KeyId {{\n    fn from(member: {name}) -> KeyId {{\n        member.key_id()\n    }}\n}}\n\nimpl TryFrom<KeyId> for {name} {{\n    type Error = UnknownKey;\n\n    fn try_from(id: KeyId) -> Result<Self, UnknownKey> {{\n        match id.kind() {{\n            Some(Kind::{name}) => Self::from_id(id.id()).ok_or_else(|| UnknownKey::id(id)),\n            _ => Err(UnknownKey::id(id)),\n        }}\n    }}\n}}\n\nimpl serde::Serialize for {name} {{\n    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {{\n        serializer.serialize_str(self.key())\n    }}\n}}\n\nimpl<'de> serde::Deserialize<'de> for {name} {{\n    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {{\n        let key = <&str>::deserialize(deserializer)?;\n        Self::from_key(key).ok_or_else(|| serde::de::Error::custom(UnknownKey::in_kind::<{name}>(key)))\n    }}\n}}\n"
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
        let _ = write!(
            s,
            "    /// {doc}\n    #[must_use]\n    pub const fn {fn_name}(self) -> {ret} {{\n        match self {{\n"
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
                    .as_sequence()
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
                let map = value.as_mapping();
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
                    "types": yaml_to_json(&Value::Mapping(k.file.types.clone())),
                    "attributes": yaml_to_json(&Value::Mapping(k.file.attributes.clone())),
                    "members": k.file.members.iter().map(|m| serde_json::json!({
                        "key": m.key, "id": m.id, "doc": m.doc, "glyph": m.glyph, "aliases": m.aliases,
                        "deprecated": m.deprecated,
                        "attributes": m.attributes.as_ref().map(|a| yaml_to_json(&Value::Mapping(a.clone()))),
                        "sources": m.sources.iter().map(|s| serde_json::json!({"text": s.text, "ref": s.reference})).collect::<Vec<_>>(),
                        "mark": m.mark, "unverified": m.unverified,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let open: Vec<serde_json::Value> = OPEN_KINDS
            .iter()
            .map(
                |(n, num, d)| serde_json::json!({"kind": n, "number": num, "doc": d, "open": true}),
            )
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
                        .and_then(|d| d.as_mapping())
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

fn collect_keys(ty: &Ty, out: &mut BTreeSet<String>) {
    match ty {
        Ty::Key(k) => {
            out.insert(k.clone());
        }
        Ty::Option(inner) | Ty::List(inner) | Ty::Array(inner, _) => collect_keys(inner, out),
        _ => {}
    }
}

fn yaml_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => n
            .as_i64()
            .map_or_else(|| serde_json::json!(n.as_f64()), |i| serde_json::json!(i)),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Sequence(seq) => serde_json::Value::Array(seq.iter().map(yaml_to_json).collect()),
        Value::Mapping(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), yaml_to_json(v)))
                .collect(),
        ),
        Value::Tagged(t) => yaml_to_json(&t.value),
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
            let tname = tname.as_str().unwrap_or_default().to_string();
            match fields
                .as_mapping()
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
