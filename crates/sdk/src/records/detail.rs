//! A record answered brief or full (`03-design/result-detail.md`): the
//! detail a call asks for, and what each record's brief answer keeps of
//! its full one, declared beside the record as an explicit list of fields.
//!
//! A brief answer is a projection of the full one: every field it keeps
//! is the full answer's own value, unchanged, and nothing is added. So a
//! caller holding a full answer derives the brief one ([`Brief::of`]), and
//! the answer key, which names the request, the settings and the engine,
//! identifies the full answer whichever detail was written.

use serde_json::{Map, Value};
use teistro_core::error::Error;

/// How much of its answer a record call asks for.
///
/// ```
/// use teistro::records::Detail;
///
/// assert_eq!(Detail::default(), Detail::Full);
/// assert_eq!(Detail::NAMES.map(Detail::from_name), [Some(Detail::Brief), Some(Detail::Full)]);
/// assert_eq!(Detail::Brief.name(), "brief");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Detail {
    /// The fields a reader needs, as the record declares them
    /// ([`Brief`]), and of the provenance [`Brief::PROVENANCE`].
    Brief,
    /// The whole answer, as serde writes it.
    #[default]
    Full,
}

impl Detail {
    /// The names [`Detail::from_name`] reads, as a call spells them.
    pub const NAMES: [&'static str; 2] = ["brief", "full"];

    /// The detail called `name`, in lower case.
    ///
    /// ```
    /// use teistro::records::Detail;
    ///
    /// assert_eq!(Detail::from_name("brief"), Some(Detail::Brief));
    /// assert_eq!(Detail::from_name("BRIEF"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Detail> {
        match name {
            "brief" => Some(Detail::Brief),
            "full" => Some(Detail::Full),
            _ => None,
        }
    }

    /// The detail called `name`, or a refusal naming the two there are.
    ///
    /// ```
    /// use teistro::records::Detail;
    ///
    /// assert_eq!(Detail::read("full").unwrap(), Detail::Full);
    /// let refused = Detail::read("short").unwrap_err();
    /// assert_eq!(refused.status, teistro::Status::InvalidArg);
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for any other name, its hint naming both.
    pub fn read(name: &str) -> Result<Detail, Error> {
        Detail::from_name(name).ok_or_else(|| {
            Error::invalid_arg(format!("no detail `{name}`"))
                .with_hint(format!("a detail is {}", Detail::NAMES.join(" or ")))
        })
    }

    /// Its name, as [`Detail::from_name`] reads it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Detail::Brief => "brief",
            Detail::Full => "full",
        }
    }
}

/// What a record's brief answer keeps of its full one, declared beside
/// the record and never inferred from the answer.
///
/// ```
/// use serde_json::json;
/// use teistro::records::Brief;
///
/// let brief = Brief::Keeps(&["date", "days.tithi"]);
/// let full = json!({"date": 1, "hash": "ab", "days": [{"tithi": 3, "yoga": 4}]});
/// assert_eq!(brief.of(full), json!({"date": 1, "days": [{"tithi": 3}]}));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brief {
    /// The full value: nothing in it is beside what a reader needs. The
    /// provenance is still the brief one.
    Whole,
    /// These fields, each a path of property names joined by `.`. A path
    /// reaching a list keeps the field in each of its items, as one
    /// reaching an object keeps it there; an answer that is a list keeps
    /// the paths in each item. A field the full answer leaves out is left
    /// out.
    Keeps(&'static [&'static str]),
}

impl Brief {
    /// What a brief answer keeps of a provenance: the profile, the
    /// confidence, the fallbacks and warnings a reader must not miss,
    /// and the answer key, which identifies the full answer.
    pub const PROVENANCE: Brief = Brief::Keeps(&[
        "profile",
        "confidence",
        "fallbacks_used",
        "warnings",
        "answer_key",
    ]);

    /// The paths it keeps; none for [`Brief::Whole`], which keeps all.
    #[must_use]
    pub const fn paths(self) -> &'static [&'static str] {
        match self {
            Brief::Whole => &[],
            Brief::Keeps(paths) => paths,
        }
    }

    /// The brief of `full`: the value itself under [`Brief::Whole`], and
    /// under [`Brief::Keeps`] each kept field's own value, unchanged.
    #[must_use]
    pub fn of(self, full: Value) -> Value {
        match self {
            Brief::Whole => full,
            Brief::Keeps(paths) => kept(full, paths),
        }
    }
}

