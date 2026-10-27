//! Dart: a test that builds each deep example ([`super::deep_examples`])
//! with the generated classes and holds what each writes to the example,
//! so a catalogue member below the top level that a Dart class writes as
//! anything but its key fails `check-dart`.
//!
//! Dart has no reader of a request, so the test cannot start from the JSON
//! as Java's does: this reads each example beside its schema and writes
//! the Dart that builds it, a member as its enum's member, a union as the
//! variant's class. A record the layer keeps by hand (`DART_FACADES`) has
//! constructors of its own, which this does not write: where one is
//! optional it is left out, and its place is dropped from both sides of
//! the comparison; where one is required, the record holding it is not
//! built, and each generated record below it is, held to its own place.

use std::collections::BTreeSet;
use std::fmt::Write;

use serde_json::{Map, Value};

use super::dart::{DART_SHADOWED, is_facade, member, quoted};
use super::{Types, key_part, variant_name};
use crate::emit::dart::{doc, identifier, preamble};
use crate::model::Api;
use crate::records::{RecordField, RecordShape, RecordType};

/// What the test says it is, after the header.
const ABOUT: &str = "Each deep example (`crates/sdk/tests/deep-requests.json`) built with
the generated request classes, a catalogue member as its enum's member,
and held to the example: the JSON a class writes must be the example's,
a member written as its key, full or bare, as serde reads either. A part
the layer builds by hand is left out where it is optional, and dropped
from both sides; where it is required, the generated records below it
are held instead.";

/// A step into a JSON value.
#[derive(Clone)]
enum Step {
    Key(String),
    Index(usize),
}

impl Step {
    fn dart(&self) -> String {
        match self {
            Step::Key(key) => quoted(key),
            Step::Index(index) => index.to_string(),
        }
    }
}

/// A path as a Dart list.
fn dart_path(path: &[Step]) -> String {
    let steps: Vec<String> = path.iter().map(Step::dart).collect();
    format!("[{}]", steps.join(", "))
}

/// A path as the test names it (`rules.bands[0]`).
fn named_path(path: &[Step]) -> String {
    let mut out = String::new();
    for step in path {
        match step {
            Step::Key(key) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(key);
            }
            Step::Index(index) => {
                let _ = write!(out, "[{index}]");
            }
        }
    }
    out
}

/// What building a value left out, by its path within the value built.
#[derive(Default)]
struct Left<'a, 'v> {
    /// The places dropped from both sides: a part not built, or a field
    /// the example leaves to serde that Dart writes as its default.
    dropped: Vec<Vec<Step>>,
    /// The parts not built, with their types, to look inside for
    /// generated records.
    unbuilt: Vec<(Vec<Step>, &'a RecordType, &'v Value)>,
}

impl Left<'_, '_> {
    fn mark(&self) -> (usize, usize) {
        (self.dropped.len(), self.unbuilt.len())
    }

    fn reset(&mut self, (dropped, unbuilt): (usize, usize)) {
        self.dropped.truncate(dropped);
        self.unbuilt.truncate(unbuilt);
    }
}

/// A generated record the test builds and holds to its place.
struct Case {
    record: String,
    path: Vec<Step>,
    class: String,
    built: String,
    dropped: Vec<Vec<Step>>,
    members: usize,
}

struct Builder<'a> {
    api: &'a Api,
    types: Types<'a>,
    /// The catalogue's kinds, the prefix of a member's full key.
    kinds: BTreeSet<&'a str>,
}

/// The longest run of items laid out on one line.
const INLINE: usize = 64;

/// Items on one line where they fit, otherwise one to a line, each closed
/// by a comma.
fn lines(open: &str, items: &[String], close: &str, depth: usize) -> String {
    let inline = items.join(", ");
    if !inline.contains('\n') && inline.len() <= INLINE {
        return format!("{open}{inline}{close}");
    }
    let inner = "  ".repeat(depth + 1);
    let mut out = format!("{open}\n");
    for item in items {
        let _ = writeln!(out, "{inner}{item},");
    }
    out.push_str(&"  ".repeat(depth));
    out.push_str(close);
    out
}

/// A raw part as a Dart literal.
fn literal(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quoted(s),
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(literal).collect();
            format!("<Object?>[{}]", items.join(", "))
        }
        Value::Object(entries) => {
            let entries: Vec<String> = entries
                .iter()
                .map(|(key, item)| format!("{}: {}", quoted(key), literal(item)))
                .collect();
            format!("<String, Object?>{{{}}}", entries.join(", "))
        }
    }
}

