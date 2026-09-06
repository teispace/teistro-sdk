//! The reference pages of the documentation site, rendered from the same
//! description every binding is generated from (ADR-0012).
//!
//! The site's reference documents the **C ABI**, because that is the one
//! surface that exists in every binding: each binding's layer is
//! generated from it, so a page that describes an entry point describes
//! what every language is really calling. Each page names what the Node
//! addon and the Dart library call that entry point, so a reader arrives
//! from either language and leaves with the right name.
//!
//! Pages are grouped by the source file the entry point comes from, which
//! is how the boundary is already divided — contexts, keys, calendars,
//! time, the locale engine, positions — so the grouping is generated too
//! and cannot drift from the code.
//!
//! Everything here is Markdown inside MDX, plus `Tabs` and `Tab`, which
//! the site registers globally so that no generated page carries an
//! import. Prose is escaped: a `{` or a `<` in a doc comment is an
//! expression or a tag to MDX, and a reference that fails to build is a
//! reference nobody reads.

use std::fmt::Write;

use crate::emit::c::{c_signature, c_type};
use crate::model::{Api, EnumDef, FunctionDef};
use crate::names::{binding_type_name, c_enum_member, c_type_name, camel, snake};
use crate::rules;

/// One generated file: where it goes under the reference directory, and
/// what is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// The path relative to the reference directory, with forward
    /// slashes, `context/ts_context_open.mdx` among them.
    pub path: String,
    /// The file's whole contents.
    pub text: String,
}

/// Every page of the generated reference, in the order they are written.
#[must_use]
pub fn render(api: &Api) -> Vec<Page> {
    let groups = groups(api);
    let mut pages = vec![
        Page {
            path: String::from("meta.json"),
            text: root_meta(&groups),
        },
        Page {
            path: String::from("index.mdx"),
            text: index(api, &groups),
        },
    ];
    for group in &groups {
        pages.push(Page {
            path: format!("{}/meta.json", group.slug),
            text: group_meta(group),
        });
        for f in &group.functions {
            pages.push(Page {
                path: format!("{}/{}.mdx", group.slug, f.name),
                text: function_page(api, f),
            });
        }
    }
    pages.push(Page {
        path: String::from("types.mdx"),
        text: types_page(api),
    });
    pages.push(Page {
        path: String::from("enums.mdx"),
        text: enums_page(api),
    });
    pages
}

// ── the grouping ───────────────────────────────────────────────────────────

/// A group of entry points: one source file of the boundary.
struct Group<'a> {
    /// The directory name, from the source file's stem.
    slug: String,
    /// The title a reader sees in the sidebar.
    title: String,
    /// The entry points in declaration order.
    functions: Vec<&'a FunctionDef>,
}

/// The entry points grouped by the file they are declared in, in the
/// order the description lists the files.
fn groups(api: &Api) -> Vec<Group<'_>> {
    let mut groups: Vec<Group<'_>> = Vec::new();
    for f in &api.functions {
        let slug = group_slug(&f.source);
        if let Some(group) = groups.iter_mut().find(|g| g.slug == slug) {
            group.functions.push(f);
        } else {
            groups.push(Group {
                title: group_title(&slug),
                slug,
                functions: vec![f],
            });
        }
    }
    groups
}

/// The directory a source file's entry points go in. `lib.rs` holds the
/// library's own calls rather than a subject of its own, so it is named
/// for what it is.
fn group_slug(source: &str) -> String {
    let stem = source
        .rsplit('/')
        .next()
        .unwrap_or(source)
        .trim_end_matches(".rs");
    if stem == "lib" {
        String::from("library")
    } else {
        stem.replace('_', "-")
    }
}

/// The group's title: its slug in words.
fn group_title(slug: &str) -> String {
    match slug {
        "library" => String::from("The library"),
        "intl" => String::from("Locale engine"),
        "ffi" => String::from("Boundary"),
        other => {
            let mut words = other.replace('-', " ");
            if let Some(first) = words.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            words
        }
    }
}

// ── the index and the navigation ───────────────────────────────────────────

