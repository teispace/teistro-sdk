//! The `teistro-spike-intl` command line: `validate`, `build`, `gen`,
//! `render` and `report`, each a thin shell over a library function.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a tooling binary reports through its streams"
)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use teistro_spike_intl::generate::{self, Model};
use teistro_spike_intl::pack::{self, Pack, locales_from_packs};
use teistro_spike_intl::render::{Intl, Params, Value};
use teistro_spike_intl::source::Tree;
use teistro_spike_intl::validate::{self, Report};
use teistro_spike_intl::{bench, mf2};

/// Teistro Intl, spike 4.
#[derive(Parser)]
#[command(name = "teistro-spike-intl", version, about)]
struct Cli {
    /// The `i18n/` root.
    #[arg(long, global = true, default_value_os_t = default_root())]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Checks the sources: syntax, parity with the base locale, selectors,
    /// references, entities, coverage.
    Validate,
    /// Compiles packs, one per locale per namespace, and reports sizes.
    Build {
        /// Where to write the `.tpack` files.
        #[arg(long, default_value_os_t = default_out("packs"))]
        out: PathBuf,
        /// Only these locales.
        #[arg(long, value_delimiter = ',')]
        locales: Vec<String>,
        /// Only these namespaces.
        #[arg(long, value_delimiter = ',')]
        namespaces: Vec<String>,
    },
    /// Generates typed accessors from the base locale.
    Gen {
        /// The targets.
        #[arg(long, value_delimiter = ',', required = true)]
        target: Vec<Target>,
        /// Where to write them (`ts/sdk.ts`, `dart/lib/sdk.dart`).
        #[arg(long, default_value_os_t = default_out("harness"))]
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
    /// Validates, builds, generates and measures; writes `intl.json`.
    Report {
        /// Where to write the results.
        #[arg(long, default_value_os_t = default_out("results"))]
        out: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Target {
    Ts,
    Dart,
}

fn default_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../i18n")
}

fn default_out(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(dir)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(passed) => {
            if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<bool, Box<dyn std::error::Error>> {
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
        } => {
            let sizes = build_packs(&tree, &out, &locales, &namespaces)?;
            print!("{}", sizes_markdown(&sizes));
            Ok(true)
        }
        Command::Gen { target, out } => {
            let written = generate_targets(&tree, &target, &out)?;
            for path in written {
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

/// One built pack and its size against its source.
#[derive(Clone, Debug, Serialize)]
struct PackSize {
    locale: String,
    namespace: String,
    source_bytes: u64,
    pack_bytes: usize,
    entries: usize,
}

fn build_packs(
    tree: &Tree,
    out: &Path,
    locales: &[String],
    namespaces: &[String],
) -> Result<Vec<PackSize>, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out)?;
    let mut sizes = Vec::new();
    for locale in tree.locales.values() {
        if !locales.is_empty() && !locales.contains(&locale.tag) {
            continue;
        }
        for (name, namespace) in &locale.namespaces {
            if !namespaces.is_empty() && !namespaces.contains(name) {
                continue;
            }
            let bytes = pack::build(locale, name)?;
            let path = out.join(format!("{}.{name}.tpack", locale.tag));
            std::fs::write(&path, &bytes)?;
            let source = tree.root.join(&locale.tag).join(format!("{name}.json"));
            sizes.push(PackSize {
                locale: locale.tag.clone(),
                namespace: name.clone(),
                source_bytes: std::fs::metadata(&source).map_or(0, |m| m.len()),
                pack_bytes: bytes.len(),
                entries: namespace.entries.len(),
            });
        }
    }
    Ok(sizes)
}

fn sizes_markdown(sizes: &[PackSize]) -> String {
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

fn generate_targets(
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
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, text)?;
        written.push(path);
    }
    Ok(written)
}

fn parse_value(text: &str) -> Value {
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

fn parse_params(items: &[String]) -> Params {
    items
        .iter()
        .filter_map(|item| item.split_once('='))
        .map(|(name, value)| (name.to_string(), parse_value(value)))
        .collect()
}

/// Everything the result page quotes.
#[derive(Debug, Serialize)]
struct Results {
    validation: Report,
    sizes: Vec<PackSize>,
    generated: Vec<Generated>,
    bench: Vec<bench::Row>,
}

#[derive(Debug, Serialize)]
struct Generated {
    target: String,
    lines: usize,
    bytes: usize,
}

impl Results {
    fn markdown(&self) -> String {
        let mut out = self.validation.markdown();
        out.push('\n');
        out.push_str(&sizes_markdown(&self.sizes));
        out.push_str("\n| target | lines | bytes |\n|---|---:|---:|\n");
        for g in &self.generated {
            let _ = writeln!(out, "| {} | {} | {} |", g.target, g.lines, g.bytes);
        }
        out.push('\n');
        out.push_str(&bench::markdown(&self.bench));
        out
    }
}

fn report(tree: &Tree, out: &Path) -> Result<Results, Box<dyn std::error::Error>> {
    let validation = validate::validate(tree);
    let sizes = build_packs(tree, &out.join("packs"), &[], &[])?;
    let base = tree.base().ok_or("no base locale")?;
    let model = Model::of(base)?;
    let generated = [
        ("ts", generate::typescript(&model)),
        ("dart", generate::dart(&model)),
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
        bench: measure(tree)?,
    })
}

fn measure(tree: &Tree) -> Result<Vec<bench::Row>, Box<dyn std::error::Error>> {
    let ne = tree.locales.get("ne-Deva-NP").ok_or("ne-Deva-NP missing")?;
    let complex = match ne.entry("sdk.reason.grahaInBhava") {
        Some(teistro_spike_intl::source::Entry::Message(source)) => source.clone(),
        _ => return Err("sdk.reason.grahaInBhava missing".into()),
    };
    let mut intl = Intl::from_tree(tree)?;
    intl.set_locale("ne-Deva-NP")?;
    let ordinal = Params::from([
        ("graha".to_string(), Value::entity("graha.JUPITER")),
        ("bhava".to_string(), Value::Int(7)),
    ]);
    let angle = Params::from([
        ("graha".to_string(), Value::entity("graha.MARS")),
        ("longitude".to_string(), Value::Num(222.5763)),
    ]);
    let entity_pack = pack::build(ne, "sdk.entity")?;
    let mut all_packs: Vec<Vec<u8>> = Vec::new();
    for locale in tree.locales.values() {
        for name in locale.namespaces.keys() {
            all_packs.push(pack::build(locale, name)?);
        }
    }
    let borrowed: Vec<&[u8]> = all_packs.iter().map(Vec::as_slice).collect();
    let parsed = Pack::parse(&entity_pack)?;
    let (iterations, warmup, rounds) = (2000, 200, 3);
    Ok(vec![
        bench::bench(
            "parse: a matcher with two declarations and five variants",
            iterations,
            warmup,
            rounds,
            || {
                let _ = mf2::parse(&complex);
            },
        ),
        bench::bench(
            "render: sdk.reason.appName (a literal)",
            iterations,
            warmup,
            rounds,
            || {
                let _ = intl.render("sdk.reason.appName", &Params::new());
            },
        ),
        bench::bench(
            "render: sdk.reason.grahaInBhava (ordinal select, entity)",
            iterations,
            warmup,
            rounds,
            || {
                let _ = intl.render("sdk.reason.grahaInBhava", &ordinal);
            },
        ),
        bench::bench(
            "render: sdk.reason.grahaAt (:entity and :zodiac)",
            iterations,
            warmup,
            rounds,
            || {
                let _ = intl.render("sdk.reason.grahaAt", &angle);
            },
        ),
        bench::bench(
            "pack: build ne-Deva-NP sdk.entity (49 entities)",
            300,
            30,
            rounds,
            || {
                let _ = pack::build(ne, "sdk.entity");
            },
        ),
        bench::bench(
            "pack: parse and verify ne-Deva-NP sdk.entity",
            iterations,
            warmup,
            rounds,
            || {
                let _ = Pack::parse(&entity_pack);
            },
        ),
        bench::bench(
            "pack: look up graha.SUN (binary search, zero copy)",
            iterations,
            warmup,
            rounds,
            || {
                let _ = parsed.get("graha.SUN");
            },
        ),
        bench::bench("engine: build from four packs", 300, 30, rounds, || {
            let _ = locales_from_packs(&borrowed)
                .and_then(|l| Intl::new(l).map_err(|e| pack::PackError(e.0)));
        }),
    ])
}
