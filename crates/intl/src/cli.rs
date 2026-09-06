//! The `teistro-intl` command line as library functions: `validate`,
//! `build`, `gen`, `render`, `extract` and `report`, each a function over a
//! loaded [`Tree`], and [`main`] the shell that parses arguments and
//! prints. The root defaults to the SDK's own `i18n/`.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the command line reports through its streams"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::{Map, Value as Json};

use crate::generate::{self, Model, RustPaths};
use crate::pack;
use crate::render::{Intl, Params, Value};
use crate::source::{BASE_LOCALE, Completeness, Entry, LocaleSource, META_FILE, Tree, sdk_root};
use crate::validate::{self, Report};

/// The command line.
#[derive(Debug, Parser)]
#[command(
    name = "teistro-intl",
    version,
    about = "Teistro Intl: validate, build, generate, render, extract and report over an i18n/ tree"
)]
pub struct Cli {
    /// The `i18n/` root; the SDK's own by default.
    #[arg(long, global = true, default_value_os_t = sdk_root())]
    pub root: PathBuf,
    /// The command.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Checks the sources: syntax, parity with the base locale, selectors,
    /// references, the catalogue, coverage.
    Validate,
    /// Compiles packs, one per locale per namespace, or one bundle per
    /// locale, and reports sizes.
    Build {
        /// Where to write the files.
        #[arg(long, default_value = "target/packs")]
        out: PathBuf,
        /// Only these locales.
        #[arg(long, value_delimiter = ',')]
        locales: Vec<String>,
        /// Only these namespaces (packs only).
        #[arg(long, value_delimiter = ',')]
        namespaces: Vec<String>,
        /// One bundle per locale instead of a pack per namespace.
        #[arg(long)]
        bundle: bool,
    },
    /// Generates typed accessors from the base locale.
    Gen {
        /// The targets.
        #[arg(long, value_delimiter = ',', required = true)]
        target: Vec<Target>,
        /// Where to write them (`ts/sdk.ts`, `dart/lib/sdk.dart`,
        /// `rs/messages.rs`).
        #[arg(long, default_value = "target/intl-gen")]
        out: PathBuf,
    },
    /// Renders one key with parameters (`name=value`, `name=@graha.SUN`
    /// for an entity, `name=@a,@b` for a list).
    Render {
        /// The locale.
        #[arg(long)]
        locale: String,
        /// The full key.
        key: String,
        /// Parameters.
        #[arg(long = "param", value_name = "NAME=VALUE")]
        params: Vec<String>,
    },
    /// Scaffolds a locale from the base: every key with the base text
    /// beside it (an existing translation kept), and a `_meta.json`
    /// template.
    Extract {
        /// The locale tag, with its script (`ta-Taml-IN`).
        #[arg(long)]
        locale: String,
        /// Only these namespaces.
        #[arg(long, value_delimiter = ',')]
        namespaces: Vec<String>,
        /// Where to write the locale directory; the root by default.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Validates, builds every pack and bundle, generates every target and
    /// writes `intl.json`.
    Report {
        /// Where to write the results.
        #[arg(long, default_value = "target/intl-report")]
        out: PathBuf,
    },
}

/// A generator target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Target {
    /// TypeScript.
    Ts,
    /// Dart.
    Dart,
    /// Rust.
    Rs,
}