/// The reference directory's own `meta.json`: the order of the sidebar.
fn root_meta(groups: &[Group<'_>]) -> String {
    let mut pages: Vec<String> = vec![String::from("\"index\"")];
    pages.extend(groups.iter().map(|g| format!("\"{}\"", g.slug)));
    pages.push(String::from("\"---Shapes---\""));
    pages.push(String::from("\"types\""));
    pages.push(String::from("\"enums\""));
    format!(
        "{{\n  \"title\": \"Reference\",\n  \"description\": \"Every entry point of the C ABI, generated from the API description.\",\n  \"pages\": [{}]\n}}\n",
        pages.join(", ")
    )
}

/// A group's `meta.json`.
fn group_meta(group: &Group<'_>) -> String {
    let pages: Vec<String> = group
        .functions
        .iter()
        .map(|f| format!("\"{}\"", f.name))
        .collect();
    format!(
        "{{\n  \"title\": \"{}\",\n  \"pages\": [{}]\n}}\n",
        group.title,
        pages.join(", ")
    )
}

/// The reference's front page: what the boundary is, and every entry
/// point in one table.
fn index(api: &Api, groups: &[Group<'_>]) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "---\ntitle: Reference\ndescription: Every entry point of the Teistro C ABI, generated from the API description.\n---\n\n\
         The SDK has one boundary: a C ABI of {} entry point{} over {} struct{} and {} enum{}. \
         Every binding's mechanical layer is generated from a description of it, `idl/api.json`, \
         which is extracted from the boundary crates' own source and gated by `cargo xtask check-ffi`. \
         These pages are generated from that same description, so a page cannot describe an entry \
         point the library does not have.\n\n\
         ABI version **{}**, SDK **{}**, symbol prefix `{}`.\n\n\
         ## Where each entry point lives\n\n",
        api.functions.len(),
        plural(api.functions.len()),
        api.structs.len(),
        plural(api.structs.len()),
        api.enums.len(),
        plural(api.enums.len()),
        api.abi_version,
        api.sdk_version,
        api.prefix,
    );
    for group in groups {
        let _ = write!(
            out,
            "### {}\n\n| entry point | what it does |\n|---|---|\n",
            group.title
        );
        for f in &group.functions {
            let _ = writeln!(
                out,
                "| [`{}`]({}/{}) | {} |",
                f.name,
                group.slug,
                f.name,
                one_line(&escape(&summary(&f.doc)))
            );
        }
        out.push('\n');
    }
    let _ = write!(
        out,
        "## The rules every entry point follows\n\n\
         - Every struct begins with `struct_size`, which the caller sets to `sizeof` before the \
         call: a library built from a newer description can tell how much of the struct the caller \
         knows about.\n\
         - A call that can fail returns a status; `{}` reports what went wrong, with the field and \
         a hint when it has them.\n\
         - The library frees what the library allocated. A result blob and an owned string are \
         handed back with the call that frees them; a lent string is valid until the next call on \
         the same context.\n\
         - A panic never crosses the boundary: it becomes an internal status.\n\n\
         ## The sources\n\n{}\n",
        rules::message_function(api).map_or("the error accessor", |f| f.name.as_str()),
        api.sources
            .iter()
            .map(|s| format!("- `{s}`"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    out
}

// ── one entry point ────────────────────────────────────────────────────────

/// A page for one entry point: what it does, what every binding calls it,
/// its declaration, its parameters and what it hands back.
fn function_page(api: &Api, f: &FunctionDef) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "---\ntitle: {}\ndescription: {}\n---\n\n{}\n\n",
        f.name,
        frontmatter(&summary(&f.doc)),
        text(&f.doc)
    );
    let _ = write!(
        out,
        "<Tabs items={{['C', 'Node', 'Dart']}}>\n<Tab value=\"C\">\n\n```c\n{}\n```\n\n</Tab>\n",
        c_signature(api, f)
    );
    let (node, dart) = binding_names(api, f);
    let _ = write!(
        out,
        "<Tab value=\"Node\">\n\n```js\n{node}\n```\n\nThe addon's generated layer; `@teistro/sdk` wraps it.\n\n</Tab>\n\
         <Tab value=\"Dart\">\n\n```dart\n{dart}\n```\n\nThe generated `TeistroLibrary`; `package:teistro` wraps it.\n\n</Tab>\n</Tabs>\n\n"
    );
    out.push_str(&parameters(api, f));
    out.push_str(&handed_back(api, f));
    if let Some(blob) = &f.meta.blob {
        let _ = write!(
            out,
            "## The result blob\n\nThe call fills a blob following the `{blob}` schema, which the \
             binding's generated decoder reads. The schema is in `idl/api.json`, and every binding \
             decodes it from the blob's own bytes rather than copying them.\n\n"
        );
    }
    if let Some(safety) = &f.safety {
        let _ = write!(out, "## Safety\n\n{}\n\n", text(safety));
    }
    let _ = writeln!(out, "Declared in `{}`.", f.source);
    out
}

/// What the Node addon and the Dart library call this entry point.
///
/// A function whose first parameter is a handle is a method on that
/// handle's class in both bindings; anything else is a free function.
fn binding_names(api: &Api, f: &FunctionDef) -> (String, String) {
    for opaque in &api.opaques {
        if rules::methods(api, opaque).iter().any(|m| m.name == f.name) {
            let method = rules::method_name(api, opaque, f);
            // The binding's own name for the handle's class, lower-cased
            // as a reader would name a variable of it.
            let receiver = snake(&binding_type_name(&opaque.name));
            return (
                format!("{receiver}.{}(…)", snake(&method)),
                format!("{receiver}.{}(…)", camel(&method)),
            );
        }
    }
    let bare = f.name.strip_prefix(&api.prefix).unwrap_or(&f.name);
    (
        format!("{}(…)", snake(bare)),
        format!("library.{}(…)", camel(bare)),
    )
}

/// The parameters table, with everything the description knows about
/// each one: what it is for, its unit, its range and an example.
fn parameters(api: &Api, f: &FunctionDef) -> String {
    if f.params.is_empty() {
        return String::from("The call takes no arguments.\n\n");
    }
    let mut out = String::from(
        "## Parameters\n\n| name | type | role | what it carries |\n|---|---|---|---|\n",
    );
    for p in &f.params {
        let mut notes = p.meta.enum_name.as_ref().map_or_else(String::new, |name| {
            format!("One of `{}`. ", c_type_name(&api.prefix, name))
        });
        if let Some(unit) = &p.meta.unit {
            let _ = write!(notes, "Unit `{unit}`. ");
        }
        if let Some(range) = &p.meta.range {
            let _ = write!(notes, "Range `{range}`. ");
        }
        if let Some(example) = &p.meta.example {
            let _ = write!(notes, "For example `{example}`. ");
        }
        if let Some(brand) = &p.meta.brand {
            let _ = write!(
                notes,
                "Carries a {brand}, which the typed bindings refuse to swap. "
            );
        }
        if p.meta.nullable {
            notes.push_str("May be null. ");
        }
        if let Some(len) = &p.meta.len {
            let _ = write!(notes, "Its length is `{len}`. ");
        }
        let notes = notes.trim_end();
        let _ = writeln!(
            out,
            "| `{}` | `{}` | {} | {} |",
            p.name,
            c_type(api, &p.ty),
            role(p.role),
            if notes.is_empty() { "—" } else { notes }
        );
    }
    out.push('\n');
    out
}

/// What a call hands back, which is never only its return value: a status
/// function writes through its out parameters.
fn handed_back(api: &Api, f: &FunctionDef) -> String {
    let results = rules::results(api, f);
    if results.is_empty() {
        return String::new();
    }
    let names: Vec<String> = results.iter().map(|r| format!("`{}`", r.name())).collect();
    let mut out = String::from("## What it hands back\n\n");
    let _ = write!(out, "{}", list(&names));
    if rules::returns_status(api, f) {
        out.push_str(
            ". The call returns a status; a status other than `TS_STATUS_OK` means nothing was written.\n\n",
        );
    } else {
        out.push_str(".\n\n");
    }
    out
}

/// A parameter's role in words.
fn role(role: crate::model::Role) -> &'static str {
    use crate::model::Role;
    match role {
        Role::Value => "a value",
        Role::Handle => "the handle",
        Role::HandleOut => "receives a handle",
        Role::StructIn => "a struct read",
        Role::StructOut => "a struct written",
        Role::VtableIn => "a provider",
        Role::UserData => "the provider's data",
        Role::StringIn => "a string read",
        Role::StringOut => "receives a string",
        Role::BytesIn => "bytes read",
        Role::BlobOut => "receives a blob",
        Role::BlobFree => "the blob being freed",
        Role::StringFree => "the string being freed",
        Role::StrOut => "receives a lent string",
        Role::ScalarOut => "receives a number",
        Role::ArrayIn => "an array read",
        Role::Length => "a length",
    }
}

// ── the shapes ─────────────────────────────────────────────────────────────

/// Every struct the boundary declares, with its fields.
fn types_page(api: &Api) -> String {
    let mut out = String::from(
        "---\ntitle: Structs\ndescription: Every struct of the C ABI, with each field's unit, range and example.\n---\n\n\
         Every struct begins with `struct_size`, which a caller sets to `sizeof` before the call. \
         A field a library may not fill is optional in the typed bindings, and a field with a unit \
         or a range carries it here because the description carries it.\n\n",
    );
    for s in &api.structs {
        let _ = write!(
            out,
            "## {}\n\n{}\n\n| field | type | what it carries |\n|---|---|---|\n",
            c_type_name(&api.prefix, &s.name),
            text(&s.doc)
        );
        for field in &s.fields {
            if !rules::is_visible(field) {
                continue;
            }
            let mut notes = text(&field.doc);
            if let Some(name) = &field.meta.enum_name {
                let _ = write!(notes, " One of `{}`.", c_type_name(&api.prefix, name));
            }
            for (label, value) in [
                ("Unit", field.meta.unit.as_deref()),
                ("Range", field.meta.range.as_deref()),
                ("Example", field.meta.example.as_deref()),
            ] {
                if let Some(value) = value {
                    let _ = write!(notes, " {label} `{value}`.");
                }
            }
            let _ = writeln!(
                out,
                "| `{}` | `{}` | {} |",
                field.name,
                c_type(api, &field.ty),
                one_line(&notes)
            );
        }
        out.push('\n');
    }
    out
}

/// Every enum, with its members and their values.
fn enums_page(api: &Api) -> String {
    let mut out = String::from(
        "---\ntitle: Enums\ndescription: Every enum of the C ABI, with the value each member carries.\n---\n\n\
         An enum crosses the boundary as its integer. The catalogue's kinds keep their values \
         forever: a member's number is part of the ABI, and a stored result read back years later \
         means what it meant when it was written.\n\n",
    );
    for e in &api.enums {
        out.push_str(&enum_section(api, e));
    }
    out
}

/// One enum's section.
fn enum_section(api: &Api, e: &EnumDef) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "## {}\n\n{}\n\n| member | value | what it is |\n|---|---|---|\n",
        c_type_name(&api.prefix, &e.name),
        text(&e.doc)
    );
    for value in &e.values {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            c_enum_member(&api.prefix, &e.name, &value.name, value.key.as_deref()),
            value.value,
            one_line(&text(&value.doc))
        );
    }
    out.push('\n');
    out
}

