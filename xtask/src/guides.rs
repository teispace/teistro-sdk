//! `cargo xtask check-guides`: every program a guide shows is an excerpt
//! of a file a gate runs, and every output it shows is what that file
//! prints.
//!
//! The guides under `site/content/docs` are written by hand, and a
//! program written by hand into a page is one nothing runs: the first
//! guides copied theirs from the shared examples and had drifted from them
//! within a week (`research.mdx` still said an uncertain birth was read at
//! its edges after the window was read across). So a guide does not hold
//! code of its own. A program block names the file it was cut from in its
//! `title`, which the site shows above it, and this gate holds its lines to
//! that file: each run of lines, between elisions (`// …`, `# …`), must
//! appear in it in order and unbroken, but for the file's comments, which
//! an excerpt may drop. The file must be a shared example, which
//! `check-parity` runs in every binding, or one listed in [`EXECUTED`]
//! with the gate that holds it.
//!
//! Two other kinds of block are allowed, and both are counted:
//!
//! - a **fragment** (`fragment` in the meta) is a program that is not one:
//!   `ctx.calendar.monthLength(...)` set beside its Python spelling. Each is
//!   listed by page and line when the gate passes, so the escape is in
//!   plain sight rather than a quiet default;
//! - an **output** (`output="NAME"` on a `text` block) is lines the shared
//!   example `NAME` prints. This gate checks the example exists;
//!   `check-parity`, which runs the examples, checks the lines against
//!   what it printed ([`printed`]).
//!
//! Everything else -- a shell command, a JSON request, a POM's
//! dependency -- is not a program, and is not checked.
//!
//! The READMEs a consumer reads where a package is published ([`READMES`])
//! are held the same way: a forge or a registry shows a fence's first word
//! as its language and ignores the rest, so `title=` and `fragment` cost
//! the reader nothing there. A crate's README is not among them: its Rust
//! runs as the crate's doctests (`check-lints`' `readme-example-runs`).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::examples::{Binding, Ran};

/// The hand-written guides, beside the generated reference under
/// [`REFERENCE`].
const GUIDES: &str = "site/content/docs";

/// The READMEs each binding and adapter publishes with its package, held
/// as the guides are.
const READMES: [&str; 13] = [
    "README.md",
    "bindings/c/README.md",
    "bindings/dart/README.md",
    "bindings/flutter/README.md",
    "bindings/java/README.md",
    "bindings/node/README.md",
    "bindings/python/README.md",
    "bindings/wasm/README.md",
    "adapters/ephemeris-teimeris/dart/README.md",
    "adapters/ephemeris-teimeris/flutter/README.md",
    "adapters/ephemeris-teimeris/java/README.md",
    "adapters/ephemeris-teimeris/node/README.md",
    "adapters/ephemeris-teimeris/python/README.md",
];
const REFERENCE: &str = "reference";

/// The files outside the shared examples a guide may cut a program from,
/// each with the xtask module that holds it to a gate.
///
/// Listed, not inferred, and held both ways: the module must name the
/// file's package, and an entry no guide cites is stale. Most of these
/// are type-checked rather than run, because running an adapter's
/// consumer needs the engine's data, which a checkout does not have, and
/// running the Flutter app needs a device; the gate's report says which
/// is which.
const EXECUTED: [Source; 5] = [
    Source {
        file: "adapters/ephemeris-teimeris/node/typecheck/consumer.ts",
        module: "xtask/src/node_binding.rs",
        names: "adapters/ephemeris-teimeris/node",
        held: "type-checked by `check-node`",
    },
    Source {
        file: "adapters/ephemeris-teimeris/dart/example/consumer.dart",
        module: "xtask/src/dart_binding.rs",
        names: "adapters/ephemeris-teimeris/dart",
        held: "analysed by `check-dart`",
    },
    Source {
        file: "adapters/ephemeris-teimeris/python/typecheck/consumer.py",
        module: "xtask/src/python_binding.rs",
        names: "adapters/ephemeris-teimeris/python",
        held: "type-checked by `check-python`",
    },
    Source {
        file: "adapters/ephemeris-teimeris/java/typecheck/Consumer.java",
        module: "xtask/src/java_binding.rs",
        names: "adapters/ephemeris-teimeris/java",
        held: "compiled by `check-java`",
    },
    Source {
        file: "bindings/flutter/example/lib/main.dart",
        module: "xtask/src/flutter_package.rs",
        names: "bindings/flutter",
        held: "built into an app by `check-flutter`",
    },
];

