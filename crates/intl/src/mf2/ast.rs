//! The `MessageFormat 2` data model, one type per production of the LDML 47
//! grammar the engine implements. Equality is structural; `Display` on
//! [`Message`] serialises back to source that parses to the same value.

/// A message: a bare pattern, or declarations with a pattern or a matcher.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::Message;
///
/// // Text with a placeholder is simple; declarations make it complex.
/// assert!(matches!(parse("Hello {$name}")?, Message::Simple(_)));
/// assert!(matches!(parse(".input {$n :integer} {{{$n}}}")?, Message::Complex(_)));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A pattern with no declarations and no `{{ }}`; it cannot start with
    /// whitespace or a full stop, which is why such patterns are complex.
    Simple(Pattern),
    /// Declarations followed by a quoted pattern or a matcher.
    Complex(Complex),
}

/// The complex form.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Body;
/// use teistro_intl::mf2::Message;
///
/// let Message::Complex(complex) = parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)? else { unreachable!() };
/// assert_eq!(complex.declarations.len(), 1);
/// assert!(matches!(complex.body, Body::Matcher(_)));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Complex {
    /// `.input` and `.local` declarations, in source order.
    pub declarations: Vec<Declaration>,
    /// What the message renders.
    pub body: Body,
}

/// A declaration.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Declaration;
///
/// let message = parse(r#".input {$n :integer} .local $m = {$n :number} {{{$m}}}"#)?;
/// assert!(matches!(&message.declarations()[0], Declaration::Input { variable, .. } if variable == "n"));
/// assert!(matches!(&message.declarations()[1], Declaration::Local { variable, .. } if variable == "m"));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
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
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Body;
/// use teistro_intl::mf2::Message;
///
/// let Message::Complex(quoted) = parse(".input {$n :integer} {{{$n} days}}")? else { unreachable!() };
/// assert!(matches!(quoted.body, Body::Pattern(_)));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    /// `{{ pattern }}`.
    Pattern(Pattern),
    /// `.match` with selectors and variants.
    Matcher(Matcher),
}

/// `.match $a $b key key {{...}} * * {{...}}`.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::{Body, Key};
/// use teistro_intl::mf2::Message;
///
/// let Message::Complex(complex) = parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)? else { unreachable!() };
/// let Body::Matcher(matcher) = complex.body else { unreachable!() };
/// assert_eq!(matcher.selectors, ["n"]);
/// assert_eq!(matcher.variants[1].keys, [Key::Wildcard]);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matcher {
    /// The selector variables, each declared with an annotation.
    pub selectors: Vec<String>,
    /// The variants, in source order.
    pub variants: Vec<Variant>,
}

/// One variant: as many keys as there are selectors, and a pattern.
///
/// ```
/// use teistro_intl::mf2::parse;
///
/// let message = parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)?;
/// // One pattern a variant: two here.
/// assert_eq!(message.patterns().len(), 2);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    /// The keys.
    pub keys: Vec<Key>,
    /// The pattern.
    pub pattern: Pattern,
}

/// A variant key.
///
/// ```
/// use teistro_intl::mf2::ast::{Key, Literal};
///
/// let one = Key::Literal(Literal { value: String::from("one"), quoted: false });
/// assert_ne!(one, Key::Wildcard);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// A literal to match.
    Literal(Literal),
    /// `*`, matching anything.
    Wildcard,
}

/// Text, placeholders and markup in order.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Part;
/// use teistro_intl::mf2::Message;
///
/// let Message::Simple(pattern) = parse("Hello {$name}!")? else { unreachable!() };
/// assert_eq!(pattern.0.len(), 3);
/// assert_eq!(pattern.0[0], Part::Text(String::from("Hello ")));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern(pub Vec<Part>);

