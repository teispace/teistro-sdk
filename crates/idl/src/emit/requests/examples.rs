//! One example of every record's request, as deep as its schema goes:
//! every field written, a catalogue member at every place one stands. The
//! bindings read each into their own request type and write it back, so a
//! member nested anywhere that a binding writes as its id, or leaves out,
//! is a failure in that binding's suite rather than a wrong answer.
//!
//! A member is the catalogue's second where it has one, so a member that
//! crosses as its id cannot pass as the first member's default. A raw part
//! (a value every binding hands through as JSON) has no type to read it
//! into, so it is taken from the record's hand-written example at the same
//! place, and left out where that example has none. The values need not
//! make sense together: the examples are held to the schema, not answered.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::model::Api;
use crate::records::{NamedValue, RecordDef, RecordField, RecordShape, RecordType};

/// Each record's example request, `[{"record": …, "request": …}]`, in the
/// order of the roots; `hand` is the hand-written examples, the same shape,
/// which give the raw parts.
#[must_use]
pub fn deep_examples(api: &Api, hand: &Value) -> Value {
    let writer = Writer::of(api);
    let hand_of = |record: &str| {
        hand.as_array()
            .and_then(|all| all.iter().find(|one| one["record"] == record))
            .map(|one| &one["request"])
    };
    let examples: Vec<Value> = api
        .requests
        .roots
        .iter()
        .map(|root| {
            let request = writer.record(&root.name, &mut Vec::new(), hand_of(&root.record));
            json!({ "record": root.record, "request": request })
        })
        .collect();
    Value::Array(examples)
}

struct Writer<'a> {
    records: BTreeMap<&'a str, &'a RecordDef>,
    /// Each catalogue kind's members, as full keys.
    members: BTreeMap<&'a str, Vec<String>>,
}

impl<'a> Writer<'a> {
    fn of(api: &'a Api) -> Writer<'a> {
        let records = api
            .requests
            .records
            .iter()
            .map(|record| (record.name.as_str(), record))
            .collect();
        let members = api
            .enums
            .iter()
            .filter_map(|e| {
                let kind = e.kind.as_deref()?;
                let keys = e
                    .values
                    .iter()
                    .map(|v| format!("{kind}.{}", v.key))
                    .collect();
                Some((kind, keys))
            })
            .collect();
        Writer { records, members }
    }

    /// A member of `catalogue`: its second, or its only one.
    fn member(&self, catalogue: &str) -> Option<Value> {
        let keys = self.members.get(catalogue)?;
        keys.get(1)
            .or_else(|| keys.first())
            .map(|key| Value::String(key.clone()))
    }

    /// The values of `ty` when it names a union of several records.
    fn several(&self, ty: &RecordType) -> Option<&'a [NamedValue]> {
        let RecordType::Record { name } = ty else {
            return None;
        };
        match &self.records.get(name.as_str())?.shape {
            RecordShape::OrNamed { values, .. } if values.len() > 1 => Some(values),
            _ => None,
        }
    }

    /// The record `name`, beside `hand`, the hand example's value at the
    /// same place; `None` where it cannot be written whole, as where it is
    /// already being written further up (a recursive record).
    fn record(&self, name: &str, path: &mut Vec<String>, hand: Option<&Value>) -> Option<Value> {
        if path.iter().any(|p| p == name) {
            return None;
        }
        let record = self.records.get(name)?;
        path.push(name.to_owned());
        let value = match &record.shape {
            RecordShape::Object { fields } => self.fields(fields, path, None, hand),
            RecordShape::Keys { values } => values
                .get(1)
                .or_else(|| values.first())
                .map(|v| Value::String(v.key.clone())),
            // The variant with the most fields that can be written whole.
            RecordShape::Tagged { tag, variants } => {
                let mut widest: Vec<_> = variants.iter().collect();
                widest.sort_by_key(|v| core::cmp::Reverse(v.fields.len()));
                widest.into_iter().find_map(|variant| {
                    let tagged = Some((tag.as_str(), variant.key.as_str()));
                    let hand = hand.filter(|h| h.get(tag) == Some(&json!(variant.key)));
                    self.fields(&variant.fields, path, tagged, hand)
                })
            }
            RecordShape::External { variants } => {
                let with = variants.iter().find_map(|variant| {
                    let hand = hand.and_then(|h| h.get(&variant.key));
                    let payload = self.value(variant.payload.as_ref()?, path, hand)?;
                    Some(json!({ variant.key.clone(): payload }))
                });
                with.or_else(|| variants.first().map(|v| Value::String(v.key.clone())))
            }
            RecordShape::OrNamed { values, .. } => values
                .first()
                .and_then(|value| self.value(&value.ty, path, hand)),
        };
        path.pop();
        value
    }

    /// The fields, each written where it can be; `None` where a field
    /// serde requires cannot be.
    fn fields(
        &self,
        fields: &[RecordField],
        path: &mut Vec<String>,
        tag: Option<(&str, &str)>,
        hand: Option<&Value>,
    ) -> Option<Value> {
        let mut object = Map::new();
        if let Some((tag, key)) = tag {
            object.insert(tag.to_owned(), json!(key));
        }
        let mut seen = BTreeSet::new();
        for field in fields {
            if !seen.insert(field.name.as_str()) {
                continue;
            }
            match self.value(&field.ty, path, hand.and_then(|h| h.get(&field.name))) {
                Some(value) => {
                    object.insert(field.name.clone(), value);
                }
                None if field.nullable => {}
                None => return None,
            }
        }
        Some(Value::Object(object))
    }

    fn value(
        &self,
        ty: &RecordType,
        path: &mut Vec<String>,
        hand: Option<&Value>,
    ) -> Option<Value> {
        let first = hand.and_then(|h| h.get(0));
        Some(match ty {
            RecordType::Bool => Value::Bool(true),
            RecordType::Integer { .. } => json!(1),
            RecordType::Number => json!(1.5),
            RecordType::Text => json!("x"),
            // A raw part stays JSON in every binding, so there is no type
            // of its to read it into: the hand example's, if it has one.
            RecordType::Json => return hand.cloned(),
            RecordType::Record { name } => self.record(name, path, hand)?,
            RecordType::Member { catalogue } | RecordType::Keyed { catalogue } => {
                self.member(catalogue)?
            }
            RecordType::List { of, len: Some(len) } => {
                Value::Array(vec![self.value(of, path, first)?; *len as usize])
            }
            // A list of a union of several records holds one of each, so
            // every record of it is written and read back.
            RecordType::List { of, len: None } => match self.several(of) {
                Some(values) => Value::Array(
                    values
                        .iter()
                        .filter_map(|value| self.value(&value.ty, path, None))
                        .collect(),
                ),
                None => Value::Array(self.value(of, path, first).into_iter().collect()),
            },
            RecordType::Pair {
                first: a,
                second: b,
            } => {
                let second = hand.and_then(|h| h.get(1));
                json!([self.value(a, path, first)?, self.value(b, path, second)?])
            }
            RecordType::Map { of } => {
                let entry = hand
                    .and_then(Value::as_object)
                    .and_then(|h| h.iter().next());
                let key = entry.map_or("x", |(key, _)| key.as_str()).to_owned();
                let value = self.value(of, path, entry.map(|(_, v)| v));
                Value::Object(value.map(|v| (key, v)).into_iter().collect())
            }
        })
    }
}