/// A caller's own field mask over a record's full answer: the paths they
/// chose, kept as [`Brief::Keeps`] keeps a brief's, but named at call
/// time (`03-design/result-detail.md` D6). Made only by
/// [`Record::fields`](crate::records::Record::fields), which checks every
/// path against the record's answer schema, so a mask in hand names no
/// field its record lacks. A mask selects from the full answer, never
/// from a brief one (crux C1260).
///
/// ```
/// # #[cfg(feature = "schema")] {
/// let convert = teistro::records::record("calendar.convert").expect("every build carries it");
/// let mask = convert.fields(["date.year", "weekday"]).expect("both are the answer's");
/// assert_eq!(mask.record(), "calendar.convert");
/// assert_eq!(mask.paths(), ["date.year", "weekday"]);
/// let full = serde_json::json!({"date": {"year": 2083, "month": 6}, "weekday": "SATURDAY", "fixed": 1});
/// assert_eq!(mask.of(full), serde_json::json!({"date": {"year": 2083}, "weekday": "SATURDAY"}));
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fields {
    record: &'static str,
    paths: Vec<String>,
}

impl Fields {
    /// The record whose answer the mask was checked against.
    #[must_use]
    pub const fn record(&self) -> &'static str {
        self.record
    }

    /// The paths it keeps, as the caller named them.
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    /// The fields of `full` it keeps, each the full answer's own value,
    /// unchanged; a field the full answer leaves out is left out.
    #[must_use]
    pub fn of(&self, full: Value) -> Value {
        let paths: Vec<&str> = self.paths.iter().map(String::as_str).collect();
        kept(full, &paths)
    }

    /// `paths` as a mask over the answer `schema` describes, whose
    /// definitions are `definitions`; `None` for a record that states no
    /// answer schema.
    #[cfg(feature = "schema")]
    pub(crate) fn checked(
        record: &'static str,
        paths: Vec<String>,
        schema: Option<(Value, Map<String, Value>)>,
    ) -> Result<Fields, Error> {
        if paths.is_empty() {
            return Err(Error::invalid_arg("a field mask names at least one field")
                .with_field("fields")
                .with_hint("leave the mask out for the whole answer"));
        }
        for (index, path) in paths.iter().enumerate() {
            if path.split('.').any(str::is_empty) {
                return Err(Error::invalid_arg(format!(
                    "`{path}` is no field path; a path is property names joined by `.`"
                ))
                .with_field(format!("fields[{index}]")));
            }
        }
        let Some((mut schema, definitions)) = schema else {
            return Err(Error::unsupported(format!(
                "`{record}` states no answer schema, so no mask can be checked against it"
            ))
            .with_field("fields"));
        };
        let named: Vec<&str> = paths.iter().map(String::as_str).collect();
        let found = narrowed_to(&named, &mut schema, &definitions);
        if let Some(unknown) = found.unknown.first() {
            let index = paths.iter().position(|path| path == unknown).unwrap_or(0);
            return Err(Error::invalid_arg(format!(
                "the answer of `{record}` has no field `{unknown}`"
            ))
            .with_field(format!("fields[{index}]"))
            .with_hint("a path names the answer's own properties, as its schema states them"));
        }
        Ok(Fields { record, paths })
    }
}

/// `value` with only `paths` kept, a list's items each kept alike.
fn kept(value: Value, paths: &[&str]) -> Value {
    match value {
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|item| kept(item, paths)).collect())
        }
        Value::Object(fields) => {
            let mut out = Map::new();
            for (key, inner) in fields {
                let mut whole = false;
                let mut tails = Vec::new();
                for path in paths {
                    match path.split_once('.') {
                        None if *path == key => whole = true,
                        Some((head, tail)) if head == key => tails.push(tail),
                        _ => {}
                    }
                }
                if whole {
                    out.insert(key, inner);
                } else if !tails.is_empty() {
                    out.insert(key, kept(inner, &tails));
                }
            }
            Value::Object(out)
        }
        scalar => scalar,
    }
}