/// One part of a pattern.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Part;
/// use teistro_intl::mf2::Message;
///
/// let Message::Simple(pattern) = parse("{#b}Mars{/b}")? else { unreachable!() };
/// assert!(matches!(pattern.0[0], Part::Markup(_)));
/// assert_eq!(pattern.0[1], Part::Text(String::from("Mars")));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
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
///
/// ```
/// use teistro_intl::mf2::parse;
///
/// let message = parse(r#"{$n :integer @locale=en}"#)?;
/// let expression = message.expressions()[0];
/// assert!(expression.operand.is_some() && expression.function.is_some());
/// assert_eq!(expression.attributes.len(), 1);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
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
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Operand;
///
/// let message = parse(r#"{|text| :string} {$name}"#)?;
/// let operands: Vec<_> = message.expressions().iter().map(|e| e.operand.clone()).collect();
/// assert!(matches!(&operands[0], Some(Operand::Literal(literal)) if literal.value == "text"));
/// assert_eq!(operands[1], Some(Operand::Variable(String::from("name"))));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    /// A literal.
    Literal(Literal),
    /// `$name`.
    Variable(String),
}

/// A literal, with whether it was quoted in the source so that it
/// serialises the same way.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::Operand;
///
/// // Quoted or not, it serialises as written.
/// let message = parse(r#"{|two words|} {plain}"#)?;
/// let quoted: Vec<bool> = message.expressions().iter().filter_map(|e| match &e.operand { Some(Operand::Literal(literal)) => Some(literal.quoted), _ => None }).collect();
/// assert_eq!(quoted, [true, false]);
/// assert_eq!(message.to_string(), "{|two words|} {plain}");
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Literal {
    /// The value, unescaped.
    pub value: String,
    /// Whether the source wrote `|value|`.
    pub quoted: bool,
}

/// `:name option=value ...`.
///
/// ```
/// use teistro_intl::mf2::parse;
///
/// let message = parse(r#"{$n :number minimumFractionDigits=2}"#)?;
/// let function = message.expressions()[0].function.as_ref().expect("annotated");
/// assert!(function.name.is("number"));
/// assert_eq!(function.options.len(), 1);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The function's identifier.
    pub name: Identifier,
    /// Its options, in source order.
    pub options: Vec<Opt>,
}

/// One option.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::OptValue;
///
/// let message = parse(r#"{$at :time timeZone=$zone}"#)?;
/// let option = &message.expressions()[0].function.as_ref().expect("annotated").options[0];
/// assert!(option.name.is("timeZone"));
/// assert_eq!(option.value, OptValue::Variable(String::from("zone")));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opt {
    /// The option's identifier.
    pub name: Identifier,
    /// Its value.
    pub value: OptValue,
}

/// An option's value.
///
/// ```
/// use teistro_intl::mf2::ast::{Literal, OptValue};
///
/// let fixed = OptValue::Literal(Literal { value: String::from("2"), quoted: false });
/// assert_ne!(fixed, OptValue::Variable(String::from("digits")));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptValue {
    /// A literal.
    Literal(Literal),
    /// `$name`.
    Variable(String),
}

/// `@name` or `@name=literal`.
///
/// ```
/// use teistro_intl::mf2::parse;
///
/// let message = parse(r#"{$n @translate=no}"#)?;
/// let attribute = &message.expressions()[0].attributes[0];
/// assert!(attribute.name.is("translate"));
/// assert_eq!(attribute.value.as_ref().map(|v| v.value.as_str()), Some("no"));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The attribute's identifier.
    pub name: Identifier,
    /// Its literal value, when given.
    pub value: Option<Literal>,
}

/// A namespaced name: `name` or `namespace:name`.
///
/// ```
/// use teistro_intl::mf2::parse;
///
/// let message = parse(r#"{$n :u:custom}"#)?;
/// let name = &message.expressions()[0].function.as_ref().expect("annotated").name;
/// assert_eq!((name.namespace.as_deref(), name.name.as_str()), (Some("u"), "custom"));
/// assert!(!name.is("custom"));
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    /// The namespace, when given.
    pub namespace: Option<String>,
    /// The name.
    pub name: String,
}

impl Identifier {
    /// A plain identifier.
    ///
    /// ```
    /// use teistro_intl::mf2::ast::Identifier;
    ///
    /// let number = Identifier::plain("number");
    /// assert!(number.namespace.is_none() && number.is("number"));
    /// ```
    #[must_use]
    pub fn plain(name: &str) -> Identifier {
        Identifier {
            namespace: None,
            name: name.to_string(),
        }
    }

