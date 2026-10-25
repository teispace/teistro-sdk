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
//! - [`java`]: the Java binding's FFM layer: the enums, the exception and
//!   the raw layouts and method handles;
//! - [`python`]: the Python binding's `ctypes` layer, its typed classes
//!   and its decoders;
//! - [`mdx`]: the documentation site's reference pages, one per entry
//!   point;
//! - [`records`]: the JSON records that cross as text, typed and decoded
//!   in TypeScript, Python and Dart;
//! - [`validators`]: the corpus every binding's validating constructors
//!   and the Rust newtypes are held to alike.
//!
//! [`reserved`] holds each target's reserved words, so an emitter's
//! renaming rule and the pass that measures it read one list.

pub mod c;
pub mod dart;
pub mod java;
pub mod mdx;
pub mod node;
pub mod python;
pub mod records;
pub mod reserved;
pub mod ts;
pub mod validators;

use std::fmt::Write;

use crate::model::FieldDef;

/// What a nullable catalogue field holds for "none" at the C boundary:
/// every id is a `u16`, and no member is numbered this high. A binding
/// writes it for its own null and reads it back as null, never as a
/// member — `0` is a member (`era.VIKRAMA`), so a null written as zero
/// would read back as a real one.
///
/// ```
/// use teistro_idl::emit::NO_MEMBER;
///
/// // Above every id a kind numbers.
/// assert_eq!(u16::from_str_radix(NO_MEMBER.trim_start_matches("0x"), 16), Ok(u16::MAX));
/// ```
pub const NO_MEMBER: &str = "0xFFFF";

/// What a nullable enum field holds for none, by its width: a one-byte
/// field cannot hold [`NO_MEMBER`], so it holds `0xFF`, the most a byte
/// holds, as a two-byte field holds the most two do.
///
/// ```
/// # use teistro_idl::Api;
/// #
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
///
/// use teistro_idl::emit::no_member_of;
/// use teistro_idl::model::{FieldDef, Meta};
/// use teistro_idl::{Scalar, TypeRef};
///
/// let field = |scalar| FieldDef { name: "sign".into(), ty: TypeRef::scalar(scalar), doc: String::new(), meta: Meta::default() };
/// // A byte cannot hold `0xFFFF`.
/// assert_eq!(no_member_of(&field(Scalar::U8)), Some("0xFF"));
/// assert_eq!(no_member_of(&field(Scalar::U16)), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn no_member_of(field: &FieldDef) -> Option<&'static str> {
    matches!(field.ty.as_scalar(), Some(crate::model::Scalar::U8)).then_some("0xFF")
}

/// How a field's metadata is spelled inside its documentation.
///
/// ```
/// # use teistro_idl::Api;
/// #
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
///
/// use teistro_idl::emit::{DocStyle, field_doc};
///
/// let longitude = &api.struct_named("ObserverC").ok_or("declared")?.fields[0];
/// assert!(field_doc(longitude, DocStyle::JsDoc).ends_with("@range [-180,180]\n@example 85.3240"));
/// // Markdown reads a bare `[-180,180]` as a link, so it is code.
/// assert!(field_doc(longitude, DocStyle::Markdown).contains("Range: `[-180,180]`."));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocStyle {
    /// `Unit: deg. Range: [1,5]. Example: 3.` on one line after the text.
    Prose,
    /// `@unit deg`, `@range [1,5]`, `@example 3`, one per line.
    JsDoc,
    /// [`DocStyle::Prose`] for a Rust doc comment, whose Markdown reads a
    /// bare `[1,5]` as a link: the range and the example are code spans.
    Markdown,
}

/// A field's documentation with the enum it stands for, its unit, range
/// and example appended; `enum_name` is the spelling the target uses for
/// the linked enum (the C name in the header, the binding name elsewhere).
///
/// ```
/// # use teistro_idl::Api;
/// #
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
///
/// use teistro_idl::emit::{DocStyle, field_doc};
///
/// let longitude = &api.struct_named("ObserverC").ok_or("declared")?.fields[0];
/// assert_eq!(field_doc(longitude, DocStyle::Prose), "Degrees, east positive.\nUnit: deg. Range: [-180,180]. Example: 85.3240.");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn field_doc(field: &FieldDef, style: DocStyle) -> String {
    field_doc_with(field, style, field.meta.enum_name.as_deref())
}