/// What narrowing a schema to a brief found: the paths it names that the
/// schema has no property for, the paths that run into a value the schema
/// leaves open (a section held as any JSON), and each tag a `oneOf`
/// branch would lose, where an instance without it could match two
/// branches or none.
#[cfg(feature = "schema")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Narrowed {
    /// Paths the schema has no property for.
    pub unknown: Vec<String>,
    /// Paths that reach a value the schema states nothing of, so nothing
    /// below it is checked.
    pub open: Vec<String>,
    /// Tags of a `oneOf` branch the brief leaves out: a property the
    /// branch requires whose schema is a single constant.
    pub untagged: Vec<String>,
    /// Every property the schema states beside a kept path that the
    /// brief leaves out, as a path: what a brief answer never carries.
    pub left_out: Vec<String>,
}

#[cfg(feature = "schema")]
impl Brief {
    /// `schema`, written by a generator whose definitions are
    /// `definitions`, narrowed so that a brief answer holds to it: every
    /// object the brief's paths reach requires only what the brief keeps
    /// of what it required. Only `required` is loosened, so a full answer
    /// holds to it too. A definition a path reaches is inlined as a copy
    /// and narrowed there, so the definition itself, which other places
    /// share, is left whole.
    ///
    /// ```
    /// use serde_json::json;
    /// use teistro::records::Brief;
    ///
    /// let mut schema = json!({"$ref": "#/$defs/Day"});
    /// let definitions = json!({"Day": {
    ///     "type": "object",
    ///     "properties": {"date": {"type": "integer"}, "hash": {"type": "string"}, "extra": {}},
    ///     "required": ["date", "hash"],
    /// }});
    /// let found = Brief::Keeps(&["date", "nothing", "extra.deep"])
    ///     .narrow(&mut schema, definitions.as_object().unwrap());
    /// assert_eq!(schema["required"], json!(["date"]));
    /// assert_eq!(found.unknown, ["nothing"]);
    /// assert_eq!(found.open, ["extra.deep"]);
    /// assert_eq!(found.left_out, ["hash"]);
    /// ```
    #[must_use]
    pub fn narrow(self, schema: &mut Value, definitions: &Map<String, Value>) -> Narrowed {
        narrowed_to(self.paths(), schema, definitions)
    }
}

/// `schema` narrowed to `paths` in place, as [`Brief::narrow`] narrows it
/// to a brief's, with what narrowing found.
#[cfg(feature = "schema")]
fn narrowed_to(paths: &[&str], schema: &mut Value, definitions: &Map<String, Value>) -> Narrowed {
    let mut narrowed = Narrowed::default();
    if paths.is_empty() {
        return narrowed;
    }
    let mut walk = Walk {
        definitions,
        untagged: Vec::new(),
    };
    let reached = walk.narrow(schema, paths, false, 0);
    narrowed.unknown = paths
        .iter()
        .filter(|path| !reached.found.contains(**path) && !reached.open.contains(**path))
        .map(|path| (*path).to_owned())
        .collect();
    narrowed.open = paths
        .iter()
        .filter(|path| reached.open.contains(**path) && !reached.found.contains(**path))
        .map(|path| (*path).to_owned())
        .collect();
    narrowed.untagged = walk.untagged;
    narrowed.left_out = reached.left_out.into_iter().collect();
    narrowed
}

/// How deep a path may follow definitions into one another: far beyond
/// any brief's paths, and a stop for a schema that refers to itself.
#[cfg(feature = "schema")]
const DEPTH: usize = 64;

/// The keywords by which a schema states something of a value; one with
/// none of them admits any JSON.
#[cfg(feature = "schema")]
const STATING: [&str; 10] = [
    "type",
    "properties",
    "items",
    "anyOf",
    "oneOf",
    "allOf",
    "$ref",
    "const",
    "enum",
    "not",
];

/// The paths a narrowing reached, relative to where it began: those a
/// property was found for, and those that ran into an open value.
#[cfg(feature = "schema")]
#[derive(Default)]
struct Reached {
    found: std::collections::BTreeSet<String>,
    open: std::collections::BTreeSet<String>,
    left_out: std::collections::BTreeSet<String>,
}

