//! Evaluation: a message, its parameters, the locale's metadata and the
//! SDK's functions become text and parts, with the locale that answered
//! and every problem met on the way. Plural categories come from ICU4X;
//! nothing of CLDR is reimplemented here. The worst case renders the key
//! itself, never a blank.

use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};
use std::sync::{Arc, PoisonError, RwLock};

use fixed_decimal::Decimal;
use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRules};

use crate::mf2::ast::{
    Body, Declaration, Expression, Function, Key, Markup, MarkupKind, Matcher, Message, Operand,
    Opt, OptValue, Part, Pattern,
};
use crate::mf2::{ParseError, parse};
use teistro_core::catalogue::{Catalogued, Kind, Rashi};

use crate::source::{BASE_LOCALE, Entity, Entry, LocaleSource, Meta, Tree};

/// A parameter value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Text.
    Str(String),
    /// An integer.
    Int(i64),
    /// A number.
    Num(f64),
    /// A catalogue key (`graha.SUN`).
    Entity(String),
    /// A list of values.
    List(Vec<Value>),
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Str(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Str(s)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Value {
        Value::Int(i)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Value {
        Value::Num(n)
    }
}

impl From<Vec<Value>> for Value {
    fn from(list: Vec<Value>) -> Value {
        Value::List(list)
    }
}

impl Value {
    /// A catalogue key given as text (`graha.SUN`).
    #[must_use]
    pub fn entity(key: &str) -> Value {
        Value::Entity(key.to_string())
    }

    /// A catalogued member as an entity (`Graha::Sun` is `graha.SUN`): the
    /// typed way, which cannot name a key the catalogue lacks.
    #[must_use]
    pub fn catalogued<T: Catalogued>(member: T) -> Value {
        Value::Entity(member.full_key())
    }
}

/// A message with typed parameters: what the generated accessors
/// ([`crate::messages`], and a consumer's own from `teistro-intl gen
/// --target rs`) implement, so a key or a parameter cannot be misspelt.
pub trait TypedMessage {
    /// The full key.
    const KEY: &'static str;
    /// The parameters, by name.
    fn params(&self) -> Params;
}

/// The parameters of one render.
pub type Params = BTreeMap<String, Value>;

/// Parameters from pairs.
#[must_use]
pub fn params<const N: usize>(pairs: [(&str, Value); N]) -> Params {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_string(), value))
        .collect()
}

/// One rendered part: text, or markup for a rich renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutPart {
    /// Text.
    Text(String),
    /// A tag, with its options resolved to strings.
    Markup {
        /// Open, close or standalone.
        kind: MarkupKind,
        /// The tag name.
        name: String,
        /// The options.
        options: Vec<(String, String)>,
    },
}

/// What a render produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// The plain text: the text parts joined, markup stripped.
    pub text: String,
    /// The parts, for rich renderers.
    pub parts: Vec<OutPart>,
    /// The locale whose message answered, `None` when no locale had it.
    pub resolved_from: Option<String>,
    /// Whether the answer came from a fallback locale.
    pub is_fallback: bool,
    /// Every problem met: a missing parameter, an unknown function, a
    /// missing entity. Rendering continues past each.
    pub warnings: Vec<String>,
}

/// The key of the sign at an index of the ecliptic, from the catalogue
/// (`rashi.ARIES` for 0), for `:zodiac`.
#[must_use]
pub fn rashi_key(index: usize) -> String {
    let sign = Rashi::ALL.get(index % 12).copied().unwrap_or(Rashi::Aries);
    format!("{}.{}", Kind::Rashi.name(), sign.key())
}

/// How deep `:msg` may link before it gives up.
pub const MSG_DEPTH: u8 = 8;

/// The digits of a CLDR numbering system, for the systems the product
/// renders in.
#[must_use]
pub fn digits(system: &str) -> Option<[char; 10]> {
    let zero = match system {
        "latn" => '0',
        "deva" => '\u{966}',
        "beng" => '\u{9E6}',
        "gujr" => '\u{AE6}',
        "orya" => '\u{B66}',
        "taml" => '\u{BE6}',
        "tibt" => '\u{F20}',
        "arab" => '\u{660}',
        _ => return None,
    };
    let mut out = ['0'; 10];
    for (i, slot) in (0u32..).zip(out.iter_mut()) {
        *slot = char::from_u32(zero as u32 + i).unwrap_or('0');
    }
    Some(out)
}

/// How a locale writes numbers, from its metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberStyle {
    digits: [char; 10],
    decimal: char,
    group: char,
    grouping: Vec<u8>,
}

impl NumberStyle {
    /// The style of a locale; an unknown numbering system falls back to
    /// Latin digits.
    #[must_use]
    pub fn of(meta: &Meta) -> NumberStyle {
        NumberStyle {
            digits: digits(&meta.numbering_system)
                .unwrap_or_else(|| digits("latn").unwrap_or(['0'; 10])),
            decimal: meta.decimal,
            group: meta.group,
            grouping: if meta.grouping.is_empty() {
                vec![3]
            } else {
                meta.grouping.clone()
            },
        }
    }

    /// An ASCII decimal (`-1234567.5`) in the locale's digits, grouping
    /// and separators.
    #[must_use]
    pub fn localise(&self, ascii: &str) -> String {
        let (sign, digits) = match ascii.strip_prefix('-') {
            Some(rest) => ("-", rest),
            None => ("", ascii),
        };
        let (int, frac) = digits.split_once('.').unwrap_or((digits, ""));
        let mut grouped: Vec<char> = Vec::new();
        let mut group_index = 0usize;
        let mut in_group = 0u8;
        for c in int.chars().rev() {
            let size = self
                .grouping
                .get(group_index)
                .or(self.grouping.last())
                .copied()
                .unwrap_or(3)
                .max(1);
            if in_group == size {
                grouped.push(self.group);
                in_group = 0;
                group_index += 1;
            }
            grouped.push(self.digit(c));
            in_group += 1;
        }
        let mut out = String::from(sign);
        out.extend(grouped.iter().rev());
        if !frac.is_empty() {
            out.push(self.decimal);
            out.extend(frac.chars().map(|c| self.digit(c)));
        }
        out
    }

