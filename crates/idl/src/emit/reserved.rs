//! Each target's reserved words, in one place.
//!
//! An emitter has to rename a member the description calls `Return` or
//! `None` before it writes a file the target's own compiler will read,
//! and ADR-0007 recorded that as a defect found by hand: Diplomat emitted
//! a Dart enum member called `true`. A list held privately inside one
//! emitter cannot be measured, so the lists live here and
//! `cargo xtask surface` counts what each of them catches.
//!
//! The rule an emitter applies is the target's own convention for a name
//! that collides: Dart appends `Value`, Python appends the trailing
//! underscore of PEP 8. Both are in [`renamed`].
//!
//! ```
//! use teistro_idl::emit::reserved::{DART, PYTHON, is_reserved, renamed};
//!
//! assert!(is_reserved("return", DART) && is_reserved("return", PYTHON));
//! assert_eq!(renamed("from", PYTHON, "_"), "from_");
//! assert_eq!(renamed("returnValue", DART, "Value"), "returnValue");
//! ```

/// Dart's reserved words, which no identifier may be.
pub const DART: &[&str] = &[
    "true", "false", "null", "default", "new", "in", "is", "as", "do", "if", "for", "switch",
    "this", "super", "var", "final", "const", "class", "enum", "void", "return", "with", "extends",
    "assert",
];

/// Python's hard keywords, which no identifier may be.
pub const PYTHON: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

/// Python's soft keywords, which are legal identifiers but read as
/// keywords in the one grammar each belongs to; an emitter may use them
/// and a reader should be told.
pub const PYTHON_SOFT: &[&str] = &["_", "case", "match", "type"];

/// The names `enum.Enum` keeps for itself, which a member may not take.
pub const PYTHON_ENUM: &[&str] = &["mro", "name", "value"];

/// JavaScript's and TypeScript's reserved words. Members of a generated
/// enum are string literals there rather than identifiers, so this list
/// bears on field and parameter names alone.
pub const TYPESCRIPT: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Whether a name is one of a target's reserved words.
#[must_use]
pub fn is_reserved(name: &str, list: &[&str]) -> bool {
    list.contains(&name)
}

/// A name with the target's collision rule applied: `suffix` appended
/// when the name is reserved, and the name itself when it is not.
#[must_use]
pub fn renamed(name: &str, list: &[&str], suffix: &str) -> String {
    if is_reserved(name, list) {
        format!("{name}{suffix}")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{DART, PYTHON, PYTHON_ENUM, PYTHON_SOFT, TYPESCRIPT, is_reserved, renamed};

    #[test]
    fn every_list_is_sorted_within_itself_and_free_of_repeats() {
        // A list a reader scans has to be scannable, and a word entered
        // twice is a word someone thought was missing.
        for list in [DART, PYTHON, PYTHON_SOFT, PYTHON_ENUM, TYPESCRIPT] {
            let mut sorted: Vec<&str> = list.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), list.len(), "a word appears twice in {list:?}");
        }
        // Python's own keyword list, as the language documents it.
        assert_eq!(PYTHON.len(), 35);
        assert!(
            PYTHON.windows(2).all(|pair| pair.first() < pair.last()),
            "sorted"
        );
    }

    #[test]
    fn a_collision_is_renamed_by_the_targets_own_convention() {
        assert_eq!(renamed("from", PYTHON, "_"), "from_");
        assert_eq!(renamed("return", DART, "Value"), "returnValue");
        assert_eq!(renamed("longitude", PYTHON, "_"), "longitude");
        assert!(is_reserved("yield", TYPESCRIPT));
        assert!(!is_reserved("from", TYPESCRIPT), "contextual, not reserved");
        assert!(!is_reserved("match", PYTHON), "soft, and legal");
        assert!(is_reserved("match", PYTHON_SOFT));
    }
}