#[cfg(feature = "schema")]
impl Reached {
    fn extend(&mut self, other: Reached) {
        self.found.extend(other.found);
        self.open.extend(other.open);
        self.left_out.extend(other.left_out);
    }

    fn under(&mut self, head: &str, other: Reached) {
        self.found
            .extend(other.found.into_iter().map(|tail| format!("{head}.{tail}")));
        self.open
            .extend(other.open.into_iter().map(|tail| format!("{head}.{tail}")));
        self.left_out.extend(
            other
                .left_out
                .into_iter()
                .map(|tail| format!("{head}.{tail}")),
        );
    }
}

#[cfg(feature = "schema")]
struct Walk<'a> {
    definitions: &'a Map<String, Value>,
    untagged: Vec<String>,
}

#[cfg(feature = "schema")]
impl Walk<'_> {
    /// `schema` narrowed to `paths` in place.
    fn narrow(
        &mut self,
        schema: &mut Value,
        paths: &[&str],
        in_one_of: bool,
        depth: usize,
    ) -> Reached {
        let mut reached = Reached::default();
        if depth > DEPTH {
            return reached;
        }
        let Some(object) = schema.as_object_mut() else {
            // `true` admits anything.
            if schema.as_bool() == Some(true) {
                reached
                    .open
                    .extend(paths.iter().map(|path| (*path).to_owned()));
            }
            return reached;
        };
        if !STATING.iter().any(|keyword| object.contains_key(*keyword)) {
            reached
                .open
                .extend(paths.iter().map(|path| (*path).to_owned()));
            return reached;
        }
        // A definition the paths reach is copied in place of its reference.
        if let Some(Value::String(reference)) = object.get("$ref")
            && let Some(name) = reference.rsplit('/').next()
            && let Some(Value::Object(definition)) = self.definitions.get(name)
        {
            object.remove("$ref");
            for (key, value) in definition {
                object.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        for key in ["items", "additionalProperties"] {
            if let Some(inner @ Value::Object(_)) = object.get_mut(key) {
                reached.extend(self.narrow(inner, paths, in_one_of, depth + 1));
            }
        }
        for key in ["anyOf", "allOf", "oneOf"] {
            if let Some(Value::Array(branches)) = object.get_mut(key) {
                let one_of = in_one_of || key == "oneOf";
                for branch in branches {
                    reached.extend(self.narrow(branch, paths, one_of, depth + 1));
                }
            }
        }
        let Some(Value::Object(properties)) = object.get_mut("properties") else {
            return reached;
        };
        let mut heads: Vec<&str> = Vec::new();
        for path in paths {
            let head = path.split_once('.').map_or(*path, |(head, _)| head);
            if !properties.contains_key(head) {
                continue;
            }
            if !heads.contains(&head) {
                heads.push(head);
            }
            if !path.contains('.') {
                reached.found.insert((*path).to_owned());
            }
        }
        reached.left_out.extend(
            properties
                .keys()
                .filter(|name| !heads.contains(&name.as_str()))
                .cloned(),
        );
        for head in &heads {
            let tails: Vec<&str> = paths
                .iter()
                .filter_map(|path| path.strip_prefix(*head)?.strip_prefix('.'))
                .collect();
            if paths.contains(head) || tails.is_empty() {
                continue;
            }
            if let Some(inner) = properties.get_mut(*head) {
                let deeper = self.narrow(inner, &tails, in_one_of, depth + 1);
                reached.under(head, deeper);
            }
        }
        // A tag a branch of a `oneOf` requires: a property whose schema is
        // one constant, which tells the branches apart.
        let tags: Vec<String> = properties
            .iter()
            .filter(|(_, schema)| {
                schema.get("const").is_some()
                    || schema
                        .get("enum")
                        .and_then(Value::as_array)
                        .is_some_and(|members| members.len() == 1)
            })
            .map(|(name, _)| name.clone())
            .collect();
        if let Some(Value::Array(required)) = object.get_mut("required") {
            required.retain(|name| {
                let Some(name) = name.as_str() else {
                    return false;
                };
                let kept = heads.contains(&name);
                if !kept && in_one_of && tags.iter().any(|tag| tag == name) {
                    self.untagged.push(name.to_owned());
                }
                kept
            });
        }
        reached
    }
}