// ── the small shared things ────────────────────────────────────────────────

/// The first sentence of a doc comment, for a summary line.
fn summary(doc: &str) -> String {
    let first = doc
        .split("\n\n")
        .next()
        .unwrap_or(doc)
        .replace('\n', " ")
        .trim()
        .to_string();
    match first.find(". ") {
        Some(at) => first[..=at].trim().to_string(),
        None => first,
    }
}

/// A summary safe to sit in YAML frontmatter on one line. Frontmatter is
/// YAML rather than MDX, so nothing is escaped for MDX here; a
/// double-quoted YAML scalar takes the same escapes a Rust debug string
/// writes.
fn frontmatter(summary: &str) -> String {
    format!("{:?}", summary.replace('\n', " ").trim())
}

/// A doc comment's paragraphs joined onto single lines.
///
/// A hard-wrapped paragraph renders the same either way, and joining it
/// removes the one construct MDX cannot survive: a line that begins with
/// `{`. MDX looks for an expression at the start of a line before it
/// resolves inline code, so `` `{"a": {...}} `` wrapped across two lines
/// puts `{...}` at a line's start and the build fails on invalid
/// JavaScript. A paragraph on one line has only one line start, and that
/// one is escaped like any other prose.
///
/// A block that is a list, a heading, a quotation or a fence keeps its
/// lines, because there the line breaks are the meaning.
fn reflow(doc: &str) -> String {
    doc.split("\n\n")
        .map(|block| {
            let structural = block.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with("- ")
                    || line.starts_with("* ")
                    || line.starts_with('#')
                    || line.starts_with('>')
                    || line.starts_with("```")
                    || line.starts_with('|')
                    || line.split_once(". ").is_some_and(|(n, _)| {
                        !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())
                    })
            });
            if structural {
                block.to_string()
            } else {
                block
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Prose reflowed and with MDX's own syntax escaped: what every page
/// puts in front of a reader.
fn text(doc: &str) -> String {
    escape(&reflow(doc))
}

/// Prose with MDX's own syntax escaped.
///
/// A doc comment is Rust prose: it holds `<`, `{` and `}` in ordinary
/// sentences, and every one of them is a tag or an expression to MDX.
/// Inside a backtick span MDX parses nothing, so those are left alone.
fn escape(doc: &str) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut in_code = false;
    for c in doc.chars() {
        match c {
            '`' => {
                in_code = !in_code;
                out.push('`');
            }
            '<' | '{' | '}' if !in_code => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Prose on one line, for a table cell: a newline would end the row and a
/// pipe would start a column.
fn one_line(text: &str) -> String {
    text.replace('\n', " ")
        .replace('|', "\\|")
        .trim()
        .to_string()
}

/// `a`, `a and b`, `a, b and c`.
fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The plural `s`, or nothing.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::{group_slug, group_title, list, one_line, summary, text};

    #[test]
    fn a_paragraph_is_joined_onto_one_line() {
        // MDX reads a `{` at the start of a line as an expression, even
        // inside what a Markdown reader would call code.
        assert_eq!(
            text("an entity is `{\"$entity\":\n\"graha.SUN\"}` today"),
            "an entity is `{\"$entity\": \"graha.SUN\"}` today"
        );
        assert_eq!(text("one\n\n- a\n- b"), "one\n\n- a\n- b");
    }

    #[test]
    fn mdx_syntax_in_prose_is_escaped() {
        assert_eq!(text("a < b"), "a \\< b");
        assert_eq!(text("`0xFFFF` for none"), "`0xFFFF` for none");
        assert_eq!(text("`a<b>` and c<d>"), "`a<b>` and c\\<d>");
        assert_eq!(text("{x}"), "\\{x\\}");
    }

    #[test]
    fn a_table_cell_stays_one_row() {
        assert_eq!(one_line("a\nb | c"), "a b \\| c");
    }

    #[test]
    fn a_summary_is_the_first_sentence() {
        assert_eq!(summary("Opens it. And more."), "Opens it.");
        assert_eq!(summary("One line only"), "One line only");
        assert_eq!(
            summary("Wrapped\nover two lines."),
            "Wrapped over two lines."
        );
    }

    #[test]
    fn groups_are_named_for_their_source() {
        assert_eq!(group_slug("crates/ffi/src/lib.rs"), "library");
        assert_eq!(group_slug("crates/ffi/src/calendar.rs"), "calendar");
        assert_eq!(group_title("calendar"), "Calendar");
        assert_eq!(group_title("library"), "The library");
    }

    #[test]
    fn lists_read_as_english() {
        let items = |n: usize| -> Vec<String> {
            ["a", "b", "c"]
                .iter()
                .take(n)
                .map(|s| (*s).to_string())
                .collect()
        };
        assert_eq!(list(&items(1)), "a");
        assert_eq!(list(&items(2)), "a and b");
        assert_eq!(list(&items(3)), "a, b and c");
    }
}
