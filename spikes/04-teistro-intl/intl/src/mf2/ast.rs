//! The `MessageFormat 2` data model, one type per production of the LDML 47
//! grammar the engine implements. Equality is structural; `Display` on
//! [`Message`] serialises back to source that parses to the same value.

/// A message: a bare pattern, or declarations with a pattern or a matcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A pattern with no declarations and no `{{ }}`; it cannot start with
    /// whitespace or a full stop, which is why such patterns are complex.
    Simple(Pattern),
    /// Declarations followed by a quoted pattern or a matcher.
    Complex(Complex),
}

/// The complex form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Complex {
    /// `.input` and `.local` declarations, in source order.
    pub declarations: Vec<Declaration>,
    /// What the message renders.
    pub body: Body,
}

/// A declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    /// `.input {$variable :function options}`: annotates an argument.
    Input {
        /// The argument's name, the expression's operand.
        variable: String,
        /// The whole variable expression.
        expression: Expression,
    },
    /// `.local $variable = {expression}`: a derived value.
    Local {
        /// The new variable.
        variable: String,
        /// Its expression.
        expression: Expression,
    },
}

/// The body of a complex message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    /// `{{ pattern }}`.
    Pattern(Pattern),
    /// `.match` with selectors and variants.
    Matcher(Matcher),
}

/// `.match $a $b key key {{...}} * * {{...}}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matcher {
    /// The selector variables, each declared with an annotation.
    pub selectors: Vec<String>,
    /// The variants, in source order.
    pub variants: Vec<Variant>,
}

/// One variant: as many keys as there are selectors, and a pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    /// The keys.
    pub keys: Vec<Key>,
    /// The pattern.
    pub pattern: Pattern,
}

/// A variant key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// A literal to match.
    Literal(Literal),
    /// `*`, matching anything.
    Wildcard,
}

/// Text, placeholders and markup in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern(pub Vec<Part>);

/// One part of a pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// Literal text, unescaped.
    Text(String),
    /// `{...}`.
    Expression(Expression),
    /// `{#tag}`, `{/tag}` or `{#tag /}`.
    Markup(Markup),
}

/// An expression: an operand, a function, or both, with attributes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    /// The literal or variable, absent for a bare function.
    pub operand: Option<Operand>,
    /// The annotation, required when there is no operand.
    pub function: Option<Function>,
    /// `@name` or `@name=literal`, carried and otherwise ignored.
    pub attributes: Vec<Attribute>,
}

/// What an expression operates on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    /// A literal.
    Literal(Literal),
    /// `$name`.
    Variable(String),
}

/// A literal, with whether it was quoted in the source so that it
/// serialises the same way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Literal {
    /// The value, unescaped.
    pub value: String,
    /// Whether the source wrote `|value|`.
    pub quoted: bool,
}

/// `:name option=value ...`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The function's identifier.
    pub name: Identifier,
    /// Its options, in source order.
    pub options: Vec<Opt>,
}

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opt {
    /// The option's identifier.
    pub name: Identifier,
    /// Its value.
    pub value: OptValue,
}

/// An option's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptValue {
    /// A literal.
    Literal(Literal),
    /// `$name`.
    Variable(String),
}

/// `@name` or `@name=literal`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The attribute's identifier.
    pub name: Identifier,
    /// Its literal value, when given.
    pub value: Option<Literal>,
}

/// A namespaced name: `name` or `namespace:name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    /// The namespace, when given.
    pub namespace: Option<String>,
    /// The name.
    pub name: String,
}

impl Identifier {
    /// A plain identifier.
    #[must_use]
    pub fn plain(name: &str) -> Identifier {
        Identifier {
            namespace: None,
            name: name.to_string(),
        }
    }

    /// Whether this is the plain identifier `name`.
    #[must_use]
    pub fn is(&self, name: &str) -> bool {
        self.namespace.is_none() && self.name == name
    }
}

/// Markup: an open, close or standalone tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Markup {
    /// Open, close or standalone.
    pub kind: MarkupKind,
    /// The tag's identifier.
    pub name: Identifier,
    /// Options.
    pub options: Vec<Opt>,
    /// Attributes.
    pub attributes: Vec<Attribute>,
}

/// The three markup forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkupKind {
    /// `{#tag}`.
    Open,
    /// `{#tag /}`.
    Standalone,
    /// `{/tag}`.
    Close,
}

impl Message {
    /// The parts of every pattern the message can render, for analysis.
    #[must_use]
    pub fn patterns(&self) -> Vec<&Pattern> {
        match self {
            Message::Simple(pattern) => vec![pattern],
            Message::Complex(complex) => match &complex.body {
                Body::Pattern(pattern) => vec![pattern],
                Body::Matcher(matcher) => matcher.variants.iter().map(|v| &v.pattern).collect(),
            },
        }
    }

    /// The declarations, empty for a simple message.
    #[must_use]
    pub fn declarations(&self) -> &[Declaration] {
        match self {
            Message::Simple(_) => &[],
            Message::Complex(complex) => &complex.declarations,
        }
    }

    /// Every expression in the message: declarations, then patterns in
    /// order.
    #[must_use]
    pub fn expressions(&self) -> Vec<&Expression> {
        let mut out: Vec<&Expression> =
            self.declarations()
                .iter()
                .map(|d| match d {
                    Declaration::Input { expression, .. }
                    | Declaration::Local { expression, .. } => expression,
                })
                .collect();
        for pattern in self.patterns() {
            out.extend(pattern.0.iter().filter_map(|part| match part {
                Part::Expression(expression) => Some(expression),
                Part::Text(_) | Part::Markup(_) => None,
            }));
        }
        out
    }
}

impl Function {
    /// The literal value of an option, when it is a literal.
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&str> {
        self.options.iter().find_map(|o| match (&o.name, &o.value) {
            (id, OptValue::Literal(literal)) if id.is(name) => Some(literal.value.as_str()),
            _ => None,
        })
    }
}
