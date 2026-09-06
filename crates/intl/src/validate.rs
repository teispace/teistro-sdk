//! The gates a source tree passes before it is built: every message
//! parses, the base locale defines keys and parameters and every other
//! locale agrees with it, selectors use categories and context values
//! the locale has, references resolve, entities are well formed, and
//! coverage is known. One report, machine-readable and as Markdown.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use serde::Serialize;
use teistro_core::catalogue::Kind;

use crate::analysis::{ParamType, Signature, signature};
use crate::mf2::parse;
use crate::render::{Intl, digits};
use crate::source::{BASE_LOCALE, Completeness, ENTITY_NAMESPACE, Entry, LocaleSource, Tree};

/// How bad a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The tree must not be built.
    Error,
    /// Worth a look; the tree builds.
    Warning,
}

/// One finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Error or warning.
    pub severity: Severity,
    /// The locale.
    pub locale: String,
    /// The full key, empty for a locale-level finding.
    pub key: String,
    /// What is wrong.
    pub message: String,
}

/// Coverage of the base keys by one locale.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Coverage {
    /// Base keys present.
    pub present: usize,
    /// Base keys.
    pub total: usize,
    /// Base keys absent, sorted.
    pub missing: Vec<String>,
}

/// How many of a catalogue kind's members the base locale's entity
/// namespace describes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct KindCoverage {
    /// Members with a record.
    pub present: usize,
    /// Members in the catalogue.
    pub total: usize,
}

/// The validation report.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Findings, errors first, then by locale and key.
    pub diagnostics: Vec<Diagnostic>,
    /// Coverage per locale.
    pub coverage: BTreeMap<String, Coverage>,
    /// The catalogue's closed kinds and how many of their members the base
    /// locale describes, by kind name; reported, not gated, until the
    /// migration of the name tables fills them.
    pub catalogue: BTreeMap<String, KindCoverage>,
    /// Messages in the base locale.
    pub messages: usize,
    /// Entities in the base locale.
    pub entities: usize,
}

impl Report {
    /// Whether the tree may be built: no errors.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.errors() == 0
    }

    /// The number of errors.
    #[must_use]
    pub fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// The report as Markdown.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        let warnings = self.diagnostics.len() - self.errors();
        let _ = writeln!(
            out,
            "validated {} locales, {} messages, {} entities: {} errors, {} warnings\n",
            self.coverage.len(),
            self.messages,
            self.entities,
            self.errors(),
            warnings
        );
        out.push_str("| locale | coverage | missing |\n|---|---:|---|\n");
        for (locale, coverage) in &self.coverage {
            let _ = writeln!(
                out,
                "| {locale} | {}/{} | {} |",
                coverage.present,
                coverage.total,
                if coverage.missing.is_empty() {
                    String::from("none")
                } else {
                    coverage.missing.join(", ")
                }
            );
        }
        if !self.catalogue.is_empty() {
            out.push_str("\n| catalogue kind | described |\n|---|---:|\n");
            for (kind, coverage) in &self.catalogue {
                let _ = writeln!(out, "| {kind} | {}/{} |", coverage.present, coverage.total);
            }
        }
        if !self.diagnostics.is_empty() {
            out.push_str("\n| severity | locale | key | finding |\n|---|---|---|---|\n");
            for d in &self.diagnostics {
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} |",
                    match d.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                    },
                    d.locale,
                    d.key,
                    d.message
                );
            }
        }
        out
    }
}

struct Findings(Vec<Diagnostic>);

impl Findings {
    fn error(&mut self, locale: &str, key: &str, message: impl Into<String>) {
        self.0.push(Diagnostic {
            severity: Severity::Error,
            locale: locale.to_string(),
            key: key.to_string(),
            message: message.into(),
        });
    }

    fn warning(&mut self, locale: &str, key: &str, message: impl Into<String>) {
        self.0.push(Diagnostic {
            severity: Severity::Warning,
            locale: locale.to_string(),
            key: key.to_string(),
            message: message.into(),
        });
    }
}

/// A parsed message with its signature.
struct Analysed {
    signature: Signature,
}

/// What every locale is checked against.
struct Context<'a> {
    tree: &'a Tree,
    base: &'a LocaleSource,
    intl: Option<Intl>,
    analysed: BTreeMap<(String, String), Analysed>,
    base_keys: BTreeSet<String>,
    entity_keys: BTreeSet<&'a str>,
    kinds: BTreeSet<&'a str>,
}

