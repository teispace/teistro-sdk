//! Typed accessors from the base locale: a model of namespaces, groups,
//! messages with typed parameters, entity kinds and contexts, and one
//! emitter per target over it. The generated code holds keys and
//! parameter shapes only; text always comes from packs, so a message can
//! change or be overridden without regenerating.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::analysis::{ParamType, signature};
use crate::mf2::parse;
use crate::source::{ENTITY_NAMESPACE, Entry, LocaleSource};

/// A tree of accessors.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Group {
    /// Children by segment, sorted.
    pub children: BTreeMap<String, Node>,
}

/// One node of the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// A nested group.
    Group(Group),
    /// A message accessor.
    Message(MessageModel),
    /// An entity accessor, by catalogue key.
    Entity(String),
}

/// A message's accessor shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageModel {
    /// The full key.
    pub key: String,
    /// Parameters in name order.
    pub params: Vec<(String, ParamType)>,
    /// Whether the message uses markup.
    pub rich: bool,
}

/// What the generators emit from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    /// The base locale tag.
    pub locale: String,
    /// Contexts with their values.
    pub contexts: BTreeMap<String, Vec<String>>,
    /// Entity kinds with their bare keys, in source order.
    pub kinds: BTreeMap<String, Vec<String>>,
    /// The forms every entity of the base locale has, and the forms only
    /// some have.
    pub forms: (BTreeSet<String>, BTreeSet<String>),
    /// The accessor tree, rooted at the first namespace segments.
    pub root: Group,
    /// Every message key, sorted.
    pub keys: Vec<String>,
}

impl Model {
    /// The model of a locale, which must be the validated base locale.
    ///
    /// # Errors
    ///
    /// A message that does not parse, with its key.
    pub fn of(base: &LocaleSource) -> Result<Model, String> {
        let mut model = Model {
            locale: base.tag.clone(),
            contexts: base.meta.contexts.clone(),
            ..Model::default()
        };
        let mut common: Option<BTreeSet<String>> = None;
        let mut all: BTreeSet<String> = BTreeSet::new();
        for (namespace, ns) in &base.namespaces {
            for (key, entry) in ns.in_source_order() {
                let full = format!("{namespace}.{key}");
                let node = match entry {
                    Entry::Message(source) => {
                        let message = parse(source).map_err(|e| format!("{full}: {e}"))?;
                        let sig = signature(&message, &base.meta);
                        model.keys.push(full.clone());
                        Node::Message(MessageModel {
                            key: full.clone(),
                            params: sig.params.into_iter().collect(),
                            rich: !sig.markup.is_empty(),
                        })
                    }
                    Entry::Entity(entity) => {
                        if namespace == ENTITY_NAMESPACE {
                            if let Some((kind, bare)) = key.split_once('.') {
                                model
                                    .kinds
                                    .entry(kind.to_string())
                                    .or_default()
                                    .push(bare.to_string());
                            }
                        }
                        let forms: BTreeSet<String> = entity.forms.keys().cloned().collect();
                        all.extend(forms.iter().cloned());
                        common = Some(match common.take() {
                            Some(c) => c.intersection(&forms).cloned().collect(),
                            None => forms,
                        });
                        Node::Entity(key.clone())
                    }
                };
                insert(&mut model.root, &full, node);
            }
        }
        let common = common.unwrap_or_default();
        model.forms = (common.clone(), all.difference(&common).cloned().collect());
        Ok(model)
    }
}

fn insert(root: &mut Group, full_key: &str, node: Node) {
    let mut segments = full_key.split('.').peekable();
    let mut group = root;
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            group.children.insert(segment.to_string(), node);
            return;
        }
        let child = group
            .children
            .entry(segment.to_string())
            .or_insert_with(|| Node::Group(Group::default()));
        group = match child {
            Node::Group(g) => g,
            _ => return,
        };
    }
}

/// `PURVA_PHALGUNI` or `sdk` as `purvaPhalguni`, `sdk`.
#[must_use]
pub fn camel(segment: &str) -> String {
    if !segment.contains('_')
        && !segment
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        return segment.to_string();
    }
    let mut out = String::new();
    for (i, word) in segment.split('_').filter(|w| !w.is_empty()).enumerate() {
        let lower = word.to_ascii_lowercase();
        if i == 0 {
            out.push_str(&lower);
        } else {
            out.push_str(&capitalise(&lower));
        }
    }
    out
}

/// `strength` or `PURVA_PHALGUNI` as `Strength`, `PurvaPhalguni`.
#[must_use]
pub fn pascal(segment: &str) -> String {
    capitalise(&camel(segment))
}

fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

const DART_RESERVED: [&str; 33] = [
    "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "else",
    "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null",
    "rethrow", "return", "super", "switch", "this", "throw", "true", "try", "var", "void", "while",
    "with",
];

fn dart_name(segment: &str) -> String {
    let name = camel(segment);
    if DART_RESERVED.contains(&name.as_str()) {
        format!("{name}$")
    } else {
        name
    }
}