/// A file a guide may cite that is not a shared example.
struct Source {
    file: &'static str,
    /// The xtask module whose gate holds it.
    module: &'static str,
    /// What that module names to reach it: the file's package.
    names: &'static str,
    /// How that gate holds it, as the report says.
    held: &'static str,
}

/// A fenced block of a guide.
#[derive(Debug)]
struct Block {
    /// The page, relative to the repository.
    page: String,
    /// The line its opening fence is on, from 1.
    line: usize,
    /// The fence's language.
    language: String,
    /// `title="…"`: the file a program was cut from.
    title: Option<String>,
    /// `output="…"`: the shared example whose output this is.
    output: Option<String>,
    /// `fragment`: a program that is not one, on purpose.
    fragment: bool,
    /// The lines between the fences.
    lines: Vec<String>,
}

impl Block {
    fn at(&self) -> String {
        format!("{}:{}", self.page, self.line)
    }
}

/// What a guide's blocks came to: how many of each kind, and the
/// fragments by place.
#[derive(Debug, Default)]
struct Tally {
    excerpts: usize,
    outputs: usize,
    fragments: Vec<String>,
    cited: BTreeSet<String>,
}

pub(crate) fn check(root: &Path) -> i32 {
    let Ok(blocks) = guide_blocks(root) else {
        return 1;
    };
    let mut tally = Tally::default();
    let mut failures = 0usize;
    let mut pages = BTreeSet::new();
    for block in &blocks {
        pages.insert(block.page.clone());
        if let Err(said) = held(root, block, &mut tally) {
            println!("FAIL  {}: {said}", block.at());
            failures += 1;
        }
    }
    for source in &EXECUTED {
        let named = fs::read_to_string(root.join(source.module))
            .is_ok_and(|text| text.contains(source.names));
        if !named {
            println!(
                "FAIL  {} is listed as {}, but {} does not name {}",
                source.file, source.held, source.module, source.names
            );
            failures += 1;
        }
        if !tally.cited.contains(source.file) {
            println!(
                "FAIL  {} is listed as a guide's source, but no guide cites it",
                source.file
            );
            failures += 1;
        }
    }
    if failures != 0 {
        println!("FAIL  the guides' examples: {failures} finding(s)");
        return 1;
    }
    println!(
        "ok    {} in {} cut from files a gate runs, {} of the shared examples' output, and {}:",
        crate::measure::plural(tally.excerpts, "program"),
        crate::measure::plural(pages.len(), "page"),
        crate::measure::plural(tally.outputs, "block"),
        crate::measure::plural(tally.fragments.len(), "fragment"),
    );
    for source in &EXECUTED {
        println!("        {}  {}", source.file, source.held);
    }
    for fragment in &tally.fragments {
        println!("        fragment  {fragment}");
    }
    0
}

/// Whether one block is what its kind says, counting it in `tally`.
fn held(root: &Path, block: &Block, tally: &mut Tally) -> Result<(), String> {
    if let Some(example) = &block.output {
        if block.language != "text" {
            return Err(format!(
                "an output is a `text` block, not `{}`",
                block.language
            ));
        }
        let shared = root
            .join(Binding::Node.directory())
            .join(format!("{example}.mjs"));
        if !shared.is_file() {
            return Err(format!("`{example}` is not a shared example"));
        }
        tally.outputs += 1;
        return Ok(());
    }
    let Some(language) = Language::of(&block.language) else {
        return Ok(());
    };
    if block.fragment {
        tally.fragments.push(block.at());
        return Ok(());
    }
    let Some(title) = &block.title else {
        return Err(format!(
            "a `{}` program names no source: give it `title=\"<the file it is cut from>\"`, or `fragment` if it is not a program",
            block.language
        ));
    };
    if !language.extensions.iter().any(|ext| title.ends_with(ext)) {
        return Err(format!("{title} is not a `{}` file", block.language));
    }
    if !is_executed(title) {
        return Err(format!(
            "{title} is not a shared example nor a file `EXECUTED` lists as run by a gate"
        ));
    }
    let source = fs::read_to_string(root.join(title)).map_err(|err| format!("{title}: {err}"))?;
    excerpt_of(&block.lines, title, &source, &language).map_err(|line| {
        format!(
            "`{}` is not in {title}, in order, where the block puts it",
            line.trim()
        )
    })?;
    tally.excerpts += 1;
    tally.cited.insert(title.clone());
    Ok(())
}

