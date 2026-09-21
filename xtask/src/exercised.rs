//! Which of each consumer surface its own tests and examples ever touch.
//!
//! Beside [`binding-surface-measured.md`], which measures what a binding
//! **is**; this measures what anything has ever **done** with it, and it
//! reads four surfaces: the three bindings' hand-written layers and the
//! Rust areas they mirror.
//!
//! [`binding-surface-measured.md`]: ../../docs/03-design/binding-surface-measured.md
//!
//! `entry-point-is-reachable` holds that every boundary function is
//! **placed** in a binding: exposed, declared, callable. It says nothing
//! about whether anyone has ever called it, and the difference is not
//! academic — `loadPack` was generated into three bindings and executed
//! from none of them, which hid a defect one layer deeper: the bytes
//! crossed, the record landed, and every form the record carried beyond
//! the six `i18n/` declares was dropped on the way out. A whole corpus was
//! unreachable from three languages and every gate was green.
//!
//! So this pass asks the other question. For each surface it reads the
//! **hand-written** declarations — the part no generator owns — and counts
//! how many of them are named anywhere in that surface's own tests or
//! examples. What none of them names is listed by name, because the list
//! is short enough to be exhaustive and a member that stops being
//! exercised has to change this page.
//!
//! The Rust areas are here to be read the others against: they are what
//! each binding's layer classes mirror, and they were exercised member for
//! member while three of the layers that mirror them were not.
//!
//! It searches for the member's name rather than for a call, so a property
//! read counts as much as a call and the count errs towards *exercised*.
//! A name this pass says is untouched is therefore untouched.
//!
//! `cargo xtask exercised` writes the page and `check-exercised`
//! regenerates it in memory and fails on any difference.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};

const PAGE: &str = "docs/03-design/binding-exercise-measured.md";

/// One consumer surface's own declarations, and where its callers live.
struct Layer {
    /// The surface, as the page names it.
    binding: &'static str,
    /// The hand-written surface: the file a generator does not write.
    surface: &'static str,
    /// Every member the surface declares, read from the whole file
    /// because a language may need to know what encloses a line.
    declares: fn(&str) -> BTreeSet<String>,
    /// The directories whose files may name a member, and their extension.
    callers: [&'static str; 2],
    /// The extension a caller's file has.
    extension: &'static str,
}

/// A member's indent: exactly two spaces, which is a class's own level.
///
/// Anything deeper is inside a body — a local helper, a call, a closure —
/// and a consumer cannot name it. `Map<String, Object?> object(Object? v)`
/// is a real declaration and a real function, and it is four spaces in,
/// because it is local to the method that uses it.
fn at_member_indent(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("  ")?;
    (!rest.starts_with(' ')).then_some(rest)
}

/// Every member a TypeScript declaration file declares: two spaces, the
/// member, an open bracket. Everything at that indent in a `.d.ts` is
/// inside a class or an interface, so no enclosing scope is tracked.
fn typescript_members(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let (name, _) = at_member_indent(line)?.split_once('(')?;
            identifier(name).map(str::to_owned)
        })
        .collect()
}

/// Every member the Dart layer declares.
///
/// A member's indent is two spaces, but so is a **local function inside a
/// top-level function** — `List<int> twelve(...)` is declared that way
/// inside `_decodeAshtakavargas` and is nobody's member. So this follows
/// the enclosing scope: a class opens at column zero and closes at a `}`
/// there, and only what falls between counts.
fn dart_members(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut in_class = false;
    for line in text.lines() {
        // A blank line is inside whatever it is inside: it opens nothing
        // and closes nothing, and treating it as column zero shut every
        // class the moment its first paragraph break arrived.
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            in_class = line.contains("class ");
            continue;
        }
        if in_class && let Some(member) = dart_member(line) {
            found.insert(member.to_owned());
        }
    }
    found
}

/// Every method the Python layer declares.
fn python_members(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| python_member(line).map(str::to_owned))
        .collect()
}