/// The value at `path` within `value`, if there is one.
fn at<'v>(value: &'v Value, path: &[Step]) -> Option<&'v Value> {
    path.iter().try_fold(value, |within, step| match step {
        Step::Key(key) => within.get(key),
        Step::Index(index) => within.get(index),
    })
}

/// `value` without the place at `path`, where it has one.
fn drop_at(value: &mut Value, path: &[Step]) {
    let Some((last, within)) = path.split_last() else {
        return;
    };
    let parent = within.iter().try_fold(value, |inner, step| match step {
        Step::Key(key) => inner.get_mut(key),
        Step::Index(index) => inner.get_mut(index),
    });
    match (parent, last) {
        (Some(Value::Object(map)), Step::Key(key)) => {
            map.remove(key);
        }
        (Some(Value::Array(items)), Step::Index(index)) if *index < items.len() => {
            items.remove(*index);
        }
        _ => {}
    }
}

impl<'a> Builder<'a> {
    fn of(api: &'a Api) -> Builder<'a> {
        Builder {
            api,
            types: Types::of(api),
            kinds: api.enums.iter().filter_map(|e| e.kind.as_deref()).collect(),
        }
    }

    /// How many strings in `value` are a catalogue member's full key.
    fn members(&self, value: &Value) -> usize {
        match value {
            Value::String(s) => usize::from(
                s.split_once('.')
                    .is_some_and(|(kind, _)| self.kinds.contains(kind)),
            ),
            Value::Array(items) => items.iter().map(|item| self.members(item)).sum(),
            Value::Object(entries) => entries.values().map(|item| self.members(item)).sum(),
            _ => 0,
        }
    }

    /// Whether `name` is a class this file builds, with a `toJson`.
    fn generated(&self, name: &str) -> bool {
        !is_facade(name) && !DART_SHADOWED.contains(&name) && !self.types.is_key(name)
    }

    fn record_def(&self, name: &str) -> Option<&'a RecordShape> {
        self.api
            .requests
            .records
            .iter()
            .find(|record| record.name == name)
            .map(|record| &record.shape)
    }