/// Validates a tree against its base locale.
#[must_use]
pub fn validate(tree: &Tree) -> Report {
    let mut findings = Findings(Vec::new());
    let mut report = Report::default();
    let Some(base) = tree.base() else {
        findings.error(BASE_LOCALE, "", "the base locale is missing");
        report.diagnostics = findings.0;
        return report;
    };
    let intl = match Intl::from_tree(tree) {
        Ok(intl) => Some(intl),
        Err(error) => {
            findings.error("", "", format!("plural rules: {error}"));
            None
        }
    };
    let mut analysed: BTreeMap<(String, String), Analysed> = BTreeMap::new();
    for locale in tree.locales.values() {
        for (namespace, ns) in &locale.namespaces {
            for (key, entry) in &ns.entries {
                let full = format!("{namespace}.{key}");
                let Entry::Message(source) = entry else {
                    continue;
                };
                match parse(source) {
                    Ok(message) => {
                        let signature = signature(&message, &locale.meta);
                        analysed.insert((locale.tag.clone(), full), Analysed { signature });
                    }
                    Err(error) => findings.error(&locale.tag, &full, error.to_string()),
                }
            }
        }
    }
    let entity_keys: BTreeSet<&str> = base
        .namespaces
        .get(ENTITY_NAMESPACE)
        .map(|ns| ns.entries.keys().map(String::as_str).collect())
        .unwrap_or_default();
    let context = Context {
        tree,
        base,
        intl,
        analysed,
        base_keys: base.keys().collect(),
        kinds: entity_keys
            .iter()
            .filter_map(|k| k.split('.').next())
            .collect(),
        entity_keys,
    };
    report.entities = context.entity_keys.len();
    report.messages = context.base_keys.len() - context.entity_keys.len();
    report.catalogue = catalogue_coverage(&context.entity_keys);
    for locale in tree.locales.values() {
        let coverage = check_locale(&mut findings, &context, locale);
        report.coverage.insert(locale.tag.clone(), coverage);
    }
    findings.0.sort_by(|a, b| {
        (a.severity, &a.locale, &a.key, &a.message)
            .cmp(&(b.severity, &b.locale, &b.key, &b.message))
    });
    report.diagnostics = findings.0;
    report
}

/// The closed catalogue kinds with members, and how many the base locale's
/// entity namespace describes.
fn catalogue_coverage(entity_keys: &BTreeSet<&str>) -> BTreeMap<String, KindCoverage> {
    Kind::ALL
        .iter()
        .filter(|kind| !kind.is_open() && kind.count() > 0)
        .map(|kind| {
            let prefix = format!("{}.", kind.name());
            let present = entity_keys
                .iter()
                .filter(|key| key.starts_with(&prefix))
                .count();
            (
                kind.name().to_string(),
                KindCoverage {
                    present,
                    total: kind.count(),
                },
            )
        })
        .collect()
}

fn check_locale(findings: &mut Findings, context: &Context<'_>, locale: &LocaleSource) -> Coverage {
    check_meta(findings, context.tree, locale);
    let keys: BTreeSet<String> = locale.keys().collect();
    for key in keys.difference(&context.base_keys) {
        findings.error(&locale.tag, key, "not a key of the base locale");
    }
    let missing: Vec<String> = context.base_keys.difference(&keys).cloned().collect();
    if locale.meta.completeness == Completeness::Strict {
        for key in &missing {
            findings.error(
                &locale.tag,
                key,
                "missing; the locale declares strict completeness",
            );
        }
    }
    for key in &keys {
        match locale.entry(key) {
            Some(Entry::Entity(entity)) => {
                check_entity(findings, context.base, locale, key, entity);
            }
            Some(Entry::Message(_)) => {
                let Some(this) = context.analysed.get(&(locale.tag.clone(), key.clone())) else {
                    continue;
                };
                check_references(findings, &locale.tag, key, &this.signature, context);
                check_selectors(
                    findings,
                    context.intl.as_ref(),
                    locale,
                    key,
                    &this.signature,
                    &context.entity_keys,
                );
                if locale.tag != context.base.tag {
                    if let Some(reference) = context
                        .analysed
                        .get(&(context.base.tag.clone(), key.clone()))
                    {
                        check_parity(
                            findings,
                            &locale.tag,
                            key,
                            &this.signature,
                            &reference.signature,
                        );
                    }
                }
            }
            None => {}
        }
    }
    Coverage {
        present: context.base_keys.len() - missing.len(),
        total: context.base_keys.len(),
        missing,
    }
}