/// Parses the arguments, runs the command, prints, and exits non-zero on
/// a failed validation, a render with warnings or an error.
#[must_use]
pub fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs a parsed command line; `Ok(true)` when the command passed.
///
/// # Errors
///
/// A tree that does not load, a file that cannot be written, a locale
/// the engine cannot answer for.
pub fn run(cli: Cli) -> Result<bool, Box<dyn std::error::Error>> {
    let tree = Tree::load(&cli.root)?;
    match cli.command {
        Command::Validate => {
            let report = validate::validate(&tree);
            print!("{}", report.markdown());
            Ok(report.passed())
        }
        Command::Build {
            out,
            locales,
            namespaces,
            bundle,
        } => {
            let sizes = build_packs(&tree, &out, &locales, &namespaces, bundle)?;
            print!("{}", sizes_markdown(&sizes));
            Ok(true)
        }
        Command::Gen { target, out } => {
            for path in generate_targets(&tree, &target, &out)? {
                println!("written {}", path.display());
            }
            Ok(true)
        }
        Command::Render {
            locale,
            key,
            params,
        } => {
            let mut intl = Intl::from_tree(&tree)?;
            intl.set_locale(&locale)?;
            let rendered = intl.render(&key, &parse_params(&params));
            println!("{}", rendered.text);
            println!(
                "resolved from {}{}",
                rendered.resolved_from.as_deref().unwrap_or("nowhere"),
                if rendered.is_fallback {
                    " (fallback)"
                } else {
                    ""
                }
            );
            for warning in &rendered.warnings {
                println!("warning: {warning}");
            }
            Ok(rendered.warnings.is_empty())
        }
        Command::Extract {
            locale,
            namespaces,
            out,
        } => {
            let files = extract(&tree, &locale, &namespaces)?;
            let dir = out.unwrap_or_else(|| tree.root.clone()).join(&locale);
            for path in write_files(&dir, &files)? {
                println!("written {}", path.display());
            }
            Ok(true)
        }
        Command::Report { out } => {
            let results = report(&tree, &out)?;
            print!("{}", results.markdown());
            std::fs::create_dir_all(&out)?;
            let path = out.join("intl.json");
            std::fs::write(
                &path,
                format!("{}\n", serde_json::to_string_pretty(&results)?),
            )?;
            println!("written {}", path.display());
            Ok(results.validation.passed())
        }
    }
}

/// One built file and its size against its source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackSize {
    /// The locale.
    pub locale: String,
    /// The namespace, or `*` for a bundle.
    pub namespace: String,
    /// The source's bytes (every namespace's for a bundle).
    pub source_bytes: u64,
    /// The file's bytes.
    pub pack_bytes: usize,
    /// The entries.
    pub entries: usize,
}

/// Builds the packs (or bundles) of a tree into a directory and reports
/// their sizes.
///
/// # Errors
///
/// A directory or file that cannot be written, a namespace that cannot be
/// packed.
pub fn build_packs(
    tree: &Tree,
    out: &Path,
    locales: &[String],
    namespaces: &[String],
    bundle: bool,
) -> Result<Vec<PackSize>, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out)?;
    let mut sizes = Vec::new();
    for locale in tree.locales.values() {
        if !locales.is_empty() && !locales.contains(&locale.tag) {
            continue;
        }
        if bundle {
            let bytes = pack::build_bundle(locale)?;
            std::fs::write(out.join(format!("{}.tbundle", locale.tag)), &bytes)?;
            sizes.push(PackSize {
                locale: locale.tag.clone(),
                namespace: String::from("*"),
                source_bytes: locale
                    .namespaces
                    .keys()
                    .map(|name| source_bytes(tree, locale, name))
                    .sum(),
                pack_bytes: bytes.len(),
                entries: locale.namespaces.values().map(|ns| ns.entries.len()).sum(),
            });
            continue;
        }
        for (name, namespace) in &locale.namespaces {
            if !namespaces.is_empty() && !namespaces.contains(name) {
                continue;
            }
            let bytes = pack::build(locale, name)?;
            std::fs::write(out.join(format!("{}.{name}.tpack", locale.tag)), &bytes)?;
            sizes.push(PackSize {
                locale: locale.tag.clone(),
                namespace: name.clone(),
                source_bytes: source_bytes(tree, locale, name),
                pack_bytes: bytes.len(),
                entries: namespace.entries.len(),
            });
        }
    }
    Ok(sizes)
}

fn source_bytes(tree: &Tree, locale: &LocaleSource, namespace: &str) -> u64 {
    let source = tree
        .root
        .join(&locale.tag)
        .join(format!("{namespace}.json"));
    std::fs::metadata(&source).map_or(0, |m| m.len())
}

/// The sizes as a Markdown table.
#[must_use]
pub fn sizes_markdown(sizes: &[PackSize]) -> String {
    let mut out = String::from(
        "| locale | namespace | entries | source bytes | pack bytes |\n|---|---|---:|---:|---:|\n",
    );
    for size in sizes {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            size.locale, size.namespace, size.entries, size.source_bytes, size.pack_bytes
        );
    }
    out
}