/// Whether a file is one a gate runs: a shared example of some binding, or
/// listed in [`EXECUTED`].
fn is_executed(file: &str) -> bool {
    EXECUTED.iter().any(|source| source.file == file)
        || Binding::ALL.iter().any(|binding| {
            file.strip_prefix(&binding.directory())
                .and_then(|rest| rest.strip_prefix('/'))
                .is_some_and(|name| !name.contains('/') && name != "parity.rs")
        })
}

/// A program language: the extensions its files have and how a line of it
/// is a comment.
struct Language {
    extensions: &'static [&'static str],
    comments: &'static [&'static str],
}

impl Language {
    const C_LIKE: &'static [&'static str] = &["//", "/*", "*"];

    fn of(fence: &str) -> Option<Language> {
        let (extensions, comments): (&[&str], &[&str]) = match fence {
            "js" | "javascript" => (&[".mjs", ".js"], Self::C_LIKE),
            "ts" | "typescript" => (&[".ts"], Self::C_LIKE),
            "dart" => (&[".dart"], Self::C_LIKE),
            "python" | "py" => (&[".py"], &["#"]),
            "java" => (&[".java"], Self::C_LIKE),
            "rust" | "rs" => (&[".rs"], Self::C_LIKE),
            "c" => (&[".c", ".h"], Self::C_LIKE),
            _ => return None,
        };
        Some(Language {
            extensions,
            comments,
        })
    }

    fn is_comment(&self, line: &str) -> bool {
        self.comments.iter().any(|mark| line.starts_with(mark))
    }
}

/// Whether a line of a block is an elision: what stands between two runs.
fn is_elision(line: &str) -> bool {
    matches!(line, "…" | "// …" | "# …" | "/* … */")
}

/// How a file imports its own package's source, against how a consumer
/// imports the package: the files whose path starts with the first, the
/// specifier they write, and the one a guide shows.
///
/// Listed, so that the one rewrite the gate allows is in plain sight.
const IMPORTS: [(&str, &str, &str); 4] = [
    ("bindings/node/", "'../lib/index.js'", "'@teistro/sdk'"),
    (
        "bindings/node/",
        "'../lib/messages.js'",
        "'@teistro/sdk/messages'",
    ),
    (
        "adapters/ephemeris-teimeris/node/",
        "'../index.js'",
        "'@teistro/ephemeris-teimeris'",
    ),
    (
        "adapters/ephemeris-teimeris/node/",
        "'../web.js'",
        "'@teistro/ephemeris-teimeris/web'",
    ),
];

/// A line of `file` as a guide quotes it: trimmed, and its import of its
/// own package's source spelled as a consumer imports the package.
fn as_quoted(file: &str, line: &str) -> String {
    let mut line = line.trim().to_owned();
    for (prefix, written, shown) in IMPORTS {
        if file.starts_with(prefix) {
            line = line.replace(written, shown);
        }
    }
    line
}

/// Whether `block` is an excerpt of `source`: each run of its lines
/// between elisions found in order and unbroken, but for the source's
/// blank lines and comments. The error is the line the run broke at, at
/// the place in the file where most of it matched.
fn excerpt_of<'a>(
    block: &'a [String],
    file: &str,
    source: &str,
    language: &Language,
) -> Result<(), &'a str> {
    let lines: Vec<String> = source
        .lines()
        .map(|line| as_quoted(file, line))
        .filter(|line| !line.is_empty())
        .collect();
    let mut runs: Vec<Vec<&'a str>> = vec![Vec::new()];
    for line in block {
        let line = line.trim();
        if is_elision(line) {
            runs.push(Vec::new());
        } else if !line.is_empty()
            && let Some(run) = runs.last_mut()
        {
            run.push(line);
        }
    }
    let mut from = 0usize;
    for run in runs.iter().filter(|run| !run.is_empty()) {
        let mut furthest = 0usize;
        let mut found = None;
        for start in from..lines.len() {
            match matched(run, &lines[start..], language) {
                Ok(left) => {
                    found = Some(lines.len() - left);
                    break;
                }
                Err(depth) => furthest = furthest.max(depth),
            }
        }
        from = found.ok_or_else(|| run.get(furthest).copied().unwrap_or_default())?;
    }
    Ok(())
}

/// Whether `run` starts `lines`, skipping comments between its lines:
/// how many of `lines` are left after it, or how many of its lines
/// matched before it broke.
fn matched(run: &[&str], lines: &[String], language: &Language) -> Result<usize, usize> {
    let mut at = lines.iter();
    for (depth, wanted) in run.iter().enumerate() {
        loop {
            let Some(line) = at.next() else {
                return Err(depth);
            };
            if line == wanted {
                break;
            }
            if depth == 0 || !language.is_comment(line) {
                return Err(depth);
            }
        }
    }
    Ok(at.len())
}