    /// An integer in the locale's digits and grouping.
    #[must_use]
    pub fn integer(&self, value: i64) -> String {
        self.localise(&value.to_string())
    }

    /// ASCII digits mapped, everything else kept.
    #[must_use]
    pub fn map_digits(&self, ascii: &str) -> String {
        ascii.chars().map(|c| self.digit(c)).collect()
    }

    fn digit(&self, c: char) -> char {
        c.to_digit(10)
            .and_then(|d| self.digits.get(d as usize).copied())
            .unwrap_or(c)
    }
}

/// A decimal string with `min` to `max` fraction digits, trailing zeros
/// beyond `min` removed, no exponent, `-0` normalised.
#[must_use]
pub fn ascii_decimal(value: f64, min: usize, max: usize) -> String {
    let max = max.max(min);
    let mut s = format!("{value:.max$}");
    if let Some(dot) = s.find('.') {
        let mut end = s.len();
        while end > dot + 1 + min && s.as_bytes().get(end - 1) == Some(&b'0') {
            end -= 1;
        }
        if end == dot + 1 {
            end = dot;
        }
        s.truncate(end);
    }
    if s.trim_start_matches('-')
        .bytes()
        .all(|b| b == b'0' || b == b'.')
    {
        s = s.trim_start_matches('-').to_string();
    }
    s
}

/// The engine over a set of locales.
pub struct Intl {
    locales: BTreeMap<String, LocaleSource>,
    plurals: BTreeMap<String, Plurals>,
    current: String,
    /// Messages parsed once, by locale and key; parsing costs four times a
    /// render, and a message never changes while the engine lives.
    parsed: RwLock<HashMap<(String, String), Arc<Message>>>,
}

impl fmt::Debug for Intl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Intl")
            .field("locales", &self.locales.keys().collect::<Vec<_>>())
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

struct Plurals {
    cardinal: PluralRules,
    ordinal: PluralRules,
}

/// A configuration failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntlError(pub String);

impl fmt::Display for IntlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for IntlError {}

fn plurals_for(tag: &str) -> Result<Plurals, IntlError> {
    let locale: Locale = tag
        .parse()
        .map_err(|e| IntlError(format!("{tag}: not a locale: {e}")))?;
    let cardinal = PluralRules::try_new_cardinal((&locale).into())
        .map_err(|e| IntlError(format!("{tag}: no cardinal plural rules: {e}")))?;
    let ordinal = PluralRules::try_new_ordinal((&locale).into())
        .map_err(|e| IntlError(format!("{tag}: no ordinal plural rules: {e}")))?;
    Ok(Plurals { cardinal, ordinal })
}

const fn category_name(category: PluralCategory) -> &'static str {
    match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}

impl Intl {
    /// An engine over locales; the current locale is the base locale when
    /// present, else the first.
    ///
    /// # Errors
    ///
    /// A locale tag ICU4X cannot parse or has no plural rules for.
    pub fn new(locales: BTreeMap<String, LocaleSource>) -> Result<Intl, IntlError> {
        let mut plurals = BTreeMap::new();
        for tag in locales.keys() {
            plurals.insert(tag.clone(), plurals_for(tag)?);
        }
        let current = locales
            .contains_key(BASE_LOCALE)
            .then(|| BASE_LOCALE.to_string())
            .or_else(|| locales.keys().next().cloned())
            .ok_or_else(|| IntlError(String::from("no locales")))?;
        Ok(Intl {
            locales,
            plurals,
            current,
            parsed: RwLock::new(HashMap::new()),
        })
    }

    /// An engine over a loaded source tree.
    ///
    /// # Errors
    ///
    /// As [`Intl::new`].
    pub fn from_tree(tree: &Tree) -> Result<Intl, IntlError> {
        Intl::new(tree.locales.clone())
    }

    /// Selects the locale every render resolves from. Explicit and
    /// deterministic: the engine never reads the environment.
    ///
    /// # Errors
    ///
    /// An unknown locale.
    pub fn set_locale(&mut self, tag: &str) -> Result<(), IntlError> {
        if !self.locales.contains_key(tag) {
            return Err(IntlError(format!("unknown locale {tag}")));
        }
        self.current = tag.to_string();
        Ok(())
    }