/// The typed accessors of the base locale for each target, written under
/// `out` (`ts/sdk.ts`, `dart/lib/sdk.dart`, `rs/messages.rs`).
///
/// # Errors
///
/// No base locale, a message that does not parse, a file that cannot be
/// written.
pub fn generate_targets(
    tree: &Tree,
    targets: &[Target],
    out: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let base = tree.base().ok_or("no base locale")?;
    let model = Model::of(base)?;
    let mut written = Vec::new();
    for target in targets {
        let (path, text) = match target {
            Target::Ts => (out.join("ts/sdk.ts"), generate::typescript(&model)),
            Target::Dart => (out.join("dart/lib/sdk.dart"), generate::dart(&model)),
            Target::Rs => (
                out.join("rs/messages.rs"),
                generate::rust(&model, RustPaths::CONSUMER),
            ),
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, text)?;
        written.push(path);
    }
    Ok(written)
}

/// A parameter value from its command-line text: `@graha.SUN` an entity,
/// `@a,@b` a list of entities, an integer, a number, or text.
#[must_use]
pub fn parse_value(text: &str) -> Value {
    if text.contains(',') && text.starts_with('@') {
        return Value::List(text.split(',').map(parse_value).collect());
    }
    if let Some(key) = text.strip_prefix('@') {
        return Value::entity(key);
    }
    if let Ok(i) = text.parse::<i64>() {
        return Value::Int(i);
    }
    if let Ok(n) = text.parse::<f64>() {
        return Value::Num(n);
    }
    Value::Str(text.to_string())
}

/// Parameters from `name=value` items.
#[must_use]
pub fn parse_params(items: &[String]) -> Params {
    items
        .iter()
        .filter_map(|item| item.split_once('='))
        .map(|(name, value)| (name.to_string(), parse_value(value)))
        .collect()
}

/// The files of a scaffolded locale, by file name: `_meta.json` from the
/// base's metadata with the tag, a fallback to the base and `base`
/// completeness (an existing locale's metadata kept), and every base
/// namespace as nested JSON with the base text where the locale has no
/// translation yet.
///
/// # Errors
///
/// No base locale, or a namespace name the base does not have.
pub fn extract(
    tree: &Tree,
    locale: &str,
    namespaces: &[String],
) -> Result<BTreeMap<String, Json>, String> {
    let base = tree.base().ok_or("no base locale")?;
    let existing = tree.locales.get(locale);
    let mut files = BTreeMap::new();
    let meta = existing.map_or_else(
        || {
            let mut meta = base.meta.clone();
            meta.locale = locale.to_string();
            meta.fallback = vec![BASE_LOCALE.to_string()];
            meta.completeness = Completeness::Base;
            meta
        },
        |found| found.meta.clone(),
    );
    files.insert(
        META_FILE.to_string(),
        serde_json::to_value(&meta).map_err(|e| e.to_string())?,
    );
    for name in namespaces {
        if !base.namespaces.contains_key(name) {
            return Err(format!("the base locale has no namespace `{name}`"));
        }
    }
    for (name, namespace) in &base.namespaces {
        if !namespaces.is_empty() && !namespaces.contains(name) {
            continue;
        }
        let translated = existing.and_then(|l| l.namespaces.get(name));
        let mut root = Map::new();
        for (key, entry) in namespace.in_source_order() {
            let entry = translated
                .and_then(|ns| ns.entries.get(key))
                .unwrap_or(entry);
            insert_nested(&mut root, key, entry_json(entry));
        }
        files.insert(format!("{name}.json"), Json::Object(root));
    }
    Ok(files)
}

fn entry_json(entry: &Entry) -> Json {
    match entry {
        Entry::Message(source) => Json::String(source.clone()),
        Entry::Entity(entity) => {
            let mut object = Map::new();
            for (form, value) in &entity.forms {
                object.insert(form.clone(), Json::String(value.clone()));
            }
            if let Some(glyph) = &entity.glyph {
                object.insert(String::from("glyph"), Json::String(glyph.clone()));
            }
            if let Some(gender) = &entity.gender {
                object.insert(String::from("gender"), Json::String(gender.clone()));
            }
            Json::Object(object)
        }
    }
}

