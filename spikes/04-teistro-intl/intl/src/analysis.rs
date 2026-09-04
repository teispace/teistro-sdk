//! What a message declares: its parameters with the types its functions
//! imply, the contexts it selects on, the markup it uses, the messages it
//! links and the entities it names. The validator compares signatures
//! across locales; the accessor generators turn the base locale's into
//! typed surfaces.

use std::collections::{BTreeMap, BTreeSet};

use crate::mf2::ast::{Body, Declaration, Expression, Key, Message, Operand, OptValue, Part};
use crate::source::Meta;

/// The type a parameter takes, inferred from the function applied to it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParamType {
    /// Text.
    String,
    /// A value of a declared context (`gender`), a closed set.
    Context(String),
    /// An integer.
    Integer,
    /// A number.
    Number,
    /// A catalogue key, optionally of one kind (`graha`).
    Entity(Option<String>),
    /// A list of values.
    List,
}

impl ParamType {
    /// Whether a translation's use of a parameter agrees with the base's.
    #[must_use]
    pub fn agrees_with(&self, base: &ParamType) -> bool {
        match (self, base) {
            (ParamType::String, ParamType::Context(_))
            | (ParamType::Context(_), ParamType::String)
            | (ParamType::Integer, ParamType::Number)
            | (ParamType::Number, ParamType::Integer) => true,
            (ParamType::Entity(a), ParamType::Entity(b)) => a.is_none() || b.is_none() || a == b,
            (a, b) => a == b,
        }
    }
}

/// What a message declares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Signature {
    /// The parameters by name.
    pub params: BTreeMap<String, ParamType>,
    /// The selectors of the matcher, in position order.
    pub selectors: Vec<Selector>,
    /// Markup tag names.
    pub markup: BTreeSet<String>,
    /// Keys named by `:msg`.
    pub links: Vec<String>,
    /// Catalogue keys named as literals by `:entity`.
    pub entities: Vec<String>,
}

/// One selector of a matcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selector {
    /// The variable selected on.
    pub variable: String,
    /// Its type.
    pub kind: ParamType,
    /// Whether a number selects ordinal categories.
    pub ordinal: bool,
    /// The literal keys used across the variants, deduplicated in order.
    pub keys: Vec<String>,
}

/// The signature of a message under a locale's metadata (for its
/// contexts).
#[must_use]
pub fn signature(message: &Message, meta: &Meta) -> Signature {
    let mut collector = Collector {
        meta,
        sig: Signature::default(),
        locals: message
            .declarations()
            .iter()
            .filter_map(|d| match d {
                Declaration::Local { variable, .. } => Some(variable.clone()),
                Declaration::Input { .. } => None,
            })
            .collect(),
        types: BTreeMap::new(),
    };
    for declaration in message.declarations() {
        let (target, expression) = match declaration {
            Declaration::Input {
                variable,
                expression,
            }
            | Declaration::Local {
                variable,
                expression,
            } => (variable.as_str(), expression),
        };
        let (kind, ordinal) = collector.expression_type(expression, target);
        if let Some(Operand::Variable(operand)) = &expression.operand {
            if !collector.locals.contains(operand) {
                collector.note(operand, kind.clone(), ordinal);
            }
        }
        collector.types.insert(target.to_string(), (kind, ordinal));
        collector.collect_names(expression);
    }
    for pattern in message.patterns() {
        for part in &pattern.0 {
            match part {
                Part::Expression(expression) => {
                    if let Some(Operand::Variable(name)) = &expression.operand {
                        let (kind, ordinal) = collector.expression_type(expression, name);
                        collector.note(name, kind, ordinal);
                    }
                    collector.collect_names(expression);
                }
                Part::Markup(markup) => {
                    collector.sig.markup.insert(markup.name.to_string());
                    for option in &markup.options {
                        if let OptValue::Variable(name) = &option.value {
                            collector.note(name, ParamType::String, false);
                        }
                    }
                }
                Part::Text(_) => {}
            }
        }
    }
    if let Message::Complex(complex) = message {
        if let Body::Matcher(matcher) = &complex.body {
            for (index, variable) in matcher.selectors.iter().enumerate() {
                collector.selector(message, matcher, index, variable);
            }
        }
    }
    collector.sig
}

struct Collector<'m> {
    meta: &'m Meta,
    sig: Signature,
    locals: BTreeSet<String>,
    types: BTreeMap<String, (ParamType, bool)>,
}