    /// The current locale.
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.current
    }

    /// The locales loaded, sorted.
    pub fn locales(&self) -> impl Iterator<Item = &LocaleSource> {
        self.locales.values()
    }

    /// The resolution chain from a locale: itself, then its declared
    /// fallbacks that are loaded.
    #[must_use]
    pub fn chain_from(&self, tag: &str) -> Vec<&LocaleSource> {
        let Some(first) = self.locales.get(tag) else {
            return Vec::new();
        };
        let mut chain = vec![first];
        chain.extend(
            first
                .meta
                .fallback
                .iter()
                .filter_map(|t| self.locales.get(t)),
        );
        chain
    }

    /// The entry for a key along the chain from `tag`, with the locale
    /// that has it.
    #[must_use]
    pub fn resolve_from<'a>(
        &'a self,
        tag: &str,
        key: &str,
    ) -> Option<(&'a LocaleSource, &'a Entry)> {
        self.chain_from(tag)
            .into_iter()
            .find_map(|locale| locale.entry(key).map(|entry| (locale, entry)))
    }

    /// The entity for a catalogue key along the chain from `tag`.
    #[must_use]
    pub fn entity_from<'a>(&'a self, tag: &str, key: &str) -> Option<&'a Entity> {
        self.chain_from(tag)
            .into_iter()
            .find_map(|locale| locale.entity(key))
    }

    /// Whether a key resolves under the current locale's chain.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.resolve_from(&self.current, key).is_some()
    }

    /// Renders a typed message ([`TypedMessage`]) under the current locale
    /// and its fallback chain.
    #[must_use]
    pub fn render_typed<M: TypedMessage>(&self, message: &M) -> Rendered {
        self.render(M::KEY, &message.params())
    }

    /// Renders a key with parameters in the current locale.
    #[must_use]
    pub fn render(&self, key: &str, params: &Params) -> Rendered {
        self.render_from(&self.current, key, params, 0)
    }

    /// Renders a message given as source, in the current locale, for
    /// tools and tests.
    ///
    /// # Errors
    ///
    /// The source does not parse.
    pub fn render_source(&self, source: &str, params: &Params) -> Result<Rendered, ParseError> {
        let message = parse(source)?;
        let locale = self.locales.get(&self.current).ok_or_else(|| ParseError {
            offset: None,
            message: String::from("no current locale"),
        })?;
        let mut eval = Eval::new(self, locale, params, 0);
        let parts = eval.message(&message);
        Ok(Rendered {
            text: plain_text(&parts),
            parts,
            resolved_from: Some(locale.tag.clone()),
            is_fallback: false,
            warnings: eval.warnings,
        })
    }

    fn render_from(&self, tag: &str, key: &str, params: &Params, depth: u8) -> Rendered {
        let Some((locale, entry)) = self.resolve_from(tag, key) else {
            return Rendered {
                text: key.to_string(),
                parts: vec![OutPart::Text(key.to_string())],
                resolved_from: None,
                is_fallback: false,
                warnings: vec![format!(
                    "missing message `{key}` in {tag} and its fallbacks"
                )],
            };
        };
        let is_fallback = locale.tag != tag;
        let mut warnings = Vec::new();
        let parts = match entry {
            Entry::Entity(entity) => vec![OutPart::Text(entity.name().to_string())],
            Entry::Message(source) => match self.parsed_message(&locale.tag, key, source) {
                Ok(message) => {
                    let mut eval = Eval::new(self, locale, params, depth);
                    let parts = eval.message(&message);
                    warnings.append(&mut eval.warnings);
                    parts
                }
                Err(error) => {
                    warnings.push(format!("`{key}` in {} does not parse: {error}", locale.tag));
                    vec![OutPart::Text(key.to_string())]
                }
            },
        };
        Rendered {
            text: plain_text(&parts),
            parts,
            resolved_from: Some(locale.tag.clone()),
            is_fallback,
            warnings,
        }
    }

    /// The plural categories a locale's rules can produce.
    #[must_use]
    pub fn categories(&self, tag: &str, ordinal: bool) -> Vec<&'static str> {
        let Some(rules) = self.plurals.get(tag) else {
            return Vec::new();
        };
        let rules = if ordinal {
            &rules.ordinal
        } else {
            &rules.cardinal
        };
        rules.categories().map(category_name).collect()
    }

    /// The parsed message for a key of a locale, from the cache or parsed
    /// now and cached.
    fn parsed_message(
        &self,
        tag: &str,
        key: &str,
        source: &str,
    ) -> Result<Arc<Message>, ParseError> {
        let id = (tag.to_string(), key.to_string());
        if let Some(message) = self
            .parsed
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
        {
            return Ok(Arc::clone(message));
        }
        let message = Arc::new(parse(source)?);
        self.parsed
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, Arc::clone(&message));
        Ok(message)
    }

    fn category(&self, tag: &str, decimal: &str, ordinal: bool) -> Option<&'static str> {
        let rules = self.plurals.get(tag)?;
        let rules = if ordinal {
            &rules.ordinal
        } else {
            &rules.cardinal
        };
        let decimal = Decimal::try_from_str(decimal).ok()?;
        Some(category_name(rules.category_for(&decimal)))
    }
}

/// The text parts joined.
#[must_use]
pub fn plain_text(parts: &[OutPart]) -> String {
    parts
        .iter()
        .filter_map(|p| match p {
            OutPart::Text(t) => Some(t.as_str()),
            OutPart::Markup { .. } => None,
        })
        .collect()
}

/// A value inside an evaluation.
#[derive(Clone, Debug)]
enum Val {
    /// A parameter or literal, not yet formatted.
    Raw(Value),
    /// A function's result.
    Formatted(Formatted),
    /// Something unresolved, rendered as its fallback text.
    Fallback(String),
}

#[derive(Clone, Debug)]
struct Formatted {
    /// The value the function formatted, for a later function on the same
    /// variable.
    value: Option<Value>,
    parts: Vec<OutPart>,
    keys: Keys,
}

/// What a formatted value offers to `.match`.
#[derive(Clone, Debug)]
enum Keys {
    None,
    /// Keys that match, best first.
    Exact(Vec<String>),
    /// A number: exact numeric match first, then its plural category.
    Plural {
        decimal: String,
        ordinal: bool,
    },
}

struct Eval<'a> {
    intl: &'a Intl,
    locale: &'a LocaleSource,
    params: &'a Params,
    depth: u8,
    style: NumberStyle,
    env: BTreeMap<String, Val>,
    warnings: Vec<String>,
}

fn text_parts(text: String) -> Vec<OutPart> {
    vec![OutPart::Text(text)]
}