/// Every method the Rust areas declare: four spaces, `pub fn` or
/// `pub const fn`, inside an `impl`.
///
/// The areas are the Rust consumer's entry points and the thing each
/// binding's layer classes mirror, so they are the fourth surface this
/// page reads and the one to compare the others against.
fn rust_members(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            // Four spaces exactly: a method of an `impl`, where two is a
            // field of a struct and more is inside a body.
            let rest = line.strip_prefix("    ")?;
            if rest.starts_with(' ') {
                return None;
            }
            let rest = rest.strip_prefix("pub ")?;
            let rest = rest.strip_prefix("const ").unwrap_or(rest);
            let (name, _) = rest.strip_prefix("fn ")?.split_once('(')?;
            identifier(name).map(str::to_owned)
        })
        .collect()
}

/// A Dart declaration: two spaces, a return type, the member, and a body
/// that opens on the same line or continues onto the next.
///
/// The `=` test is what tells a declaration from a statement: `final x =
/// jsonDecode(...)` sits at the same indent inside a body and would
/// otherwise read as a member called `jsonDecode`. Dart's getters are
/// `Foo get name =>` and carry no bracket, so they are not members this
/// pass sees — which is why the language's count is the smallest of the
/// three and not a claim that its surface is.
fn dart_member(line: &str) -> Option<&str> {
    let (before, after) = at_member_indent(line)?.split_once('(')?;
    // `=>` is a body and not a declaration: `Foo get x => jsonDecode(y)`
    // opens with a member's indent and names a function it calls.
    if before.contains('=') || before.contains("return") || before.contains("get ") {
        return None;
    }
    let trailing = after.trim_end();
    if !(trailing.is_empty() || trailing.ends_with('{') || trailing.ends_with("=>")) {
        return None;
    }
    // A declaration has a return type before the name; a call has nothing
    // before it, which is how `jsonDecode(` on its own line is refused.
    let declared = before.trim();
    if !declared.contains(' ') {
        return None;
    }
    identifier(declared.rsplit([' ', '>']).next()?)
}

/// A Python method: four spaces, `def`, the member, `(self`.
fn python_member(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("    def ")?;
    let (name, after) = rest.split_once('(')?;
    after.starts_with("self").then(|| identifier(name))?
}

/// The name, when it is one a consumer could write: lower camel or snake
/// case, and not a private member or a language's own machinery.
fn identifier(name: &str) -> Option<&str> {
    const MACHINERY: [&str; 8] = [
        "constructor",
        "toString",
        "noSuchMethod",
        "hashCode",
        "super",
        "assert",
        "this",
        "if",
    ];
    let name = name.trim();
    let first = name.chars().next()?;
    (first.is_ascii_lowercase()
        && name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '_')
        && !MACHINERY.contains(&name))
    .then_some(name)
}

const LAYERS: [Layer; 4] = [
    Layer {
        binding: "Node",
        surface: "bindings/node/lib/index.d.ts",
        declares: typescript_members,
        callers: ["bindings/node/test", "bindings/node/example"],
        extension: "mjs",
    },
    Layer {
        binding: "Dart",
        surface: "bindings/dart/lib/teistro.dart",
        declares: dart_members,
        callers: ["bindings/dart/test", "bindings/dart/example"],
        extension: "dart",
    },
    Layer {
        binding: "Rust",
        surface: "crates/sdk/src/area",
        declares: rust_members,
        callers: ["crates/sdk/tests", "crates/sdk/examples"],
        extension: "rs",
    },
    Layer {
        binding: "Python",
        surface: "bindings/python/teistro/__init__.py",
        declares: python_members,
        callers: ["bindings/python/tests", "bindings/python/example"],
        extension: "py",
    },
];

/// Every member the layer declares.
///
/// A surface is one file in three of the four languages and a directory in
/// the fourth: Rust's areas are a module each, and reading the directory
/// is what keeps an area added from being an area unmeasured.
fn declared(root: &Path, layer: &Layer) -> Result<BTreeSet<String>, String> {
    let path = root.join(layer.surface);
    if path.is_dir() {
        let mut found = BTreeSet::new();
        let entries =
            std::fs::read_dir(&path).map_err(|why| format!("{}: {why}", layer.surface))?;
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|kind| kind == "rs"))
            .collect();
        paths.sort();
        for file in paths {
            let text = std::fs::read_to_string(&file)
                .map_err(|why| format!("{}: {why}", file.display()))?;
            found.extend((layer.declares)(&text));
        }
        return Ok(found);
    }
    let text = std::fs::read_to_string(&path).map_err(|why| format!("{}: {why}", layer.surface))?;
    Ok((layer.declares)(&text))
}