    /// Whether this is the plain identifier `name`.
    ///
    /// ```
    /// use teistro_intl::mf2::ast::Identifier;
    ///
    /// assert!(Identifier::plain("integer").is("integer"));
    /// let namespaced = Identifier { namespace: Some(String::from("x")), name: String::from("integer") };
    /// assert!(!namespaced.is("integer"));
    /// ```
    #[must_use]
    pub fn is(&self, name: &str) -> bool {
        self.namespace.is_none() && self.name == name
    }
}

/// Markup: an open, close or standalone tag.
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::{MarkupKind, Part};
/// use teistro_intl::mf2::Message;
///
/// let Message::Simple(pattern) = parse("{#link href=|x| /}")? else { unreachable!() };
/// let Part::Markup(link) = &pattern.0[0] else { unreachable!() };
/// assert_eq!(link.kind, MarkupKind::Standalone);
/// assert!(link.name.is("link") && link.options.len() == 1);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
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
///
/// ```
/// use teistro_intl::mf2::parse;
/// use teistro_intl::mf2::ast::{MarkupKind, Part};
/// use teistro_intl::mf2::Message;
///
/// let Message::Simple(pattern) = parse("{#b}x{/b}")? else { unreachable!() };
/// let kinds: Vec<MarkupKind> = pattern.0.iter().filter_map(|part| match part { Part::Markup(markup) => Some(markup.kind), _ => None }).collect();
/// assert_eq!(kinds, [MarkupKind::Open, MarkupKind::Close]);
/// # Ok::<(), teistro_intl::mf2::ParseError>(())
/// ```
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
    ///
    /// ```
    /// use teistro_intl::mf2::parse;
    ///
    /// // A simple message has one; a matcher, one a variant.
    /// assert_eq!(parse("plain")?.patterns().len(), 1);
    /// assert_eq!(parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)?.patterns().len(), 2);
    /// # Ok::<(), teistro_intl::mf2::ParseError>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_intl::mf2::parse;
    ///
    /// assert!(parse("plain")?.declarations().is_empty());
    /// assert_eq!(parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)?.declarations().len(), 1);
    /// # Ok::<(), teistro_intl::mf2::ParseError>(())
    /// ```
    #[must_use]
    pub fn declarations(&self) -> &[Declaration] {
        match self {
            Message::Simple(_) => &[],
            Message::Complex(complex) => &complex.declarations,
        }
    }

    /// Every expression in the message: declarations, then patterns in
    /// order.
    ///
    /// ```
    /// use teistro_intl::mf2::parse;
    ///
    /// // The declaration's, then each variant's placeholder.
    /// assert_eq!(parse(r#".input {$n :integer select=ordinal} .match $n one {{{$n}st}} * {{{$n}th}}"#)?.expressions().len(), 3);
    /// # Ok::<(), teistro_intl::mf2::ParseError>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_intl::mf2::parse;
    ///
    /// let message = parse(r#"{$n :integer select=ordinal}"#)?;
    /// let function = message.expressions()[0].function.as_ref().expect("annotated");
    /// assert_eq!(function.option("select"), Some("ordinal"));
    /// assert_eq!(function.option("style"), None);
    /// # Ok::<(), teistro_intl::mf2::ParseError>(())
    /// ```
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&str> {
        self.options.iter().find_map(|o| match (&o.name, &o.value) {
            (id, OptValue::Literal(literal)) if id.is(name) => Some(literal.value.as_str()),
            _ => None,
        })
    }

    /// Whether the function names an option at all, as a literal or as a
    /// variable the caller supplies at render time.
    ///
    /// ```
    /// use teistro_intl::mf2::parse;
    ///
    /// // A variable option has no literal value, but is named.
    /// let message = parse(r#"{$at :time timeZone=$zone}"#)?;
    /// let function = message.expressions()[0].function.as_ref().expect("annotated");
    /// assert!(function.has_option("timeZone") && function.option("timeZone").is_none());
    /// # Ok::<(), teistro_intl::mf2::ParseError>(())
    /// ```
    #[must_use]
    pub fn has_option(&self, name: &str) -> bool {
        self.options.iter().any(|o| o.name.is(name))
    }
}