    /// The Dart that builds `value`, a `ty`; `None` where it cannot be
    /// built. `path` is where it stands within the value being built.
    fn build<'v>(
        &self,
        ty: &'a RecordType,
        value: &'v Value,
        path: &mut Vec<Step>,
        left: &mut Left<'a, 'v>,
        depth: usize,
    ) -> Option<String> {
        match ty {
            RecordType::Bool => value.as_bool().map(|b| b.to_string()),
            RecordType::Integer { .. } => value
                .as_i64()
                .map(|n| n.to_string())
                .or_else(|| value.as_u64().map(|n| n.to_string())),
            RecordType::Number => {
                let text = value.as_number()?.to_string();
                Some(if text.contains(['.', 'e', 'E']) {
                    text
                } else {
                    format!("{text}.0")
                })
            }
            RecordType::Text => value.as_str().map(quoted),
            RecordType::Json => Some(literal(value)),
            RecordType::Member { catalogue } | RecordType::Keyed { catalogue } => {
                let key = value.as_str()?;
                let bare = key
                    .strip_prefix(catalogue.as_str())
                    .and_then(|rest| rest.strip_prefix('.'))
                    .unwrap_or(key);
                self.types.enum_member(Some(catalogue), None, bare)
            }
            RecordType::Record { name } => self.record(name, value, path, left, depth),
            RecordType::List { of, .. } => {
                let mut items = Vec::new();
                for (index, item) in value.as_array()?.iter().enumerate() {
                    path.push(Step::Index(index));
                    let built = self.build(of, item, path, left, depth + 1);
                    path.pop();
                    items.push(built?);
                }
                Some(lines("[", &items, "]", depth))
            }
            RecordType::Pair { first, second } => {
                let [a, b] = value.as_array()?.as_slice() else {
                    return None;
                };
                let mut items = Vec::new();
                for (index, (ty, item)) in [(first, a), (second, b)].into_iter().enumerate() {
                    path.push(Step::Index(index));
                    let built = self.build(ty, item, path, left, depth + 1);
                    path.pop();
                    items.push(built?);
                }
                Some(lines("(", &items, ")", depth))
            }
            RecordType::Map { of } => {
                let mut entries = Vec::new();
                for (key, item) in value.as_object()? {
                    path.push(Step::Key(key.clone()));
                    let built = self.build(of, item, path, left, depth + 1);
                    path.pop();
                    entries.push(format!("{}: {}", quoted(key), built?));
                }
                Some(lines("{", &entries, "}", depth))
            }
        }
    }

    fn record<'v>(
        &self,
        name: &str,
        value: &'v Value,
        path: &mut Vec<Step>,
        left: &mut Left<'a, 'v>,
        depth: usize,
    ) -> Option<String> {
        if DART_SHADOWED.contains(&name) {
            return Some(literal(value));
        }
        if let Some(boundary) = self.api.requests.enums.get(name) {
            return self
                .types
                .enum_member(None, Some(boundary), value.as_str()?);
        }
        if is_facade(name) && !self.types.is_key(name) {
            return None;
        }
        match self.record_def(name)? {
            RecordShape::Object { fields } => {
                self.object(name, fields, value.as_object()?, path, left, depth)
            }
            RecordShape::Keys { values } => {
                let key = value.as_str()?;
                values
                    .iter()
                    .any(|v| v.key == key)
                    .then(|| format!("{name}.{}", member(key)))
            }
            RecordShape::Tagged { tag, variants } => {
                let given = value.as_object()?;
                let key = given.get(tag)?.as_str()?;
                let variant = variants.iter().find(|v| v.key == key)?;
                let class = variant_name(name, variant);
                self.object(&class, &variant.fields, given, path, left, depth)
            }
            RecordShape::External { variants } => match value {
                Value::String(key) => variants
                    .iter()
                    .find(|v| v.key == *key && v.payload.is_none())
                    .map(|v| format!("{name}{}()", key_part(&v.key))),
                Value::Object(one) if one.len() == 1 => {
                    let (key, payload) = one.iter().next()?;
                    let variant = variants.iter().find(|v| v.key == *key)?;
                    path.push(Step::Key(key.clone()));
                    let built = self.build(variant.payload.as_ref()?, payload, path, left, depth);
                    path.pop();
                    Some(format!("{name}{}({})", key_part(key), built?))
                }
                _ => None,
            },
            RecordShape::OrNamed { names, of, .. } => match value.as_str() {
                Some(key) if names.iter().any(|named| named == key) => {
                    Some(format!("{name}{}()", key_part(key)))
                }
                _ => {
                    let built = self.build(of, value, path, left, depth)?;
                    Some(format!("{name}Value({built})"))
                }
            },
        }
    }

    /// A class's constructor call with every field the example gives.
    fn object<'v>(
        &self,
        class: &str,
        fields: &'a [RecordField],
        given: &'v Map<String, Value>,
        path: &mut Vec<Step>,
        left: &mut Left<'a, 'v>,
        depth: usize,
    ) -> Option<String> {
        let mut named = Vec::new();
        let mut seen = BTreeSet::new();
        for field in fields {
            if !seen.insert(field.name.as_str()) {
                continue;
            }
            path.push(Step::Key(field.name.clone()));
            let built = match given.get(&field.name) {
                Some(item) if !item.is_null() => {
                    let mark = left.mark();
                    match self.build(&field.ty, item, path, left, depth + 1) {
                        Some(built) => Some(Some(built)),
                        None if field.nullable => {
                            left.reset(mark);
                            left.dropped.push(path.clone());
                            left.unbuilt.push((path.clone(), &field.ty, item));
                            Some(None)
                        }
                        None => None,
                    }
                }
                _ if field.nullable => {
                    // The example leaves the field to serde, where Dart
                    // writes a default the schema states. A raw part not
                    // given is left off, as the example leaves it.
                    if field
                        .default
                        .as_ref()
                        .and_then(|default| self.types.constant(&field.ty, default))
                        .is_some()
                    {
                        left.dropped.push(path.clone());
                    }
                    Some(None)
                }
                _ => None,
            };
            path.pop();
            if let Some(built) = built? {
                named.push(format!("{}: {built}", identifier(&field.name)));
            }
        }
        Some(lines(&format!("{class}("), &named, ")", depth))
    }

    /// The cases `value`, a `ty` at `path` of `record`'s example, gives:
    /// itself where it is a generated record that can be built, otherwise
    /// each generated record within it that can.
    fn cases(
        &self,
        record: &str,
        ty: &'a RecordType,
        value: &Value,
        path: &mut Vec<Step>,
        whole: &Value,
        out: &mut Vec<Case>,
    ) {
        if let RecordType::Record { name } = ty
            && self.generated(name)
        {
            let mut left = Left::default();
            if let Some(built) = self.build(ty, value, &mut Vec::new(), &mut left, 3) {
                let mut expected = at(whole, path).cloned().unwrap_or(Value::Null);
                for dropped in &left.dropped {
                    drop_at(&mut expected, dropped);
                }
                out.push(Case {
                    record: record.to_owned(),
                    path: path.clone(),
                    class: name.clone(),
                    built,
                    members: self.members(&expected),
                    dropped: left.dropped,
                });
                for (within, ty, value) in left.unbuilt {
                    let depth = path.len();
                    path.extend(within);
                    self.cases(record, ty, value, path, whole, out);
                    path.truncate(depth);
                }
                return;
            }
        }
        let mut inside = |ty: &'a RecordType, step: Step, value: &Value, out: &mut Vec<Case>| {
            path.push(step);
            self.cases(record, ty, value, path, whole, out);
            path.pop();
        };
        match ty {
            RecordType::Record { name } => match self.record_def(name) {
                Some(RecordShape::Object { fields }) => {
                    for field in fields {
                        if let Some(item) = value.get(&field.name) {
                            inside(&field.ty, Step::Key(field.name.clone()), item, out);
                        }
                    }
                }
                Some(RecordShape::Tagged { tag, variants }) => {
                    let key = value.get(tag).and_then(Value::as_str);
                    if let Some(variant) = variants.iter().find(|v| Some(v.key.as_str()) == key) {
                        for field in &variant.fields {
                            if let Some(item) = value.get(&field.name) {
                                inside(&field.ty, Step::Key(field.name.clone()), item, out);
                            }
                        }
                    }
                }
                Some(RecordShape::External { variants }) => {
                    for variant in variants {
                        if let (Some(payload), Some(item)) =
                            (&variant.payload, value.get(&variant.key))
                        {
                            inside(payload, Step::Key(variant.key.clone()), item, out);
                        }
                    }
                }
                Some(RecordShape::OrNamed { of, .. }) if !value.is_string() => {
                    self.cases(record, of, value, path, whole, out);
                }
                _ => {}
            },
            RecordType::List { of, .. } => {
                for (index, item) in value.as_array().into_iter().flatten().enumerate() {
                    inside(of, Step::Index(index), item, out);
                }
            }
            RecordType::Pair { first, second } => {
                for (index, ty) in [first, second].into_iter().enumerate() {
                    if let Some(item) = value.get(index) {
                        inside(ty, Step::Index(index), item, out);
                    }
                }
            }
            RecordType::Map { of } => {
                for (key, item) in value.as_object().into_iter().flatten() {
                    inside(of, Step::Key(key.clone()), item, out);
                }
            }
            _ => {}
        }
    }
}