fn check_meta(findings: &mut Findings, tree: &Tree, locale: &LocaleSource) {
    let tag = &locale.tag;
    let meta = &locale.meta;
    if digits(&meta.numbering_system).is_none() {
        findings.error(
            tag,
            "",
            format!("unknown numbering system `{}`", meta.numbering_system),
        );
    }
    if meta.grouping.contains(&0) {
        findings.error(tag, "", "a digit group size of 0");
    }
    for (context, values) in &meta.contexts {
        let distinct: BTreeSet<&String> = values.iter().collect();
        if values.is_empty() || distinct.len() != values.len() {
            findings.error(
                tag,
                "",
                format!("context `{context}` must list distinct values"),
            );
        }
    }
    for fallback in &meta.fallback {
        if !tree.locales.contains_key(fallback) {
            findings.error(
                tag,
                "",
                format!("fallback `{fallback}` is not a loaded locale"),
            );
        }
        if fallback == tag {
            findings.error(tag, "", "a locale cannot fall back to itself");
        }
    }
    if tag == BASE_LOCALE && !meta.fallback.is_empty() {
        findings.warning(tag, "", "the base locale needs no fallback chain");
    }
    if tag != BASE_LOCALE && !meta.fallback.iter().any(|f| f == BASE_LOCALE) {
        findings.warning(
            tag,
            "",
            "the fallback chain does not end in the base locale",
        );
    }
    for (kind, pattern) in &meta.list_patterns {
        for (name, template) in [
            ("pair", &pattern.pair),
            ("middle", &pattern.middle),
            ("end", &pattern.end),
        ] {
            if !template.contains("{0}") || !template.contains("{1}") {
                findings.error(
                    tag,
                    "",
                    format!("list pattern `{kind}.{name}` needs `{{0}}` and `{{1}}`"),
                );
            }
        }
    }
}

fn check_entity(
    findings: &mut Findings,
    base: &LocaleSource,
    locale: &LocaleSource,
    key: &str,
    entity: &crate::source::Entity,
) {
    let tag = &locale.tag;
    if let Some(gender) = &entity.gender {
        let allowed = locale.meta.contexts.get("gender");
        if !allowed.is_some_and(|values| values.contains(gender)) {
            findings.error(
                tag,
                key,
                format!("gender `{gender}` is not a value of the `gender` context"),
            );
        }
    }
    if entity.name().is_empty() {
        findings.error(tag, key, "an entity needs a non-empty `name`");
    }
    // The full key is the namespace and the catalogue key; the catalogue
    // knows the latter.
    let catalogue_key = key
        .strip_prefix(ENTITY_NAMESPACE)
        .and_then(|rest| rest.strip_prefix('.'))
        .unwrap_or(key);
    if let Err(unknown) = teistro_core::key::resolve(catalogue_key) {
        findings.error(tag, key, format!("not a catalogue key: {unknown}"));
    }
    if *tag != base.tag {
        if let Some(Entry::Entity(reference)) = base.entry(key) {
            for form in reference.forms.keys() {
                if !entity.forms.contains_key(form) {
                    findings.warning(tag, key, format!("no `{form}` form; `name` will stand in"));
                }
            }
            if reference.gender != entity.gender {
                findings.warning(tag, key, "gender differs from the base locale");
            }
        }
    }
}

fn check_references(
    findings: &mut Findings,
    tag: &str,
    key: &str,
    sig: &Signature,
    context: &Context<'_>,
) {
    let (base_keys, entity_keys, kinds) =
        (&context.base_keys, &context.entity_keys, &context.kinds);
    for link in &sig.links {
        if !base_keys.contains(link) {
            findings.error(
                tag,
                key,
                format!("`:msg` names `{link}`, which the base locale does not have"),
            );
        }
    }
    for entity in &sig.entities {
        if let Err(unknown) = teistro_core::key::resolve(entity) {
            findings.error(
                tag,
                key,
                format!("`:entity` names `{entity}`, which is not a catalogue key: {unknown}"),
            );
        } else if !entity_keys.contains(entity.as_str()) {
            findings.warning(
                tag,
                key,
                format!(
                    "`:entity` names `{entity}`, which the entity namespace does not describe yet"
                ),
            );
        }
    }
    for kind in sig.params.values().filter_map(|p| match p {
        ParamType::Entity(Some(kind)) => Some(kind),
        _ => None,
    }) {
        if Kind::from_name(kind).is_none() {
            findings.error(tag, key, format!("`kind={kind}` is not a catalogue kind"));
        } else if !kinds.contains(kind.as_str()) {
            findings.warning(
                tag,
                key,
                format!("`kind={kind}`: the entity namespace describes no `{kind}` yet"),
            );
        }
    }
}