impl Collector<'_> {
    /// Records a variable's type; a parameter keeps the first type that is
    /// more specific than text.
    fn note(&mut self, name: &str, kind: ParamType, ordinal: bool) {
        self.types
            .entry(name.to_string())
            .or_insert((kind.clone(), ordinal));
        if !self.locals.contains(name) {
            let slot = self
                .sig
                .params
                .entry(name.to_string())
                .or_insert(ParamType::String);
            if *slot == ParamType::String {
                *slot = kind;
            }
        }
    }

    fn expression_type(&self, expression: &Expression, variable: &str) -> (ParamType, bool) {
        let Some(function) = &expression.function else {
            return self
                .types
                .get(variable)
                .cloned()
                .or_else(|| match &expression.operand {
                    Some(Operand::Variable(operand)) => self.types.get(operand).cloned(),
                    _ => None,
                })
                .unwrap_or((ParamType::String, false));
        };
        let ordinal = function.option("select") == Some("ordinal");
        let kind = match function.name.to_string().as_str() {
            "integer" => ParamType::Integer,
            "number" | "dms" | "zodiac" => ParamType::Number,
            "entity" => ParamType::Entity(function.option("kind").map(str::to_string)),
            "list" => ParamType::List,
            "string" if self.meta.contexts.contains_key(variable) => {
                ParamType::Context(variable.to_string())
            }
            _ => ParamType::String,
        };
        (kind, ordinal)
    }

    fn collect_names(&mut self, expression: &Expression) {
        let Some(function) = &expression.function else {
            return;
        };
        for option in &function.options {
            if let OptValue::Variable(name) = &option.value {
                self.note(name, ParamType::String, false);
            }
        }
        if let Some(Operand::Literal(literal)) = &expression.operand {
            if function.name.is("msg") {
                self.sig.links.push(literal.value.clone());
            } else if function.name.is("entity") {
                self.sig.entities.push(literal.value.clone());
            }
        }
    }

    fn selector(
        &mut self,
        message: &Message,
        matcher: &crate::mf2::ast::Matcher,
        index: usize,
        variable: &str,
    ) {
        let (kind, ordinal) = self
            .types
            .get(variable)
            .cloned()
            .unwrap_or((ParamType::String, false));
        let mut keys: Vec<String> = Vec::new();
        for variant in &matcher.variants {
            if let Some(Key::Literal(literal)) = variant.keys.get(index) {
                if !keys.contains(&literal.value) {
                    keys.push(literal.value.clone());
                }
            }
        }
        // A `:string` selector whose keys sit inside a declared context, or
        // whose name is one, is that context.
        let kind = match kind {
            ParamType::String => self
                .context_of(variable, &keys)
                .map_or(ParamType::String, ParamType::Context),
            other => other,
        };
        if let ParamType::Context(context) = &kind {
            if let Some(source) = root_variable(message, variable) {
                self.sig
                    .params
                    .insert(source, ParamType::Context(context.clone()));
            }
        }
        self.sig.selectors.push(Selector {
            variable: variable.to_string(),
            kind,
            ordinal,
            keys,
        });
    }

    fn context_of(&self, variable: &str, keys: &[String]) -> Option<String> {
        if let Some(values) = self.meta.contexts.get(variable) {
            if keys.iter().all(|k| values.contains(k)) {
                return Some(variable.to_string());
            }
        }
        if keys.is_empty() {
            return None;
        }
        self.meta
            .contexts
            .iter()
            .find(|(_, values)| keys.iter().all(|k| values.contains(k)))
            .map(|(name, _)| name.clone())
    }
}

/// The parameter a selector variable comes from, through locals.
fn root_variable(message: &Message, variable: &str) -> Option<String> {
    let mut current = variable.to_string();
    for _ in 0..8 {
        let declaration = message.declarations().iter().find(|d| match d {
            Declaration::Input { variable, .. } | Declaration::Local { variable, .. } => {
                *variable == current
            }
        });
        match declaration {
            Some(Declaration::Input { .. }) | None => return Some(current),
            Some(Declaration::Local { expression, .. }) => match &expression.operand {
                Some(Operand::Variable(operand)) => current.clone_from(operand),
                _ => return None,
            },
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::mf2::parse;

    fn meta() -> Meta {
        let mut meta: Meta =
            serde_json::from_str(r#"{"locale":"en-Latn"}"#).unwrap_or_else(|e| panic!("{e}"));
        meta.contexts
            .insert("gender".into(), vec!["m".into(), "f".into(), "n".into()]);
        meta
    }

    fn sig(source: &str) -> Signature {
        signature(&parse(source).unwrap_or_else(|e| panic!("{e}")), &meta())
    }

    #[test]
    fn parameters_take_the_types_their_functions_imply() {
        let s = sig(
            ".input {$graha :entity kind=graha} .input {$bhava :integer select=ordinal} .match $bhava one {{{$graha} {$bhava}st {$name}}} * {{{$graha}}}",
        );
        assert_eq!(
            s.params.get("graha"),
            Some(&ParamType::Entity(Some("graha".into())))
        );
        assert_eq!(s.params.get("bhava"), Some(&ParamType::Integer));
        assert_eq!(s.params.get("name"), Some(&ParamType::String));
        assert_eq!(s.selectors.len(), 1);
        assert!(s.selectors[0].ordinal);
        assert_eq!(s.selectors[0].keys, ["one"]);
    }

    #[test]
    fn contexts_are_recognised_by_name_and_by_keys() {
        let by_name = sig(".input {$gender :string} .match $gender * {{x}}");
        assert_eq!(
            by_name.params.get("gender"),
            Some(&ParamType::Context("gender".into()))
        );
        let by_keys = sig(".input {$g :string} .match $g f {{a}} * {{b}}");
        assert_eq!(
            by_keys.params.get("g"),
            Some(&ParamType::Context("gender".into()))
        );
        let plain = sig(".input {$g :string} .match $g x {{a}} * {{b}}");
        assert_eq!(plain.params.get("g"), Some(&ParamType::String));
    }

    #[test]
    fn locals_links_entities_and_markup_are_collected() {
        let s = sig(
            ".local $g = {$graha :entity} .match $g f {{{#b}{$g}{/b} {sdk.x.y :msg} {graha.SUN :entity}}} * {{z}}",
        );
        assert_eq!(s.params.keys().collect::<Vec<_>>(), ["graha"]);
        assert_eq!(s.params.get("graha"), Some(&ParamType::Entity(None)));
        assert_eq!(s.links, ["sdk.x.y"]);
        assert_eq!(s.entities, ["graha.SUN"]);
        assert!(s.markup.contains("b"));
        assert_eq!(s.selectors[0].kind, ParamType::Entity(None));
    }
}
