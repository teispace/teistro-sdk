//! The emitters: each renders one binding's mechanical layer from the
//! description and nothing else. The documentation helpers live here so
//! every emitter spells a unit, a range and an example the same way.
//!
//! - [`c`]: the header every other binding is generated against;
//! - [`ts`]: the TypeScript surface and the JavaScript decoders of the
//!   Node and wasm bindings;
//! - [`node`]: the Node addon's napi glue over the C ABI;
//! - [`dart`]: the Dart binding's `dart:ffi` layer, its typed classes and
//!   its decoders;
//! - [`mdx`]: the documentation site's reference pages, one per entry
//!   point.

pub mod c;
pub mod dart;
pub mod mdx;
pub mod node;
pub mod ts;

use std::fmt::Write;

use crate::model::FieldDef;

/// How a field's metadata is spelled inside its documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocStyle {
    /// `Unit: deg. Range: [1,5]. Example: 3.` on one line after the text.
    Prose,
    /// `@unit deg`, `@range [1,5]`, `@example 3`, one per line.
    JsDoc,
}

/// A field's documentation with the enum it stands for, its unit, range
/// and example appended; `enum_name` is the spelling the target uses for
/// the linked enum (the C name in the header, the binding name elsewhere).
#[must_use]
pub fn field_doc(field: &FieldDef, style: DocStyle) -> String {
    field_doc_with(field, style, field.meta.enum_name.as_deref())
}

/// [`field_doc`] with the linked enum spelled as given.
#[must_use]
pub fn field_doc_with(field: &FieldDef, style: DocStyle, enum_name: Option<&str>) -> String {
    let mut text = field.doc.clone();
    let tags = [
        ("enum", enum_name),
        ("unit", field.meta.unit.as_deref()),
        ("range", field.meta.range.as_deref()),
        ("example", field.meta.example.as_deref()),
    ];
    let mut first = true;
    for (tag, value) in tags {
        let Some(value) = value else { continue };
        match style {
            DocStyle::Prose => {
                let separator = if first { "\n" } else { " " };
                let label = capitalised(tag);
                let _ = write!(text, "{separator}{label}: {value}.");
            }
            DocStyle::JsDoc => {
                let _ = write!(text, "\n@{tag} {value}");
            }
        }
        first = false;
    }
    if field.meta.nullable {
        match style {
            DocStyle::Prose => text.push_str(" May be null."),
            DocStyle::JsDoc => text.push_str("\n@nullable"),
        }
    }
    text
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// Documentation without the Rust examples a boundary crate's items
/// carry: a fenced block with no language, or one tagged as Rust, is an
/// example in a language the reader of a binding does not write, so it is
/// dropped rather than shown as if it were theirs. A block tagged with
/// another language (`text`, `json`) is kept.
#[must_use]
pub fn without_rust_examples(text: &str) -> String {
    const RUST: [&str; 6] = [
        "",
        "rust",
        "ignore",
        "no_run",
        "compile_fail",
        "should_panic",
    ];
    let mut out = String::with_capacity(text.len());
    let mut dropping = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(tag) = trimmed.strip_prefix("```") {
            if dropping {
                dropping = false;
                continue;
            }
            if RUST.contains(&tag.trim()) {
                dropping = true;
                continue;
            }
        }
        if !dropping {
            out.push_str(line);
            out.push('\n');
        }
    }
    // A dropped example often leaves the blank line that introduced it.
    while out.ends_with("\n\n") {
        out.pop();
    }
    out.trim_end().to_string()
}

/// A comment where every line carries the same marker (`/// `, `// `).
#[must_use]
pub fn line_comment(text: &str, indent: &str, marker: &str) -> String {
    let text = without_rust_examples(text);
    let mut out = String::new();
    for line in text.lines() {
        if line.is_empty() {
            let _ = writeln!(out, "{indent}{}", marker.trim_end());
        } else {
            let _ = writeln!(out, "{indent}{marker}{line}");
        }
    }
    out
}

/// A `/** … */` block, as in C and JavaScript documentation comments;
/// empty text renders nothing, and a `*/` inside the text is defused.
#[must_use]
pub fn block_comment(text: &str, indent: &str) -> String {
    let text = without_rust_examples(text);
    if text.trim().is_empty() {
        return String::new();
    }
    let mut out = format!("{indent}/**\n");
    for line in text.lines() {
        let line = line.replace("*/", "* /");
        if line.is_empty() {
            let _ = writeln!(out, "{indent} *");
        } else {
            let _ = writeln!(out, "{indent} * {line}");
        }
    }
    let _ = writeln!(out, "{indent} */");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Meta, Scalar, TypeRef};

    #[test]
    fn documentation_carries_the_metadata_in_both_styles() {
        let field = FieldDef {
            name: "lon_deg".into(),
            ty: TypeRef::scalar(Scalar::F64),
            doc: "The longitude.".into(),
            meta: Meta {
                unit: Some("deg".into()),
                range: Some("[0,360)".into()),
                example: Some("280.46".into()),
                nullable: true,
                ..Meta::default()
            },
        };
        assert_eq!(
            field_doc(&field, DocStyle::Prose),
            "The longitude.\nUnit: deg. Range: [0,360). Example: 280.46. May be null."
        );
        assert_eq!(
            field_doc(&field, DocStyle::JsDoc),
            "The longitude.\n@unit deg\n@range [0,360)\n@example 280.46\n@nullable"
        );
        assert_eq!(
            line_comment("a\n\nb", "  ", "/// "),
            "  /// a\n  ///\n  /// b\n"
        );
        assert_eq!(
            block_comment("x */ y\n\nz", ""),
            "/**\n * x * / y\n *\n * z\n */\n"
        );
        assert_eq!(block_comment("  ", ""), "");
    }
}