impl<'a> Eval<'a> {
    fn new(intl: &'a Intl, locale: &'a LocaleSource, params: &'a Params, depth: u8) -> Eval<'a> {
        Eval {
            intl,
            locale,
            params,
            depth,
            style: NumberStyle::of(&locale.meta),
            env: BTreeMap::new(),
            warnings: Vec::new(),
        }
    }

    fn warn(&mut self, message: String) {
        self.warnings.push(message);
    }

    fn message(&mut self, message: &Message) -> Vec<OutPart> {
        match message {
            Message::Simple(pattern) => self.pattern(pattern),
            Message::Complex(complex) => {
                for declaration in &complex.declarations {
                    let (variable, expression) = match declaration {
                        Declaration::Input {
                            variable,
                            expression,
                        }
                        | Declaration::Local {
                            variable,
                            expression,
                        } => (variable, expression),
                    };
                    let value = self.expression(expression);
                    self.env.insert(variable.clone(), value);
                }
                match &complex.body {
                    Body::Pattern(pattern) => self.pattern(pattern),
                    Body::Matcher(matcher) => {
                        let pattern = self.select(matcher);
                        self.pattern(pattern)
                    }
                }
            }
        }
    }

    fn pattern(&mut self, pattern: &Pattern) -> Vec<OutPart> {
        let mut out = Vec::new();
        for part in &pattern.0 {
            match part {
                Part::Text(text) => out.push(OutPart::Text(text.clone())),
                Part::Expression(expression) => {
                    let value = self.expression(expression);
                    out.extend(self.parts_of(value));
                }
                Part::Markup(markup) => out.push(self.markup(markup)),
            }
        }
        out
    }

    fn markup(&mut self, markup: &Markup) -> OutPart {
        let options = self.options(&markup.options).into_iter().collect();
        OutPart::Markup {
            kind: markup.kind,
            name: markup.name.to_string(),
            options,
        }
    }

    fn variable(&mut self, name: &str) -> Val {
        if let Some(value) = self.env.get(name) {
            return value.clone();
        }
        if let Some(value) = self.params.get(name) {
            return Val::Raw(value.clone());
        }
        self.warn(format!("missing parameter `${name}`"));
        Val::Fallback(format!("{{${name}}}"))
    }

    fn expression(&mut self, expression: &Expression) -> Val {
        let operand = match &expression.operand {
            Some(Operand::Variable(name)) => Some(self.variable(name)),
            Some(Operand::Literal(literal)) => Some(Val::Raw(Value::Str(literal.value.clone()))),
            None => None,
        };
        match &expression.function {
            Some(function) => Val::Formatted(self.apply(function, operand)),
            None => operand.unwrap_or_else(|| Val::Fallback(String::from("{}"))),
        }
    }

    fn options(&mut self, options: &[Opt]) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for option in options {
            let value = match &option.value {
                OptValue::Literal(literal) => literal.value.clone(),
                OptValue::Variable(name) => {
                    let value = self.variable(name);
                    self.plain(Some(value))
                }
            };
            out.insert(option.name.to_string(), value);
        }
        out
    }

    /// A value as plain text under the default formatting.
    fn plain(&mut self, value: Option<Val>) -> String {
        let parts = self.parts_of(value.unwrap_or_else(|| Val::Fallback(String::new())));
        plain_text(&parts)
    }

    fn parts_of(&mut self, value: Val) -> Vec<OutPart> {
        match value {
            Val::Raw(raw) => text_parts(self.default_text(&raw)),
            Val::Formatted(formatted) => formatted.parts,
            Val::Fallback(text) => text_parts(text),
        }
    }

    fn default_text(&mut self, value: &Value) -> String {
        match value {
            Value::Str(s) => s.clone(),
            Value::Int(i) => self.style.integer(*i),
            Value::Num(n) => self.style.localise(&ascii_decimal(*n, 0, 3)),
            Value::Entity(key) => self.entity_form(key, "name"),
            Value::List(items) => {
                let texts: Vec<String> = items.iter().map(|item| self.default_text(item)).collect();
                self.join_list(&texts, "and")
            }
        }
    }

    fn entity_form(&mut self, key: &str, form: &str) -> String {
        let Some(entity) = self.intl.entity_from(&self.locale.tag, key) else {
            self.warn(format!("missing entity `{key}`"));
            return key.to_string();
        };
        if let Some(text) = entity.form(form) {
            text.to_string()
        } else {
            self.warn(format!("entity `{key}` has no `{form}` form"));
            entity.name().to_string()
        }
    }

    fn join_list(&mut self, items: &[String], kind: &str) -> String {
        let Some(pattern) = self.locale.meta.list_patterns.get(kind).cloned() else {
            self.warn(format!(
                "{} declares no `{kind}` list pattern",
                self.locale.tag
            ));
            return items.join(", ");
        };
        let fill = |template: &str, a: &str, b: &str| template.replace("{0}", a).replace("{1}", b);
        match items {
            [] => String::new(),
            [one] => one.clone(),
            [a, b] => fill(&pattern.pair, a, b),
            [first, middle @ .., last] => {
                let mut acc = first.clone();
                for item in middle {
                    acc = fill(&pattern.middle, &acc, item);
                }
                fill(&pattern.end, &acc, last)
            }
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "counts and ranks, far below the mantissa"
    )]
    fn numeric(&mut self, value: Option<&Val>) -> Option<f64> {
        let number = match value? {
            Val::Raw(Value::Int(i)) => Some(*i as f64),
            Val::Raw(Value::Num(n)) => Some(*n),
            Val::Raw(Value::Str(s)) => s.parse().ok(),
            Val::Formatted(formatted) => match (&formatted.value, &formatted.keys) {
                (Some(Value::Int(i)), _) => Some(*i as f64),
                (Some(Value::Num(n)), _) => Some(*n),
                (_, Keys::Plural { decimal, .. }) => decimal.parse().ok(),
                _ => None,
            },
            _ => None,
        };
        if number.is_none() {
            self.warn(String::from("a number was expected"));
        }
        number
    }