/// Every fenced block of every hand-written guide, in page order.
fn guide_blocks(root: &Path) -> Result<Vec<Block>, ()> {
    let mut pages = Vec::new();
    collect(&root.join(GUIDES), &mut pages);
    pages.sort();
    if pages.is_empty() {
        println!("FAIL  {GUIDES} holds no guides");
        return Err(());
    }
    pages.extend(READMES.iter().map(|readme| root.join(readme)));
    let mut blocks = Vec::new();
    for page in pages {
        let text =
            fs::read_to_string(&page).map_err(|err| println!("FAIL  {}: {err}", page.display()))?;
        let name = page
            .strip_prefix(root)
            .unwrap_or(&page)
            .to_string_lossy()
            .replace('\\', "/");
        blocks.extend(blocks_of(&name, &text)?);
    }
    Ok(blocks)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            if path.file_name().is_none_or(|name| name != REFERENCE) {
                collect(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "mdx") {
            out.push(path);
        }
    }
}

/// A page's fenced blocks; an error for a fence left open.
fn blocks_of(page: &str, text: &str) -> Result<Vec<Block>, ()> {
    let mut blocks = Vec::new();
    let mut open: Option<Block> = None;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_start();
        match open.take() {
            Some(block) if line.trim_end() == "```" => blocks.push(block),
            Some(mut block) => {
                block.lines.push(raw.to_owned());
                open = Some(block);
            }
            None => {
                if let Some(info) = line.strip_prefix("```") {
                    open = Some(opened(page, index + 1, info));
                }
            }
        }
    }
    if let Some(block) = open {
        println!("FAIL  {}: the fence is never closed", block.at());
        return Err(());
    }
    Ok(blocks)
}

/// A block from its opening fence's info string: the language, then
/// `key="value"` pairs and bare words.
fn opened(page: &str, line: usize, info: &str) -> Block {
    let info = info.trim();
    let (language, mut rest) = info.split_once(' ').unwrap_or((info, ""));
    let mut block = Block {
        page: page.to_owned(),
        line,
        language: language.to_owned(),
        title: None,
        output: None,
        fragment: false,
        lines: Vec::new(),
    };
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let word_end = rest.find([' ', '=']).unwrap_or(rest.len());
        let (key, after) = rest.split_at(word_end);
        if let Some(quoted) = after.strip_prefix("=\"") {
            let (value, next) = quoted.split_once('"').unwrap_or((quoted, ""));
            match key {
                "title" => block.title = Some(value.to_owned()),
                "output" => block.output = Some(value.to_owned()),
                _ => {}
            }
            rest = next;
        } else {
            if key == "fragment" {
                block.fragment = true;
            }
            rest = after.strip_prefix('=').unwrap_or(after);
        }
    }
    block
}

/// The guides' output blocks held to what the shared examples printed,
/// for `check-parity`, which ran them: each block's lines must be lines
/// its example printed, in order and unbroken. The number of failures,
/// each printed.
pub(crate) fn printed(root: &Path, ran: &[Ran]) -> usize {
    guide_blocks(root).map_or(1, |blocks| outputs_held(&blocks, ran))
}