/// [`field_doc`] with the linked enum spelled as given.
///
/// ```
/// # use teistro_idl::Api;
/// #
/// # let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
/// # let api: Api = serde_json::from_str(&std::fs::read_to_string(root.join("idl/api.json"))?)?;
///
/// use teistro_idl::emit::{DocStyle, field_doc_with};
///
/// // The linked enum spelled as the target spells it.
/// let request = api.struct_named("PositionRequestC").ok_or("declared")?;
/// let scale = request.fields.iter().find(|field| field.name == "scale").ok_or("a field")?;
/// assert!(field_doc_with(scale, DocStyle::Prose, Some("ts_time_scale")).contains("Enum: ts_time_scale."));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
            DocStyle::Prose | DocStyle::Markdown => {
                let separator = if first { "\n" } else { " " };
                let label = capitalised(tag);
                if style == DocStyle::Markdown && matches!(tag, "range" | "example") {
                    let _ = write!(text, "{separator}{label}: `{value}`.");
                } else {
                    let _ = write!(text, "{separator}{label}: {value}.");
                }
            }
            DocStyle::JsDoc => {
                let _ = write!(text, "\n@{tag} {value}");
            }
        }
        first = false;
    }
    if field.meta.nullable {
        match style {
            DocStyle::Prose | DocStyle::Markdown => text.push_str(" May be null."),
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
/// another language (`text`, `json`) is kept. A dropped example leaves
/// the blank lines either side of it, and one of them goes with it.
///
/// ```
/// use teistro_idl::emit::without_rust_examples;
///
/// let doc = "The scale.\n\n```\nassert!(true);\n```\n\n```text\n1 2 3\n```";
/// // The Rust example goes, with one of its blank lines; a text block stays.
/// assert_eq!(without_rust_examples(doc), "The scale.\n\n```text\n1 2 3\n```");
/// ```
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
    /// Where a line stands among the fences.
    #[derive(PartialEq)]
    enum Fence {
        /// Prose.
        Outside,
        /// Inside a block that is kept, whose blank lines are its own.
        Kept,
        /// Inside an example that is dropped.
        Dropped,
    }
    let mut out = String::with_capacity(text.len());
    let mut fence = Fence::Outside;
    for line in text.lines() {
        if let Some(tag) = line.trim_start().strip_prefix("```") {
            let (next, shown) = match fence {
                Fence::Outside if RUST.contains(&tag.trim()) => (Fence::Dropped, false),
                Fence::Outside => (Fence::Kept, true),
                Fence::Kept => (Fence::Outside, true),
                Fence::Dropped => (Fence::Outside, false),
            };
            fence = next;
            if shown {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }
        let doubled = fence == Fence::Outside
            && line.trim().is_empty()
            && (out.is_empty() || out.ends_with("\n\n"));
        if fence != Fence::Dropped && !doubled {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

/// A comment where every line carries the same marker (`/// `, `// `).
///
/// ```
/// use teistro_idl::emit::line_comment;
///
/// assert_eq!(line_comment("One.\n\nTwo.", "  ", "/// "), "  /// One.\n  ///\n  /// Two.\n");
/// ```
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
///
/// ```
/// use teistro_idl::emit::block_comment;
///
/// // A `*/` in the text cannot end the comment early.
/// assert_eq!(block_comment("a */ b", ""), "/**\n * a * / b\n */\n");
/// assert_eq!(block_comment("", ""), "");
/// ```
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::model::{Meta, Scalar, TypeRef};

    #[test]
    fn documentation_carries_the_metadata_in_every_style() {
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
            field_doc(&field, DocStyle::Markdown),
            "The longitude.\nUnit: deg. Range: `[0,360)`. Example: `280.46`. May be null."
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

    #[test]
    fn a_kept_block_closes_and_keeps_its_blank_lines() {
        let doc =
            "A.\n\n```\nlet x = 1;\n```\n\n```text\n1\n\n2\n```\n\nB.\n\n```rust\nf();\n```\n\nC.";
        // The prose after a kept block survives, and the block's own blank
        // line is not taken for one either side of a dropped example.
        assert_eq!(
            without_rust_examples(doc),
            "A.\n\n```text\n1\n\n2\n```\n\nB.\n\nC."
        );
    }
}