    fn source_value(value: Option<&Val>) -> Option<Value> {
        match value? {
            Val::Raw(raw) => Some(raw.clone()),
            Val::Formatted(formatted) => formatted.value.clone(),
            Val::Fallback(_) => None,
        }
    }

    fn fallback_text(value: Option<&Val>, function: &str) -> String {
        match value {
            Some(Val::Fallback(text)) => text.clone(),
            _ => format!("{{:{function}}}"),
        }
    }

    fn apply(&mut self, function: &Function, operand: Option<Val>) -> Formatted {
        let options = self.options(&function.options);
        let name = function.name.to_string();
        match name.as_str() {
            "string" => {
                let value = Self::source_value(operand.as_ref());
                let text = self.plain(operand);
                Formatted {
                    value,
                    parts: text_parts(text.clone()),
                    keys: Keys::Exact(vec![text]),
                }
            }
            "number" | "integer" => self.number(operand.as_ref(), &options, name == "integer"),
            "dms" => self.dms(operand.as_ref(), &options),
            "zodiac" => self.zodiac(operand.as_ref(), &options),
            "entity" => self.entity(operand.as_ref(), &options),
            "list" => self.list(operand, &options),
            "msg" => self.msg(operand),
            other => {
                self.warn(format!("unknown function `:{other}`"));
                Formatted {
                    value: Self::source_value(operand.as_ref()),
                    parts: text_parts(Self::fallback_text(operand.as_ref(), other)),
                    keys: Keys::None,
                }
            }
        }
    }

    fn number(
        &mut self,
        operand: Option<&Val>,
        options: &BTreeMap<String, String>,
        integer: bool,
    ) -> Formatted {
        let Some(mut value) = self.numeric(operand) else {
            return Formatted {
                value: None,
                parts: text_parts(Self::fallback_text(operand, "number")),
                keys: Keys::None,
            };
        };
        let option = |name: &str| options.get(name).and_then(|v| v.parse::<usize>().ok());
        let (min, max) = if integer {
            value = value.round();
            (0, 0)
        } else {
            let min = option("minimumFractionDigits").unwrap_or(0);
            (min, option("maximumFractionDigits").unwrap_or(min.max(3)))
        };
        let decimal = ascii_decimal(value, min, max);
        Formatted {
            value: Self::source_value(operand),
            parts: text_parts(self.style.localise(&decimal)),
            keys: Keys::Plural {
                decimal,
                ordinal: options.get("select").is_some_and(|s| s == "ordinal"),
            },
        }
    }