fn ts_type(kind: &ParamType) -> String {
    match kind {
        ParamType::String => String::from("string"),
        ParamType::Context(context) => pascal(context),
        ParamType::Integer | ParamType::Number => String::from("number"),
        ParamType::Entity(Some(kind)) => format!("{}Key", pascal(kind)),
        ParamType::Entity(None) => String::from("EntityKey"),
        ParamType::List => String::from("readonly (string | EntityKey)[]"),
    }
}

/// The TypeScript surface.
#[must_use]
pub fn typescript(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Generated by teistro-spike-intl from i18n/{}. Do not edit.\n// Keys and parameter shapes only; text comes from packs.\n",
        model.locale
    );
    for (context, values) in &model.contexts {
        let union: Vec<String> = values.iter().map(|v| format!("'{v}'")).collect();
        let _ = writeln!(
            out,
            "export type {} = {};",
            pascal(context),
            union.join(" | ")
        );
    }
    let mut kind_types = Vec::new();
    for (kind, keys) in &model.kinds {
        let name = format!("{}Key", pascal(kind));
        let union: Vec<String> = keys.iter().map(|k| format!("'{kind}.{k}'")).collect();
        let _ = writeln!(out, "export type {name} = {};", union.join(" | "));
        kind_types.push(name);
    }
    if !kind_types.is_empty() {
        let _ = writeln!(out, "export type EntityKey = {};", kind_types.join(" | "));
    }
    let keys: Vec<String> = model.keys.iter().map(|k| format!("'{k}'")).collect();
    let _ = writeln!(out, "export type MessageKey = {};", keys.join(" | "));
    out.push_str("\nexport interface EntityForms {\n");
    for form in &model.forms.0 {
        let _ = writeln!(out, "  readonly {form}: string;");
    }
    for form in &model.forms.1 {
        let _ = writeln!(out, "  readonly {form}?: string;");
    }
    out.push_str("  readonly glyph?: string;\n");
    if model.contexts.contains_key("gender") {
        out.push_str("  readonly gender?: Gender;\n");
    }
    out.push_str("}\n\n");
    out.push_str("export interface Renderer {\n  render(key: MessageKey, params?: Readonly<Record<string, unknown>>): string;\n  entity(key: EntityKey): EntityForms;\n}\n\n");
    out.push_str("export function messages(r: Renderer) {\n  return {\n");
    ts_group(&mut out, &model.root, 2);
    out.push_str("  } as const;\n}\n\nexport type Messages = ReturnType<typeof messages>;\n");
    out
}

fn ts_group(out: &mut String, group: &Group, depth: usize) {
    let pad = "  ".repeat(depth);
    for (segment, node) in &group.children {
        match node {
            Node::Group(child) => {
                let _ = writeln!(out, "{pad}{segment}: {{");
                ts_group(out, child, depth + 1);
                let _ = writeln!(out, "{pad}}},");
            }
            Node::Message(message) => {
                if message.params.is_empty() {
                    let _ = writeln!(out, "{pad}{segment}: () => r.render('{}'),", message.key);
                } else {
                    let fields: Vec<String> = message
                        .params
                        .iter()
                        .map(|(name, kind)| format!("{name}: {}", ts_type(kind)))
                        .collect();
                    let _ = writeln!(
                        out,
                        "{pad}{segment}: (p: {{ {} }}) => r.render('{}', p),",
                        fields.join("; "),
                        message.key
                    );
                }
            }
            Node::Entity(key) => {
                let _ = writeln!(out, "{pad}{segment}: () => r.entity('{key}'),");
            }
        }
    }
}

fn dart_type(kind: &ParamType) -> String {
    match kind {
        ParamType::String | ParamType::Entity(None) => String::from("String"),
        ParamType::Context(context) => pascal(context),
        ParamType::Integer => String::from("int"),
        ParamType::Number => String::from("num"),
        ParamType::Entity(Some(kind)) => format!("{}Key", pascal(kind)),
        ParamType::List => String::from("List<Object>"),
    }
}

fn dart_value(name: &str, kind: &ParamType) -> String {
    match kind {
        ParamType::Context(_) | ParamType::Entity(Some(_)) => format!("{name}.key"),
        _ => name.to_string(),
    }
}