/// [`printed`], over blocks already read.
fn outputs_held(blocks: &[Block], ran: &[Ran]) -> usize {
    let mut failures = 0usize;
    let mut held = 0usize;
    for block in blocks {
        let Some(example) = &block.output else {
            continue;
        };
        let Some(output) = ran.iter().find(|ran| &ran.name == example) else {
            println!(
                "FAIL  {}: the shared example `{example}` did not run",
                block.at()
            );
            failures += 1;
            continue;
        };
        let lines: Vec<&str> = output.output.lines().map(str::trim_end).collect();
        // Trimmed at the end only: an output's indentation is what it printed.
        let shown: Vec<&str> = block.lines.iter().map(|line| line.trim_end()).collect();
        if lines
            .windows(shown.len())
            .any(|window| window == shown.as_slice())
        {
            held += 1;
        } else {
            println!(
                "FAIL  {}: `{example}` did not print these lines; it printed:\n{}",
                block.at(),
                output.output
            );
            failures += 1;
        }
    }
    if failures == 0 {
        println!(
            "ok    the guides' {} are what the examples printed",
            crate::measure::plural(held, "output block")
        );
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_owned).collect()
    }

    const JS: Language = Language {
        extensions: &[".mjs"],
        comments: Language::C_LIKE,
    };

    const NODE: &str = "bindings/node/example/quickstart.mjs";

    const SOURCE: &str = "import { Context } from '../lib/index.js';\n\
        import { messages } from '../lib/messages.js';\n\
        \n\
        // A context.\n\
        const ctx = new Context({\n\
        \x20 // the profile\n\
        \x20 profile: 'nepali-default',\n\
        });\n\
        console.log(ctx);\n\
        ctx.dispose();\n";

    #[test]
    fn an_excerpt_is_in_order_and_unbroken_but_for_comments() {
        let whole = lines(
            "import { Context } from '@teistro/sdk';\n// …\nconst ctx = new Context({\n  profile: 'nepali-default',\n});",
        );
        assert_eq!(excerpt_of(&whole, NODE, SOURCE, &JS), Ok(()));
        let elided = lines("const ctx = new Context({\n// …\nctx.dispose();");
        assert_eq!(excerpt_of(&elided, NODE, SOURCE, &JS), Ok(()));
        let subpath = lines("import { messages } from '@teistro/sdk/messages';");
        assert_eq!(excerpt_of(&subpath, NODE, SOURCE, &JS), Ok(()));
        // A line skipped without an elision breaks the run.
        let skipped = lines("import { Context } from '@teistro/sdk';\nconst ctx = new Context({");
        assert!(excerpt_of(&skipped, NODE, SOURCE, &JS).is_err());
        // A line dropped from the middle of a run breaks it.
        let broken = lines("const ctx = new Context({\n});");
        assert_eq!(excerpt_of(&broken, NODE, SOURCE, &JS), Err("});"));
        // Runs out of order.
        let reversed = lines("ctx.dispose();\n// …\nconsole.log(ctx);");
        assert_eq!(
            excerpt_of(&reversed, NODE, SOURCE, &JS),
            Err("console.log(ctx);")
        );
        // A line the file does not have.
        let invented = lines("const ctx = new Context({ profile: 'x' });");
        assert!(excerpt_of(&invented, NODE, SOURCE, &JS).is_err());
    }

    #[test]
    fn a_fence_says_its_kind() {
        let blocks = blocks_of(
            "p.mdx",
            "a\n```js title=\"bindings/node/example/quickstart.mjs\"\nx\n```\n\n```python fragment\ny\n```\n```text output=\"research\"\nz\n```\n",
        )
        .unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].line, 2);
        assert_eq!(
            blocks[0].title.as_deref(),
            Some("bindings/node/example/quickstart.mjs")
        );
        assert_eq!(blocks[0].lines, ["x"]);
        assert!(blocks[1].fragment && blocks[1].language == "python");
        assert_eq!(blocks[2].output.as_deref(), Some("research"));
        assert!(blocks_of("p.mdx", "```js\nnever closed\n").is_err());
    }

    #[test]
    fn an_output_is_lines_its_example_printed() {
        let ran = [Ran {
            name: "research".to_owned(),
            output: "8 rules\nregistered as abc\n999 permutations\n".to_owned(),
        }];
        let shown = |text: &str| {
            blocks_of(
                "p.mdx",
                &format!("```text output=\"research\"\n{text}\n```\n"),
            )
            .unwrap()
        };
        assert_eq!(
            outputs_held(&shown("registered as abc\n999 permutations"), &ran),
            0
        );
        assert_eq!(outputs_held(&shown("registered as abd"), &ran), 1);
        // Indentation is part of what was printed.
        assert_eq!(outputs_held(&shown("  registered as abc"), &ran), 1);
        // In order and unbroken.
        assert_eq!(outputs_held(&shown("8 rules\n999 permutations"), &ran), 1);
        let other = blocks_of("p.mdx", "```text output=\"quickstart\"\nx\n```\n").unwrap();
        assert_eq!(outputs_held(&other, &ran), 1);
    }

    #[test]
    fn a_source_is_a_file_a_gate_runs() {
        assert!(is_executed("bindings/node/example/research.mjs"));
        assert!(is_executed("bindings/java/example/Research.java"));
        assert!(is_executed("crates/sdk/examples/research.rs"));
        assert!(!is_executed("crates/sdk/examples/parity.rs"));
        assert!(!is_executed("bindings/node/lib/index.js"));
        assert!(is_executed(EXECUTED[0].file));
        assert!(
            EXECUTED
                .iter()
                .all(|source| source.file.starts_with(source.names))
        );
    }

    #[test]
    fn the_guides_hold() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(check(&root), 0);
    }
}