    /// Degrees, minutes and seconds of an angle, rounded at `precision`,
    /// as `(sign, degrees, minutes, seconds)`.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rounded non-negative magnitudes of a few thousand"
    )]
    fn sexagesimal(
        value: f64,
        precision: &str,
        signed: bool,
    ) -> (bool, u64, Option<u64>, Option<u64>) {
        let negative = signed && value < 0.0;
        let magnitude = if signed {
            value.abs()
        } else {
            value.rem_euclid(360.0)
        };
        match precision {
            "deg" => (negative, magnitude.round() as u64, None, None),
            "min" => {
                let total = (magnitude * 60.0).round() as u64;
                (negative, total / 60, Some(total % 60), None)
            }
            _ => {
                let total = (magnitude * 3600.0).round() as u64;
                (
                    negative,
                    total / 3600,
                    Some(total % 3600 / 60),
                    Some(total % 60),
                )
            }
        }
    }

    fn dms_text(
        &self,
        sign: bool,
        degrees: u64,
        minutes: Option<u64>,
        seconds: Option<u64>,
    ) -> String {
        let mut ascii = String::new();
        if sign {
            ascii.push('-');
        }
        let _ = write!(ascii, "{degrees}°");
        if let Some(minutes) = minutes {
            let _ = write!(ascii, "{minutes:02}′");
        }
        if let Some(seconds) = seconds {
            let _ = write!(ascii, "{seconds:02}″");
        }
        self.style.map_digits(&ascii)
    }

    fn dms(&mut self, operand: Option<&Val>, options: &BTreeMap<String, String>) -> Formatted {
        let Some(value) = self.numeric(operand) else {
            return Formatted {
                value: None,
                parts: text_parts(Self::fallback_text(operand, "dms")),
                keys: Keys::None,
            };
        };
        let precision = options.get("precision").map_or("sec", String::as_str);
        let signed = options.get("signed").is_some_and(|s| s == "true");
        let (sign, d, m, s) = Self::sexagesimal(value, precision, signed);
        Formatted {
            value: Self::source_value(operand),
            parts: text_parts(self.dms_text(sign, d, m, s)),
            keys: Keys::None,
        }
    }

    fn zodiac(&mut self, operand: Option<&Val>, options: &BTreeMap<String, String>) -> Formatted {
        let Some(value) = self.numeric(operand) else {
            return Formatted {
                value: None,
                parts: text_parts(Self::fallback_text(operand, "zodiac")),
                keys: Keys::None,
            };
        };
        let precision = options.get("precision").map_or("min", String::as_str);
        let (_, d, m, s) = Self::sexagesimal(value, precision, false);
        // Rounding may carry into the next sign: 29°59′59.7″ is 30°00′00″.
        let index = usize::try_from(d / 30).unwrap_or(0) % 12;
        let within = d % 30;
        let sign_key = rashi_key(index);
        let sign_form = options.get("signNames").map_or("name", String::as_str);
        let sign = self.entity_form(&sign_key, sign_form);
        let degrees = self.dms_text(false, within, m, s);
        let text = match options.get("form").map(String::as_str) {
            Some("sign-degree") => format!("{sign} {degrees}"),
            _ => format!("{degrees} {sign}"),
        };
        Formatted {
            value: Self::source_value(operand),
            parts: text_parts(text),
            keys: Keys::Exact(vec![sign_key]),
        }
    }

    fn entity(&mut self, operand: Option<&Val>, options: &BTreeMap<String, String>) -> Formatted {
        let Some(Value::Entity(key) | Value::Str(key)) = Self::source_value(operand) else {
            self.warn(String::from("`:entity` needs a catalogue key"));
            return Formatted {
                value: None,
                parts: text_parts(Self::fallback_text(operand, "entity")),
                keys: Keys::None,
            };
        };
        if let Some(kind) = options.get("kind") {
            if key.split('.').next() != Some(kind.as_str()) {
                self.warn(format!("entity `{key}` is not a `{kind}`"));
            }
        }
        if let Err(unknown) = teistro_core::key::resolve(&key) {
            self.warn(format!("entity `{key}` is not a catalogue key: {unknown}"));
        }
        let form = options.get("form").map_or("name", String::as_str);
        let text = self.entity_form(&key, form);
        let mut keys = vec![
            key.split_once('.')
                .map_or(key.as_str(), |(_, bare)| bare)
                .to_string(),
            key.clone(),
        ];
        if let Some(gender) = self
            .intl
            .entity_from(&self.locale.tag, &key)
            .and_then(|e| e.gender.clone())
        {
            keys.push(gender);
        }
        Formatted {
            value: Some(Value::Entity(key)),
            parts: text_parts(text),
            keys: Keys::Exact(keys),
        }
    }

    fn list(&mut self, operand: Option<Val>, options: &BTreeMap<String, String>) -> Formatted {
        let value = Self::source_value(operand.as_ref());
        let items: Vec<String> = match &value {
            Some(Value::List(items)) => items.iter().map(|item| self.default_text(item)).collect(),
            Some(other) => vec![self.default_text(other)],
            None => vec![self.plain(operand)],
        };
        let kind = options.get("type").map_or("and", String::as_str);
        let text = self.join_list(&items, kind);
        Formatted {
            value,
            parts: text_parts(text),
            keys: Keys::None,
        }
    }

    fn msg(&mut self, operand: Option<Val>) -> Formatted {
        let key = self.plain(operand);
        if self.depth >= MSG_DEPTH {
            self.warn(format!("`:msg` nesting deeper than {MSG_DEPTH} at `{key}`"));
            return Formatted {
                value: None,
                parts: text_parts(key),
                keys: Keys::None,
            };
        }
        let rendered = self
            .intl
            .render_from(&self.locale.tag, &key, self.params, self.depth + 1);
        self.warnings.extend(rendered.warnings);
        Formatted {
            value: Some(Value::Str(rendered.text)),
            parts: rendered.parts,
            keys: Keys::None,
        }
    }

    fn rank(&self, key: &Key, keys: &Keys) -> Option<usize> {
        let literal = match key {
            Key::Wildcard => return Some(usize::MAX),
            Key::Literal(literal) => &literal.value,
        };
        match keys {
            Keys::None => None,
            Keys::Exact(list) => list.iter().position(|k| k == literal),
            Keys::Plural { decimal, ordinal } => {
                let exact = literal
                    .parse::<f64>()
                    .ok()
                    .zip(decimal.parse::<f64>().ok())
                    .is_some_and(|(a, b)| a.total_cmp(&b).is_eq());
                if exact {
                    return Some(0);
                }
                let category = self.intl.category(&self.locale.tag, decimal, *ordinal)?;
                (category == literal).then_some(1)
            }
        }
    }

    fn select<'m>(&mut self, matcher: &'m Matcher) -> &'m Pattern {
        let selectors: Vec<Keys> = matcher
            .selectors
            .iter()
            .map(|name| match self.env.get(name) {
                Some(Val::Formatted(formatted)) => formatted.keys.clone(),
                _ => Keys::None,
            })
            .collect();
        let mut best: Option<(Vec<usize>, &Pattern)> = None;
        for variant in &matcher.variants {
            let ranks: Option<Vec<usize>> = variant
                .keys
                .iter()
                .zip(&selectors)
                .map(|(key, keys)| self.rank(key, keys))
                .collect();
            if let Some(ranks) = ranks {
                if best.as_ref().is_none_or(|(b, _)| ranks < *b) {
                    best = Some((ranks, &variant.pattern));
                }
            }
        }
        if let Some((_, pattern)) = best {
            pattern
        } else {
            self.warn(String::from(
                "no variant matched; the message has no `*` fallback",
            ));
            matcher.variants.last().map_or(&EMPTY, |v| &v.pattern)
        }
    }
}

