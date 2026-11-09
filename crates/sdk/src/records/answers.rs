//! The shapes of the records' answers a caller's field mask is checked
//! against, read from `answers.json` beside this file.
//!
//! `cargo xtask answers` writes the file from each record's answer
//! schema, kept to the keywords that say where a field is (`type`,
//! `properties`, `items`, the combinators, `$ref` and the like) and
//! without descriptions, so a library answers a mask without linking
//! schemars and every type's schema: on 2026-11-09 doing so grew the
//! native library by half (27 to 41 MB). `check-answers` holds the file to
//! the schemas.

use std::sync::OnceLock;

use serde_json::{Map, Value};

/// The answers' shapes as `xtask` wrote them.
const SOURCE: &str = include_str!("answers.json");

/// The parsed table: every record's answer and the definitions they share.
pub(crate) struct Shapes {
    answers: Map<String, Value>,
    definitions: Map<String, Value>,
}

impl Shapes {
    /// A record's answer schema, `None` for a record that states none.
    pub(crate) fn of(&self, record: &str) -> Option<Value> {
        self.answers.get(record).cloned()
    }

    /// The definitions the answers refer to.
    pub(crate) const fn definitions(&self) -> &Map<String, Value> {
        &self.definitions
    }
}

/// The table, parsed once on the first mask asked for.
pub(crate) fn shapes() -> &'static Shapes {
    static SHAPES: OnceLock<Shapes> = OnceLock::new();
    SHAPES.get_or_init(|| {
        let mut table: Map<String, Value> = serde_json::from_str(SOURCE).unwrap_or_default();
        let mut part = |name: &str| match table.remove(name) {
            Some(Value::Object(part)) => part,
            _ => Map::new(),
        };
        Shapes {
            answers: part("answers"),
            definitions: part("definitions"),
        }
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::shapes;

    /// The file parses and states the records' answers: a table that
    /// failed to parse would leave every mask unchecked as unsupported.
    #[test]
    fn the_table_states_the_answers() {
        let shapes = shapes();
        assert!(shapes.of("calendar.convert").is_some());
        assert!(!shapes.definitions().is_empty());
    }
}