fn insert_nested(root: &mut Map<String, Json>, key: &str, value: Json) {
    let mut segments = key.split('.').peekable();
    let mut object = root;
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            object.insert(segment.to_string(), value);
            return;
        }
        let child = object
            .entry(segment.to_string())
            .or_insert_with(|| Json::Object(Map::new()));
        match child {
            Json::Object(map) => object = map,
            _ => return,
        }
    }
}

/// Writes scaffolded files into a locale directory.
///
/// # Errors
///
/// A directory or file that cannot be written.
pub fn write_files(dir: &Path, files: &BTreeMap<String, Json>) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    for (name, value) in files {
        let path = dir.join(name);
        std::fs::write(&path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
        written.push(path);
    }
    Ok(written)
}

/// Everything `report` measures.
#[derive(Debug, Serialize)]
pub struct Results {
    /// The validation report.
    pub validation: Report,
    /// Every pack's size, then every bundle's.
    pub sizes: Vec<PackSize>,
    /// The generated surfaces' sizes.
    pub generated: Vec<Generated>,
}

/// One generated surface's size.
#[derive(Debug, Serialize)]
pub struct Generated {
    /// The target.
    pub target: String,
    /// Lines.
    pub lines: usize,
    /// Bytes.
    pub bytes: usize,
}

impl Results {
    /// The results as Markdown.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = self.validation.markdown();
        out.push('\n');
        out.push_str(&sizes_markdown(&self.sizes));
        out.push_str("\n| target | lines | bytes |\n|---|---:|---:|\n");
        for g in &self.generated {
            let _ = writeln!(out, "| {} | {} | {} |", g.target, g.lines, g.bytes);
        }
        out
    }
}

/// Validates, builds every pack and every bundle under `out`, and sizes
/// every generated surface.
///
/// # Errors
///
/// A file that cannot be written, no base locale, a message that does not
/// parse.
pub fn report(tree: &Tree, out: &Path) -> Result<Results, Box<dyn std::error::Error>> {
    let validation = validate::validate(tree);
    let mut sizes = build_packs(tree, &out.join("packs"), &[], &[], false)?;
    sizes.extend(build_packs(tree, &out.join("bundles"), &[], &[], true)?);
    let base = tree.base().ok_or("no base locale")?;
    let model = Model::of(base)?;
    let generated = [
        ("ts", generate::typescript(&model)),
        ("dart", generate::dart(&model)),
        ("rs", generate::rust(&model, RustPaths::CONSUMER)),
    ]
    .into_iter()
    .map(|(target, text)| Generated {
        target: target.to_string(),
        lines: text.lines().count(),
        bytes: text.len(),
    })
    .collect();
    Ok(Results {
        validation,
        sizes,
        generated,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read small JSON trees"
    )]

    use super::*;

    #[test]
    fn values_and_parameters_parse_from_their_text() {
        assert_eq!(parse_value("@graha.SUN"), Value::entity("graha.SUN"));
        assert_eq!(parse_value("7"), Value::Int(7));
        assert_eq!(parse_value("222.5"), Value::Num(222.5));
        assert_eq!(parse_value("Nepal"), Value::Str(String::from("Nepal")));
        assert_eq!(
            parse_value("@graha.SUN,@graha.MOON"),
            Value::List(vec![
                Value::entity("graha.SUN"),
                Value::entity("graha.MOON")
            ])
        );
        let params = parse_params(&[String::from("bhava=7"), String::from("broken")]);
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn a_scaffolded_locale_carries_every_base_key_and_keeps_translations() {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        let fresh = extract(&tree, "ta-Taml-IN", &[]).unwrap();
        let meta = fresh.get(META_FILE).unwrap();
        assert_eq!(meta["locale"], "ta-Taml-IN");
        assert_eq!(meta["fallback"][0], BASE_LOCALE);
        assert_eq!(meta["completeness"], "base");
        assert_eq!(
            fresh["sdk.entity.json"]["graha"]["SUN"]["name"], "Sun",
            "the base text stands in"
        );
        let kept = extract(&tree, "ne-Deva-NP", &[String::from("sdk.entity")]).unwrap();
        assert_eq!(kept["sdk.entity.json"]["graha"]["SUN"]["name"], "सूर्य");
        assert!(!kept.contains_key("sdk.reason.json"));
        assert!(extract(&tree, "x-Latn", &[String::from("nope")]).is_err());
    }
}