/// The Dart surface.
#[must_use]
pub fn dart(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Generated by teistro-spike-intl from i18n/{}. Do not edit.\n// Keys and parameter shapes only; text comes from packs.\n",
        model.locale
    );
    for (context, values) in &model.contexts {
        let name = pascal(context);
        let members: Vec<String> = values
            .iter()
            .map(|v| format!("{}('{v}')", dart_name(v)))
            .collect();
        let _ = writeln!(
            out,
            "enum {name} {{\n  {};\n\n  const {name}(this.key);\n\n  final String key;\n}}\n",
            members.join(",\n  ")
        );
    }
    for (kind, keys) in &model.kinds {
        let name = format!("{}Key", pascal(kind));
        let members: Vec<String> = keys
            .iter()
            .map(|k| format!("{}('{kind}.{k}')", dart_name(k)))
            .collect();
        let _ = writeln!(
            out,
            "enum {name} {{\n  {};\n\n  const {name}(this.key);\n\n  final String key;\n}}\n",
            members.join(",\n  ")
        );
    }
    out.push_str("abstract interface class Renderer {\n  String render(String key, [Map<String, Object?> params = const {}]);\n  EntityForms entity(String key);\n}\n\n");
    out.push_str("final class EntityForms {\n  const EntityForms({\n");
    for form in &model.forms.0 {
        let _ = writeln!(out, "    required this.{form},");
    }
    for form in &model.forms.1 {
        let _ = writeln!(out, "    this.{form},");
    }
    out.push_str("    this.glyph,\n");
    if model.contexts.contains_key("gender") {
        out.push_str("    this.gender,\n");
    }
    out.push_str("  });\n\n");
    for form in &model.forms.0 {
        let _ = writeln!(out, "  final String {form};");
    }
    for form in &model.forms.1 {
        let _ = writeln!(out, "  final String? {form};");
    }
    out.push_str("  final String? glyph;\n");
    if model.contexts.contains_key("gender") {
        out.push_str("  final Gender? gender;\n");
    }
    out.push_str("}\n\n");
    let mut classes = Vec::new();
    dart_group(&mut classes, "Messages", &model.root);
    for class in classes {
        out.push_str(&class);
    }
    out
}

fn dart_group(classes: &mut Vec<String>, name: &str, group: &Group) {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "final class {name} {{\n  const {name}(this._r);\n\n  final Renderer _r;\n"
    );
    for (segment, node) in &group.children {
        let member = dart_name(segment);
        match node {
            Node::Group(child) => {
                let class = format!("{name}{}", pascal(segment));
                let _ = writeln!(out, "  {class} get {member} => {class}(_r);");
                dart_group(classes, &class, child);
            }
            Node::Message(message) => {
                if message.params.is_empty() {
                    let _ = writeln!(out, "  String {member}() => _r.render('{}');", message.key);
                } else {
                    let signature: Vec<String> = message
                        .params
                        .iter()
                        .map(|(n, k)| format!("required {} {n}", dart_type(k)))
                        .collect();
                    let map: Vec<String> = message
                        .params
                        .iter()
                        .map(|(n, k)| format!("'{n}': {}", dart_value(n, k)))
                        .collect();
                    let _ = writeln!(
                        out,
                        "  String {member}({{{}}}) =>\n      _r.render('{}', {{{}}});",
                        signature.join(", "),
                        message.key,
                        map.join(", ")
                    );
                }
            }
            Node::Entity(key) => {
                let _ = writeln!(out, "  EntityForms get {member} => _r.entity('{key}');");
            }
        }
    }
    out.push_str("}\n\n");
    classes.push(out);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use super::*;
    use crate::source::{Tree, spike_root};

    fn model() -> Model {
        let tree = Tree::load(&spike_root()).unwrap_or_else(|e| panic!("{e}"));
        Model::of(tree.base().unwrap_or_else(|| panic!("base"))).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn names_follow_each_target() {
        assert_eq!(camel("PURVA_PHALGUNI"), "purvaPhalguni");
        assert_eq!(camel("SUN"), "sun");
        assert_eq!(camel("grahaInBhava"), "grahaInBhava");
        assert_eq!(pascal("strength"), "Strength");
        assert_eq!(dart_name("in"), "in$");
    }

    #[test]
    fn the_surfaces_carry_every_key_and_no_text() {
        let model = model();
        let ts = typescript(&model);
        let dart = dart(&model);
        for key in &model.keys {
            assert!(ts.contains(&format!("'{key}'")), "ts lacks {key}");
            assert!(dart.contains(&format!("'{key}'")), "dart lacks {key}");
        }
        for text in ["Welcome to", "conjoins", "rules house", "Jupiter", "वृश्चिक"] {
            assert!(!ts.contains(text), "ts leaks {text:?}");
            assert!(!dart.contains(text), "dart leaks {text:?}");
        }
        assert!(ts.contains("grahaInBhava: (p: { bhava: number; graha: GrahaKey })"));
        assert!(ts.contains("greeting: (p: { gender: Gender; name: string })"));
        assert!(ts.contains("export type GrahaKey = 'graha.SUN'"));
        assert!(
            dart.contains("String grahaInBhava({required int bhava, required GrahaKey graha})")
        );
        assert!(dart.contains("enum NakshatraKey {\n  ashwini('nakshatra.ASHWINI')"));
        assert!(dart.contains("SdkReasonStrength get strength"));
        assert!(model.forms.0.contains("prose") && model.forms.0.contains("iast"));
        assert!(model.forms.1.is_empty(), "{:?}", model.forms.1);
    }
}