fn check_selectors(
    findings: &mut Findings,
    intl: Option<&Intl>,
    locale: &LocaleSource,
    key: &str,
    sig: &Signature,
    entity_keys: &BTreeSet<&str>,
) {
    let tag = &locale.tag;
    for selector in &sig.selectors {
        match &selector.kind {
            ParamType::Integer | ParamType::Number => {
                let Some(intl) = intl else {
                    continue;
                };
                let categories = intl.categories(tag, selector.ordinal);
                for k in &selector.keys {
                    if k.parse::<f64>().is_ok() {
                        continue;
                    }
                    if !categories.contains(&k.as_str()) {
                        findings.error(
                            tag,
                            key,
                            format!(
                                "`{k}` is never a {} category in {tag} (it has {})",
                                if selector.ordinal {
                                    "ordinal"
                                } else {
                                    "cardinal"
                                },
                                categories.join(", ")
                            ),
                        );
                    }
                }
            }
            ParamType::Context(context) => {
                let values = locale
                    .meta
                    .contexts
                    .get(context)
                    .cloned()
                    .unwrap_or_default();
                for k in &selector.keys {
                    if !values.contains(k) {
                        findings.error(
                            tag,
                            key,
                            format!("`{k}` is not a value of the `{context}` context"),
                        );
                    }
                }
            }
            ParamType::Entity(kind) => {
                let genders = locale
                    .meta
                    .contexts
                    .get("gender")
                    .cloned()
                    .unwrap_or_default();
                for k in &selector.keys {
                    let is_entity = entity_keys.contains(k.as_str())
                        || entity_keys.iter().any(|e| {
                            kind.as_ref()
                                .is_none_or(|kind| e.starts_with(&format!("{kind}.")))
                                && e.split_once('.').is_some_and(|(_, bare)| bare == k)
                        });
                    if !is_entity && !genders.contains(k) {
                        findings.error(
                            tag,
                            key,
                            format!("`{k}` is neither an entity key nor a gender"),
                        );
                    }
                }
            }
            ParamType::String | ParamType::List => {}
        }
    }
}