/// Everything the binding's tests and examples say, as one text.
fn callers(root: &Path, layer: &Layer) -> String {
    let mut said = String::new();
    for directory in layer.callers {
        let Ok(entries) = std::fs::read_dir(root.join(directory)) else {
            continue;
        };
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|kind| kind == layer.extension))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(text) = std::fs::read_to_string(&path) {
                said.push_str(&text);
                said.push('\n');
            }
        }
    }
    said
}

/// Whether the text names the member, as a member and not inside a longer
/// word: `.tithi` is the tithi and `.tithiClass` is not.
fn names(text: &str, member: &str) -> bool {
    let mut from = 0;
    while let Some(at) = text.get(from..).and_then(|rest| rest.find(member)) {
        let start = from + at;
        let end = start + member.len();
        let before = text.get(..start).and_then(|head| head.chars().next_back());
        let after = text.get(end..).and_then(|tail| tail.chars().next());
        let joined =
            |letter: Option<char>| letter.is_some_and(|l| l.is_ascii_alphanumeric() || l == '_');
        if !joined(after) && before == Some('.') {
            return true;
        }
        from = end;
    }
    false
}

fn main_page(root: &Path) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("# What each consumer surface exercises, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask exercised` over each surface's \
         own declarations and its own tests and examples, 2026-09-22. \
         Do not edit: `check-exercised` regenerates this page and fails on \
         any difference.\n\n",
    );
    out.push_str(
        "`entry-point-is-reachable` holds that every boundary function is \
         **placed** in a binding — exposed, declared, callable. It says \
         nothing about whether anyone has ever called it, and that \
         difference cost a whole corpus: `loadPack` was generated into \
         three bindings and executed from none, so nothing showed that a \
         record's forms were being dropped on the way out. This page asks \
         the other question of the four surfaces a generator does **not** \
         own — the three bindings' layers and the Rust areas they mirror, \
         which is the one to read the others against.\n\n",
    );
    out.push_str(
        "A member counts as exercised when its name appears after a dot \
         anywhere in that surface's own tests or examples, so a property read \
         counts as much as a call and the count errs towards exercised. A \
         member named here is therefore one nothing touches.\n\n",
    );
    out.push_str("| surface | declares | exercised | untouched |\n|---|---:|---:|---:|\n");
    let mut untouched_by: Vec<(&str, Vec<String>)> = Vec::new();
    let mut declared_all = 0usize;
    let mut untouched_all = 0usize;
    for layer in &LAYERS {
        let members = declared(root, layer)?;
        let said = callers(root, layer);
        let untouched: Vec<String> = members
            .iter()
            .filter(|member| !names(&said, member))
            .cloned()
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            layer.binding,
            count(members.len()),
            count(members.len() - untouched.len()),
            count(untouched.len())
        );
        declared_all += members.len();
        untouched_all += untouched.len();
        untouched_by.push((layer.binding, untouched));
    }
    out.push('\n');
    let _ = write!(
        out,
        "**{} nothing names**, of {} the four surfaces declare. They are \
         listed rather than counted, because a member that stops being \
         exercised has to change this page and one that starts has to as \
         well.\n\n",
        plural(untouched_all, "member"),
        plural(declared_all, "member"),
    );
    for (binding, untouched) in &untouched_by {
        if untouched.is_empty() {
            let _ = writeln!(out, "- **{binding}**: every member is touched.");
            continue;
        }
        let named: Vec<String> = untouched.iter().map(|m| format!("`{m}`")).collect();
        let _ = writeln!(out, "- **{binding}**: {}", named.join(", "));
    }
    out.push('\n');
    out.push_str(&table(&[Claim::counted(
        "every surface is read and its members found",
        LAYERS
            .iter()
            .filter(|layer| declared(root, layer).is_ok_and(|found| found.is_empty()))
            .count(),
        LAYERS.len(),
    )]));
    out.push('\n');
    Ok(fill(&out))
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    Ok(vec![Output::new(PAGE, main_page(root)?)])
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(out) => write(root, &out),
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}

/// Regenerates the page in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(out) => check(root, &out, "cargo xtask exercised"),
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}