static EMPTY: Pattern = Pattern(Vec::new());

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use super::*;
    use crate::source::sdk_root;

    fn intl(locale: &str) -> Intl {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        let mut intl = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
        intl.set_locale(locale).unwrap_or_else(|e| panic!("{e}"));
        intl
    }

    fn text(intl: &Intl, key: &str, params: &Params) -> String {
        let rendered = intl.render(key, params);
        assert!(
            rendered.warnings.is_empty(),
            "{key}: {:?}",
            rendered.warnings
        );
        rendered.text
    }

    fn graha(key: &str) -> Value {
        Value::entity(&format!("graha.{key}"))
    }

    #[test]
    fn a_catalogued_value_is_the_typed_entity_and_an_unknown_key_warns() {
        use teistro_core::catalogue::{Graha, Rashi};

        assert_eq!(
            Value::catalogued(Graha::Jupiter),
            Value::entity("graha.JUPITER")
        );
        let intl = intl("en-Latn");
        let good = intl.render(
            "sdk.reason.grahaInRashi",
            &params([
                ("graha", Value::catalogued(Graha::Mars)),
                ("rashi", Value::catalogued(Rashi::Leo)),
            ]),
        );
        assert!(good.warnings.is_empty(), "{:?}", good.warnings);
        assert_eq!(good.text, "Mars in Leo");
        let bad = intl.render(
            "sdk.reason.grahaInRashi",
            &params([("graha", graha("VULCAN")), ("rashi", graha("LEO"))]),
        );
        assert!(
            bad.warnings
                .iter()
                .any(|w| w.contains("not a catalogue key")),
            "{:?}",
            bad.warnings
        );
        // The sign keys of `:zodiac` are the catalogue's.
        assert_eq!(rashi_key(0), "rashi.ARIES");
        assert_eq!(rashi_key(11), "rashi.PISCES");
        assert_eq!(rashi_key(12), "rashi.ARIES");
    }

    #[test]
    fn the_generated_typed_messages_render_through_their_keys() {
        use teistro_core::catalogue::Graha;

        use crate::messages::sdk::reason::strength::Score;
        use crate::messages::sdk::reason::{AppName, GrahaInBhava};

        let intl = intl("ne-Deva-NP");
        let typed = intl.render_typed(&GrahaInBhava {
            bhava: 7,
            graha: Graha::Jupiter,
        });
        let untyped = intl.render(
            "sdk.reason.grahaInBhava",
            &params([
                ("graha", Value::catalogued(Graha::Jupiter)),
                ("bhava", Value::Int(7)),
            ]),
        );
        assert_eq!(typed, untyped);
        assert!(typed.warnings.is_empty(), "{:?}", typed.warnings);
        assert_eq!(<AppName as TypedMessage>::KEY, "sdk.reason.appName");
        assert_eq!(
            intl.render_typed(&AppName),
            intl.render("sdk.reason.appName", &Params::new())
        );
        let score = intl.render_typed(&Score {
            graha: Graha::Saturn,
            score: 4.5,
        });
        assert!(score.warnings.is_empty(), "{:?}", score.warnings);
    }

    #[test]
    fn ordinal_houses_in_both_languages() {
        let en = intl("en-Latn");
        let ne = intl("ne-Deva-NP");
        let cases = [
            (1, "Jupiter in the 1st house", "गुरु पहिलो भावमा"),
            (2, "Jupiter in the 2nd house", "गुरु दोस्रो भावमा"),
            (3, "Jupiter in the 3rd house", "गुरु तेस्रो भावमा"),
            (4, "Jupiter in the 4th house", "गुरु चौथो भावमा"),
            (7, "Jupiter in the 7th house", "गुरु ७औं भावमा"),
            (11, "Jupiter in the 11th house", "गुरु ११औं भावमा"),
            (12, "Jupiter in the 12th house", "गुरु १२औं भावमा"),
        ];
        for (bhava, expected_en, expected_ne) in cases {
            let p = params([("graha", graha("JUPITER")), ("bhava", Value::Int(bhava))]);
            assert_eq!(text(&en, "sdk.reason.grahaInBhava", &p), expected_en);
            assert_eq!(text(&ne, "sdk.reason.grahaInBhava", &p), expected_ne);
        }
        let p = params([("rank", Value::Int(21))]);
        assert_eq!(text(&en, "sdk.reason.strength.rank", &p), "21st");
        assert_eq!(text(&ne, "sdk.reason.strength.rank", &p), "२१औं");
        let p = params([("rank", Value::Int(2))]);
        assert_eq!(text(&ne, "sdk.reason.strength.rank", &p), "२रो");
    }

    #[test]
    fn cardinal_plurals_and_prose_forms() {
        let en = intl("en-Latn");
        let ne = intl("ne-Deva-NP");
        let cases = [
            (0, "No planet conjoins the Sun", "सूर्यसँग कुनै ग्रह छैन"),
            (1, "One planet conjoins the Sun", "सूर्यसँग एक ग्रह छ"),
            (2, "2 planets conjoin the Sun", "सूर्यसँग २ ग्रह छन्"),
        ];
        for (count, expected_en, expected_ne) in cases {
            let p = params([("graha", graha("SUN")), ("count", Value::Int(count))]);
            assert_eq!(text(&en, "sdk.reason.conjunction", &p), expected_en);
            assert_eq!(text(&ne, "sdk.reason.conjunction", &p), expected_ne);
        }
    }

    #[test]
    fn selection_on_an_entitys_gender_and_on_a_context() {
        let en = intl("en-Latn");
        let ne = intl("ne-Deva-NP");
        let scorpio = params([("rashi", Value::entity("rashi.SCORPIO"))]);
        let aries = params([("rashi", Value::entity("rashi.ARIES"))]);
        assert_eq!(
            text(&en, "sdk.reason.rashiNature", &scorpio),
            "Scorpio is a feminine sign"
        );
        assert_eq!(
            text(&ne, "sdk.reason.rashiNature", &aries),
            "मेष पुरुष राशि हो"
        );
        let sita = params([("gender", "f".into()), ("name", "Sita".into())]);
        let ram = params([("gender", "m".into()), ("name", "Ram".into())]);
        assert_eq!(
            text(&en, "sdk.reason.greeting", &sita),
            "Dear Sita, your chart is ready."
        );
        assert_eq!(
            text(&ne, "sdk.reason.greeting", &sita),
            "प्रिय Sita ज्यू, तपाईंकी कुण्डली तयार छ।"
        );
        assert_eq!(
            text(&ne, "sdk.reason.greeting", &ram),
            "प्रिय Ram ज्यू, तपाईंको कुण्डली तयार छ।"
        );
    }

    #[test]
    fn angles_in_both_numbering_systems() {
        let en = intl("en-Latn");
        let ne = intl("ne-Deva-NP");
        let p = params([
            ("graha", graha("MARS")),
            ("longitude", Value::Num(222.5763)),
        ]);
        assert_eq!(
            text(&en, "sdk.reason.grahaAt", &p),
            "Mars at 12°35′ Scorpio"
        );
        assert_eq!(text(&ne, "sdk.reason.grahaAt", &p), "मंगल १२°३५′ वृश्चिकमा");
        let p = params([("longitude", Value::Num(222.576_388_9))]);
        assert_eq!(text(&en, "sdk.reason.exactLongitude", &p), "222°34′35″");
        assert_eq!(text(&ne, "sdk.reason.exactLongitude", &p), "२२२°३४′३५″");
        // Rounding carries across a sign boundary.
        let p = params([("graha", graha("SUN")), ("longitude", Value::Num(29.9999))]);
        assert_eq!(text(&en, "sdk.reason.grahaAt", &p), "Sun at 0°00′ Taurus");
    }

    #[test]
    fn lists_markup_links_and_numbers() {
        let en = intl("en-Latn");
        let ne = intl("ne-Deva-NP");
        let p = params([
            (
                "grahas",
                vec![graha("SUN"), graha("MOON"), graha("MARS")].into(),
            ),
            ("rashi", Value::entity("rashi.LEO")),
        ]);
        assert_eq!(
            text(&en, "sdk.reason.occupants", &p),
            "Sun, Moon and Mars in Leo"
        );
        assert_eq!(
            text(&ne, "sdk.reason.occupants", &p),
            "सिंहमा सूर्य, चन्द्र र मंगल"
        );
        let p = params([("graha", graha("JUPITER")), ("bhava", Value::Int(9))]);
        let rendered = en.render("sdk.reason.lordship", &p);
        assert_eq!(rendered.text, "Jupiter rules house 9");
        assert!(matches!(
            rendered.parts.first(),
            Some(OutPart::Markup { kind: MarkupKind::Open, name, .. }) if name == "b"
        ));
        assert_eq!(
            text(&en, "sdk.reason.welcome", &Params::new()),
            "Welcome to Teistro"
        );
        assert_eq!(
            text(&ne, "sdk.reason.welcome", &Params::new()),
            "टेइस्ट्रोमा स्वागत छ"
        );
        let p = params([("graha", graha("SATURN")), ("score", Value::Num(5.5))]);
        assert_eq!(
            text(&en, "sdk.reason.strength.score", &p),
            "Saturn scores 5.50 rupas"
        );
        assert_eq!(
            text(&ne, "sdk.reason.strength.score", &p),
            "शनिले ५.५० रूपा पाउँछ"
        );
        let big = params([("n", Value::Int(1_234_567))]);
        let indian = ne
            .render_source("{$n :integer}", &big)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(indian.text, "१२,३४,५६७");
        let western = en
            .render_source("{$n :integer}", &big)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(western.text, "1,234,567");
    }

    #[test]
    fn fallback_missing_keys_and_missing_parameters_are_reported() {
        let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
        let mut locales = tree.locales.clone();
        if let Some(ne) = locales.get_mut("ne-Deva-NP") {
            if let Some(ns) = ne.namespaces.get_mut("sdk.reason") {
                ns.entries.remove("welcome");
            }
        }
        let mut intl = Intl::new(locales).unwrap_or_else(|e| panic!("{e}"));
        intl.set_locale("ne-Deva-NP")
            .unwrap_or_else(|e| panic!("{e}"));
        let rendered = intl.render("sdk.reason.welcome", &Params::new());
        assert_eq!(rendered.text, "Welcome to Teistro");
        assert_eq!(rendered.resolved_from.as_deref(), Some("en-Latn"));
        assert!(rendered.is_fallback);
        let missing = intl.render("sdk.reason.nothing", &Params::new());
        assert_eq!(missing.text, "sdk.reason.nothing");
        assert_eq!(missing.resolved_from, None);
        assert_eq!(missing.warnings.len(), 1);
        let p = params([("graha", graha("SUN"))]);
        let partial = intl.render("sdk.reason.grahaInBhava", &p);
        assert!(partial.text.contains("{$bhava}"), "{}", partial.text);
        assert!(partial.warnings.iter().any(|w| w.contains("$bhava")));
        assert!(intl.has("sdk.reason.welcome"));
        assert!(!intl.has("sdk.reason.nothing"));
    }

    #[test]
    fn decimals_and_localisation_helpers() {
        assert_eq!(ascii_decimal(5.5, 2, 3), "5.50");
        assert_eq!(ascii_decimal(5.0, 0, 3), "5");
        assert_eq!(ascii_decimal(1.23456, 0, 3), "1.235");
        assert_eq!(ascii_decimal(-0.0001, 0, 2), "0");
        assert_eq!(ascii_decimal(-2.5, 0, 3), "-2.5");
        let deva = NumberStyle {
            digits: digits("deva").unwrap_or(['0'; 10]),
            decimal: '.',
            group: ',',
            grouping: vec![3, 2],
        };
        assert_eq!(deva.localise("1234567.5"), "१२,३४,५६७.५");
        assert_eq!(deva.localise("-12"), "-१२");
        assert_eq!(deva.localise("999"), "९९९");
        assert_eq!(deva.localise("1000"), "१,०००");
    }
}
