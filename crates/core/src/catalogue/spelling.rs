//! Writing catalogue members in full inside JSON a boundary section
//! carries: every binding reads a member back as `nakshatra.ROHINI`, and
//! a section built from serde's bare keys says where its members are and
//! of which kind, then has this write them.

use serde_json::Value;

use super::Kind;

/// Writes the member at a dotted path in full, each of a list's members,
/// and nothing for an absent or null one. A path through a list
/// (`score.factors.graha`) reaches every element, and a value that is
/// itself a list has the path applied to each.
///
/// ```
/// use serde_json::json;
/// use teistro_core::catalogue::{Kind, write_in_full};
///
/// let mut day = json!([{"date": {"calendar": "GREGORIAN"}}, {"date": null}]);
/// write_in_full(&mut day, "date.calendar", Kind::Calendar);
/// assert_eq!(day, json!([{"date": {"calendar": "calendar.GREGORIAN"}}, {"date": null}]));
/// ```
pub fn write_in_full(value: &mut Value, path: &str, kind: Kind) {
    match value {
        Value::Array(list) => {
            for item in list {
                write_in_full(item, path, kind);
            }
        }
        Value::Object(fields) => {
            let (field, rest) = path.split_once('.').unwrap_or((path, ""));
            if let Some(inner) = fields.get_mut(field) {
                if rest.is_empty() {
                    member_in_full(inner, kind);
                } else {
                    write_in_full(inner, rest, kind);
                }
            }
        }
        _ => {}
    }
}

/// A bare member key, or a list of them, as full keys.
fn member_in_full(value: &mut Value, kind: Kind) {
    match value {
        Value::String(key) => *key = format!("{}.{key}", kind.name()),
        Value::Array(list) => {
            for item in list {
                member_in_full(item, kind);
            }
        }
        _ => {}
    }
}