/// The helpers the test calls, after its cases.
const HELPERS: &str = r"
/// The deep examples, found by walking up from the directory the suite
/// runs in.
final String _examples = () {
  for (var dir = Directory.current.absolute; ; dir = dir.parent) {
    final file = File('${dir.path}/crates/sdk/tests/deep-requests.json');
    if (file.existsSync()) return file.readAsStringSync();
    if (dir.parent.path == dir.path) {
      throw StateError('no crates/sdk/tests/deep-requests.json above ${Directory.current.path}');
    }
  }
}();

/// The catalogue's kinds, the prefix of a member's full key.
final Set<String> _kinds = {for (final kind in Kind.values) kind.key};

/// Holds [written], a request's JSON, to [record]'s example at [path],
/// with each place in [dropping] left out of both; [members] is how many
/// catalogue members the example holds there.
void _holds(
  String record, {
  required List<Object> path,
  required List<List<Object>> dropping,
  required int members,
  required Object? written,
}) {
  final examples = (jsonDecode(_examples) as List<Object?>).cast<Map<String, Object?>>();
  Object? expected = examples.singleWhere((one) => one['record'] == record)['request'];
  for (final step in path) {
    expected = _at(expected, step);
  }
  final Object? actual = jsonDecode(jsonEncode(written));
  for (final place in dropping) {
    _drop(expected, place);
    _drop(actual, place);
  }
  final compared = <String>[];
  _same(actual, expected, record, compared);
  expect(compared, hasLength(members), reason: '$record: the members compared');
}