fn check_parity(findings: &mut Findings, tag: &str, key: &str, this: &Signature, base: &Signature) {
    for (name, kind) in &this.params {
        match base.params.get(name) {
            None => findings.error(
                tag,
                key,
                format!("uses `${name}`, which the base message does not declare"),
            ),
            Some(reference) if !kind.agrees_with(reference) => findings.error(
                tag,
                key,
                format!("uses `${name}` as {kind:?} where the base message has {reference:?}"),
            ),
            Some(_) => {}
        }
    }
    for name in base.params.keys() {
        if !this.params.contains_key(name) {
            findings.warning(tag, key, format!("does not use `${name}`"));
        }
    }
    for markup in &this.markup {
        if !base.markup.contains(markup) {
            findings.warning(
                tag,
                key,
                format!("uses markup `{markup}` the base message does not"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use super::*;
    use crate::source::{Entity, Namespace, sdk_root};

    fn tree() -> Tree {
        Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"))
    }

    fn namespace<'t>(tree: &'t mut Tree, locale: &str, namespace: &str) -> &'t mut Namespace {
        tree.locales
            .get_mut(locale)
            .and_then(|l| l.namespaces.get_mut(namespace))
            .unwrap_or_else(|| panic!("{locale} {namespace}"))
    }

    fn set(tree: &mut Tree, locale: &str, namespace: &str, key: &str, source: &str) {
        self::namespace(tree, locale, namespace)
            .insert(key.to_string(), Entry::Message(source.to_string()));
    }

    fn errors(report: &Report) -> Vec<String> {
        report
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| format!("{} {}: {}", d.locale, d.key, d.message))
            .collect()
    }

    #[test]
    fn the_sdk_sources_pass() {
        let report = validate(&tree());
        assert!(report.passed(), "{}", report.markdown());
        assert_eq!(report.coverage.len(), 2);
        assert!(report.coverage.values().all(|c| c.missing.is_empty()));
        assert!(report.messages > 10 && report.entities == 49, "{report:?}");
        // The catalogue's coverage is reported per closed kind: the signs
        // and the nakshatras complete, nine grahas so far (the outer
        // planets' records come with the migration of the name tables),
        // most kinds untouched.
        let kind = |name: &str| {
            report
                .catalogue
                .get(name)
                .unwrap_or_else(|| panic!("no coverage row for {name}"))
        };
        assert!(kind("graha").present >= 9 && kind("graha").total == 12);
        assert_eq!(kind("rashi").present, 12);
        assert_eq!(kind("nakshatra").present, 27);
        assert_eq!(kind("point").present, 1);
        assert!(kind("tithi").present == 0 && kind("tithi").total == 30);
        assert!(
            !report.catalogue.contains_key("rule"),
            "an open kind has no coverage row"
        );
    }

    /// A Nepali Sun with a gender outside the context, and an English
    /// entity the catalogue does not know.
    fn sabotage_entities(t: &mut Tree) {
        if let Some(ne) = t.locales.get_mut("ne-Deva-NP") {
            ne.meta.numbering_system = String::from("mars");
            ne.namespaces
                .entry(ENTITY_NAMESPACE.into())
                .or_insert_with(Namespace::default)
                .entries
                .insert(
                    "graha.SUN".into(),
                    Entry::Entity(Entity {
                        forms: [("name".to_string(), "सूर्य".to_string())]
                            .into_iter()
                            .collect(),
                        gender: Some("x".into()),
                        glyph: None,
                    }),
                );
        }
        if let Some(en) = t.locales.get_mut("en-Latn") {
            en.namespaces
                .entry(ENTITY_NAMESPACE.into())
                .or_insert_with(Namespace::default)
                .entries
                .insert(
                    "graha.VULCAN".into(),
                    Entry::Entity(Entity {
                        forms: [("name".to_string(), "Vulcan".to_string())]
                            .into_iter()
                            .collect(),
                        gender: None,
                        glyph: None,
                    }),
                );
        }
    }

    #[test]
    fn each_gate_fires() {
        let mut t = tree();
        set(&mut t, "ne-Deva-NP", "sdk.reason", "extra", "x");
        set(&mut t, "ne-Deva-NP", "sdk.reason", "appName", "{$who}");
        set(
            &mut t,
            "ne-Deva-NP",
            "sdk.reason",
            "welcome",
            ".input {$n :integer} .match $n two {{a}} * {{b}}",
        );
        set(
            &mut t,
            "ne-Deva-NP",
            "sdk.reason",
            "greeting",
            ".input {$gender :string} .match $gender x {{a}} * {{b}}",
        );
        set(&mut t, "en-Latn", "sdk.reason", "broken", "a } b");
        set(
            &mut t,
            "en-Latn",
            "sdk.reason",
            "dangling",
            "{sdk.reason.nowhere :msg} {graha.VULCAN :entity} {graha.PLUTO :entity} {$x :entity kind=asteroid}",
        );
        set(
            &mut t,
            "en-Latn",
            "sdk.reason",
            "rashiNature",
            ".input {$rashi :entity kind=rashi} .match $rashi SUN {{a}} * {{b}}",
        );
        namespace(&mut t, "ne-Deva-NP", "sdk.reason")
            .entries
            .remove("lordship");
        sabotage_entities(&mut t);
        let report = validate(&t);
        let found = errors(&report).join("\n");
        for expected in [
            "extra: not a key of the base locale",
            "appName: uses `$who`, which the base message does not declare",
            "welcome: `two` is never a cardinal category in ne-Deva-NP",
            "greeting: `x` is not a value of the `gender` context",
            "broken: `}` in text must be escaped",
            "dangling: `:msg` names `sdk.reason.nowhere`",
            "dangling: `:entity` names `graha.VULCAN`, which is not a catalogue key",
            "dangling: `kind=asteroid` is not a catalogue kind",
            "graha.VULCAN: not a catalogue key: unknown graha key `VULCAN`",
            "rashiNature: `SUN` is neither an entity key nor a gender",
            "lordship: missing; the locale declares strict completeness",
            "unknown numbering system `mars`",
            "graha.SUN: gender `x` is not a value",
        ] {
            assert!(
                found.contains(expected),
                "expected {expected:?} in:\n{found}"
            );
        }
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning && d.message.contains("no `prose` form"))
        );
        // A catalogue key the entity namespace does not describe yet is a
        // warning, not an error: the record can come later.
        assert!(report.diagnostics.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message
                    .contains("`graha.PLUTO`, which the entity namespace does not describe yet")
        }));
        assert!(!report.passed());
    }
}