Object? _at(Object? value, Object step) => switch ((value, step)) {
  (final Map<String, Object?> map, final String key) => map[key],
  (final List<Object?> list, final int index) when index < list.length => list[index],
  _ => null,
};

void _drop(Object? value, List<Object> place) {
  var within = value;
  for (final step in place.take(place.length - 1)) {
    within = _at(within, step);
  }
  switch ((within, place.last)) {
    case (final Map<String, Object?> map, final String key):
      map.remove(key);
    case (final List<Object?> list, final int index) when index < list.length:
      list.removeAt(index);
  }
}

/// Whether [actual] is [expected], adding each member compared to
/// [members]. A member crosses as its key, full or bare, as serde reads
/// either; an integer must stay one, as serde reads no `1.0` as an
/// integer.
void _same(Object? actual, Object? expected, String at, List<String> members) {
  switch (expected) {
    case final Map<String, Object?> map:
      if (actual is! Map<String, Object?>) fail('$at: $actual is not an object');
      expect(actual.keys.toSet(), map.keys.toSet(), reason: '$at: the fields');
      for (final MapEntry(:key, :value) in map.entries) {
        _same(actual[key], value, '$at.$key', members);
      }
    case final List<Object?> list:
      if (actual is! List<Object?>) fail('$at: $actual is not a list');
      expect(actual, hasLength(list.length), reason: '$at: the items');
      for (var i = 0; i < list.length; i += 1) {
        _same(actual[i], list[i], '$at[$i]', members);
      }
    case final String key when key.contains('.') && _kinds.contains(key.split('.').first):
      members.add(key);
      expect(actual, anyOf(key, key.substring(key.indexOf('.') + 1)), reason: '$at: a member');
    case final int number:
      expect(actual is int && actual == number, isTrue, reason: '$at: $actual for $number');
    case final double number:
      expect(actual is double && actual == number, isTrue, reason: '$at: $actual for $number');
    default:
      expect(actual, expected, reason: at);
  }
}
";

/// `deep_requests_test.dart`: every deep example built with the generated
/// Dart request classes and held to the example; `deep` is
/// [`super::deep_examples`]'s value.
#[must_use]
pub fn dart_deep_test(api: &Api, deep: &Value) -> String {
    // Each root's type is made here, and the builder borrows it as it
    // borrows the description.
    let roots: Vec<(&str, RecordType)> = api
        .requests
        .roots
        .iter()
        .map(|root| {
            let ty = RecordType::Record {
                name: root.name.clone(),
            };
            (root.record.as_str(), ty)
        })
        .collect();
    let builder = Builder::of(api);
    let mut cases = Vec::new();
    for (record, ty) in &roots {
        let example = deep
            .as_array()
            .and_then(|all| all.iter().find(|one| one["record"] == *record))
            .map(|one| &one["request"]);
        if let Some(example) = example {
            builder.cases(record, ty, example, &mut Vec::new(), example, &mut cases);
        }
    }
    let held: usize = cases.iter().map(|case| case.members).sum();
    let all: usize = deep
        .as_array()
        .into_iter()
        .flatten()
        .map(|one| builder.members(&one["request"]))
        .sum();
    let mut out = preamble(api, "");
    out.push_str(&doc(
        &format!("{ABOUT}\n\nThe cases compare {held} of the examples' {all} members."),
        "",
    ));
    out.push_str("library;\n\nimport 'dart:convert';\nimport 'dart:io';\n\nimport 'package:teistro/teistro.dart';\nimport 'package:test/test.dart';\n\nvoid main() {\n");
    for case in &cases {
        let place = if case.path.is_empty() {
            String::new()
        } else {
            format!(" at {}", named_path(&case.path))
        };
        let dropping: Vec<String> = case.dropped.iter().map(|p| dart_path(p)).collect();
        let _ = writeln!(
            out,
            "  test({}, () {{\n    _holds(\n      {},\n      path: {},\n      dropping: [{}],\n      members: {},\n      written: {}.toJson(),\n    );\n  }});\n",
            quoted(&format!("{}{place}: {}", case.record, case.class)),
            quoted(&case.record),
            dart_path(&case.path),
            dropping.join(", "),
            case.members,
            case.built
        );
    }
    out.push_str("}\n");
    out.push_str(HELPERS);
    out
}
