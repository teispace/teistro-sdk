//! A recursive-descent parser for the `MessageFormat 2` grammar (LDML 47,
//! `message.abnf`), byte offsets in every error, no allocation beyond the
//! tree it returns. What it accepts is the standard; what the SDK adds
//! (its functions, `:msg`, entity selection) is vocabulary, not syntax.

use core::fmt;

use super::ast::{
    Attribute, Body, Complex, Declaration, Expression, Function, Identifier, Key, Literal, Markup,
    MarkupKind, Matcher, Message, Operand, Opt, OptValue, Part, Pattern, Variant,
};

/// Why a message did not parse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The byte offset in the source, when the error is at a place.
    pub offset: Option<usize>,
    /// What went wrong.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} at offset {offset}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a message and checks it against the data-model rules.
///
/// # Errors
///
/// A syntax error with its offset, or a data-model error.
pub fn parse(source: &str) -> Result<Message, ParseError> {
    let mut parser = Parser {
        src: source,
        pos: 0,
    };
    let message = parser.message()?;
    super::check::check(&message).map_err(|message| ParseError {
        offset: None,
        message,
    })?;
    Ok(message)
}

/// Whitespace per the grammar: space, tab, CR, LF, ideographic space, and
/// the bidi marks the grammar allows where whitespace is.
fn is_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\r' | '\n' | '\u{3000}' | '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{2066}'
            ..='\u{2069}'
    )
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || matches!(
            c,
            '\u{C0}'..='\u{D6}'
                | '\u{D8}'..='\u{F6}'
                | '\u{F8}'..='\u{2FF}'
                | '\u{370}'..='\u{37D}'
                | '\u{37F}'..='\u{1FFF}'
                | '\u{200C}'..='\u{200D}'
                | '\u{2070}'..='\u{218F}'
                | '\u{2C00}'..='\u{2FEF}'
                | '\u{3001}'..='\u{D7FF}'
                | '\u{F900}'..='\u{FDCF}'
                | '\u{FDF0}'..='\u{FFFC}'
                | '\u{10000}'..='\u{EFFFF}'
        )
}

fn is_name_char(c: char) -> bool {
    is_name_start(c)
        || c.is_ascii_digit()
        || matches!(c, '-' | '.' | '\u{B7}' | '\u{300}'..='\u{36F}' | '\u{203F}'..='\u{2040}')
}

/// Whether `s` is a `name` of the grammar.
#[must_use]
pub fn is_name(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(is_name_start) && chars.all(is_name_char)
}

/// Whether `s` is a `number-literal` of the grammar.
#[must_use]
pub fn is_number_literal(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    let (int, rest) = match s.find(|c: char| !c.is_ascii_digit()) {
        Some(i) => s.split_at(i),
        None => (s, ""),
    };
    let int_ok = int == "0" || (!int.is_empty() && !int.starts_with('0'));
    if !int_ok {
        return false;
    }
    let rest = match rest.strip_prefix('.') {
        Some(frac) => {
            let digits = frac.len() - frac.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            if digits == 0 {
                return false;
            }
            &frac[digits..]
        }
        None => rest,
    };
    match rest.strip_prefix(['e', 'E']) {
        Some(exp) => {
            let exp = exp.strip_prefix(['-', '+']).unwrap_or(exp);
            !exp.is_empty() && exp.bytes().all(|b| b.is_ascii_digit())
        }
        None => rest.is_empty(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PatternEnd {
    Source,
    Quoted,
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn rest(&self) -> &str {
        self.src.get(self.pos..).unwrap_or_default()
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.rest().starts_with(s)
    }

    fn eat(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn fail<T>(&self, message: &str) -> Result<T, ParseError> {
        Err(ParseError {
            offset: Some(self.pos),
            message: message.to_string(),
        })
    }

    fn expect(&mut self, s: &str, what: &str) -> Result<(), ParseError> {
        if self.eat(s) {
            Ok(())
        } else {
            self.fail(&format!("expected {what}"))
        }
    }

    /// `o`: optional whitespace.
    fn skip_o(&mut self) {
        while self.peek().is_some_and(is_space) {
            self.bump();
        }
    }

    /// `s`: required whitespace; `false` when there is none.
    fn skip_s(&mut self) -> bool {
        let start = self.pos;
        self.skip_o();
        self.pos > start
    }

    fn message(&mut self) -> Result<Message, ParseError> {
        self.skip_o();
        if self.starts_with(".") || self.starts_with("{{") {
            self.complex()
        } else {
            let pattern = self.pattern(PatternEnd::Source)?;
            Ok(Message::Simple(pattern))
        }
    }

    fn complex(&mut self) -> Result<Message, ParseError> {
        let mut declarations = Vec::new();
        loop {
            self.skip_o();
            if self.eat(".input") {
                self.skip_o();
                let expression = self.expression()?;
                let Some(Operand::Variable(variable)) = &expression.operand else {
                    return self.fail("`.input` takes a variable expression");
                };
                declarations.push(Declaration::Input {
                    variable: variable.clone(),
                    expression,
                });
            } else if self.eat(".local") {
                if !self.skip_s() {
                    return self.fail("expected whitespace after `.local`");
                }
                let variable = self.variable()?;
                self.skip_o();
                self.expect("=", "`=`")?;
                self.skip_o();
                let expression = self.expression()?;
                declarations.push(Declaration::Local {
                    variable,
                    expression,
                });
            } else {
                break;
            }
        }
        self.skip_o();
        let body = if self.eat(".match") {
            self.matcher()?
        } else if self.starts_with("{{") {
            Body::Pattern(self.quoted_pattern()?)
        } else {
            return self.fail("expected `.input`, `.local`, `.match` or a quoted pattern");
        };
        self.skip_o();
        if !self.at_end() {
            return self.fail("unexpected text after the message body");
        }
        Ok(Message::Complex(Complex { declarations, body }))
    }

    fn matcher(&mut self) -> Result<Body, ParseError> {
        let mut selectors = Vec::new();
        loop {
            let save = self.pos;
            if !self.skip_s() || !self.starts_with("$") {
                self.pos = save;
                break;
            }
            selectors.push(self.variable()?);
        }
        if selectors.is_empty() {
            return self.fail("`.match` needs at least one selector variable");
        }
        let mut variants = Vec::new();
        loop {
            self.skip_o();
            if self.at_end() {
                break;
            }
            let keys = self.keys()?;
            self.skip_o();
            let pattern = self.quoted_pattern()?;
            variants.push(Variant { keys, pattern });
        }
        if variants.is_empty() {
            return self.fail("`.match` needs at least one variant");
        }
        Ok(Body::Matcher(Matcher {
            selectors,
            variants,
        }))
    }

    fn keys(&mut self) -> Result<Vec<Key>, ParseError> {
        let mut keys = vec![self.key()?];
        loop {
            let save = self.pos;
            if !self.skip_s() || self.starts_with("{{") {
                self.pos = save;
                break;
            }
            keys.push(self.key()?);
        }
        Ok(keys)
    }

    fn key(&mut self) -> Result<Key, ParseError> {
        if self.eat("*") {
            Ok(Key::Wildcard)
        } else {
            Ok(Key::Literal(self.literal()?))
        }
    }

    fn quoted_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.expect("{{", "`{{`")?;
        let pattern = self.pattern(PatternEnd::Quoted)?;
        self.expect("}}", "`}}`")?;
        Ok(pattern)
    }

    fn pattern(&mut self, end: PatternEnd) -> Result<Pattern, ParseError> {
        let mut parts = Vec::new();
        let mut text = String::new();
        let flush = |text: &mut String, parts: &mut Vec<Part>| {
            if !text.is_empty() {
                parts.push(Part::Text(core::mem::take(text)));
            }
        };
        loop {
            match self.peek() {
                None => {
                    if end == PatternEnd::Quoted {
                        return self.fail("unterminated quoted pattern");
                    }
                    break;
                }
                Some('}') => {
                    if end == PatternEnd::Quoted && self.starts_with("}}") {
                        break;
                    }
                    return self.fail("`}` in text must be escaped as `\\}`");
                }
                Some('\\') => {
                    self.bump();
                    match self.bump() {
                        Some(c @ ('\\' | '{' | '|' | '}')) => text.push(c),
                        _ => {
                            return self.fail(
                                "invalid escape; only `\\\\`, `\\{`, `\\|` and `\\}` are allowed",
                            );
                        }
                    }
                }
                Some('{') => {
                    flush(&mut text, &mut parts);
                    parts.push(self.placeholder()?);
                }
                Some('\0') => return self.fail("NUL is not allowed in text"),
                Some(c) => {
                    text.push(c);
                    self.bump();
                }
            }
        }
        flush(&mut text, &mut parts);
        Ok(Pattern(parts))
    }

    fn placeholder(&mut self) -> Result<Part, ParseError> {
        self.expect("{", "`{`")?;
        self.skip_o();
        let kind = if self.eat("#") {
            Some(MarkupKind::Open)
        } else if self.eat("/") {
            Some(MarkupKind::Close)
        } else {
            None
        };
        let Some(mut kind) = kind else {
            return Ok(Part::Expression(self.expression_body()?));
        };
        let name = self.identifier()?;
        let options = self.options()?;
        let attributes = self.attributes()?;
        self.skip_o();
        if self.eat("/") {
            if kind == MarkupKind::Close {
                return self.fail("a closing tag cannot also be self-closing");
            }
            kind = MarkupKind::Standalone;
        }
        self.expect("}", "`}` to close the markup")?;
        Ok(Part::Markup(Markup {
            kind,
            name,
            options,
            attributes,
        }))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.expect("{", "`{`")?;
        self.skip_o();
        self.expression_body()
    }

    fn expression_body(&mut self) -> Result<Expression, ParseError> {
        let operand = match self.peek() {
            Some('$') => Some(Operand::Variable(self.variable()?)),
            Some(':') => None,
            Some(_) => Some(Operand::Literal(self.literal()?)),
            None => return self.fail("unterminated expression"),
        };
        let function = if operand.is_some() {
            let save = self.pos;
            if self.skip_s() && self.starts_with(":") {
                Some(self.function()?)
            } else {
                self.pos = save;
                None
            }
        } else {
            Some(self.function()?)
        };
        let attributes = self.attributes()?;
        self.skip_o();
        self.expect("}", "`}` to close the expression")?;
        Ok(Expression {
            operand,
            function,
            attributes,
        })
    }

    fn function(&mut self) -> Result<Function, ParseError> {
        self.expect(":", "`:`")?;
        let name = self.identifier()?;
        let options = self.options()?;
        Ok(Function { name, options })
    }

    fn options(&mut self) -> Result<Vec<Opt>, ParseError> {
        let mut options = Vec::new();
        loop {
            let save = self.pos;
            if !self.skip_s()
                || self.starts_with("@")
                || self.starts_with("}")
                || self.starts_with("/")
            {
                self.pos = save;
                break;
            }
            let name = self.identifier()?;
            self.skip_o();
            self.expect("=", "`=` after the option name")?;
            self.skip_o();
            let value = if self.starts_with("$") {
                OptValue::Variable(self.variable()?)
            } else {
                OptValue::Literal(self.literal()?)
            };
            options.push(Opt { name, value });
        }
        Ok(options)
    }

    fn attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        let mut attributes = Vec::new();
        loop {
            let save = self.pos;
            if !self.skip_s() || !self.eat("@") {
                self.pos = save;
                break;
            }
            let name = self.identifier()?;
            let save = self.pos;
            self.skip_o();
            let value = if self.eat("=") {
                self.skip_o();
                Some(self.literal()?)
            } else {
                self.pos = save;
                None
            };
            attributes.push(Attribute { name, value });
        }
        Ok(attributes)
    }

    fn variable(&mut self) -> Result<String, ParseError> {
        self.expect("$", "`$`")?;
        self.name()
    }

    fn name(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        if !self.peek().is_some_and(is_name_start) {
            return self.fail("expected a name");
        }
        while self.peek().is_some_and(is_name_char) {
            self.bump();
        }
        Ok(self
            .src
            .get(start..self.pos)
            .unwrap_or_default()
            .to_string())
    }

    fn identifier(&mut self) -> Result<Identifier, ParseError> {
        let first = self.name()?;
        if self.starts_with(":") && self.rest().chars().nth(1).is_some_and(is_name_start) {
            self.bump();
            let name = self.name()?;
            return Ok(Identifier {
                namespace: Some(first),
                name,
            });
        }
        Ok(Identifier {
            namespace: None,
            name: first,
        })
    }

    fn literal(&mut self) -> Result<Literal, ParseError> {
        if self.eat("|") {
            let mut value = String::new();
            loop {
                match self.bump() {
                    None => return self.fail("unterminated quoted literal"),
                    Some('|') => break,
                    Some('\\') => match self.bump() {
                        Some(c @ ('\\' | '{' | '|' | '}')) => value.push(c),
                        _ => return self.fail("invalid escape in a quoted literal"),
                    },
                    Some('\0') => return self.fail("NUL is not allowed in a literal"),
                    Some(c) => value.push(c),
                }
            }
            return Ok(Literal {
                value,
                quoted: true,
            });
        }
        let start = self.pos;
        if self.peek().is_some_and(is_name_start) {
            while self.peek().is_some_and(is_name_char) {
                self.bump();
            }
        } else {
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
            {
                self.bump();
            }
            let text = self.src.get(start..self.pos).unwrap_or_default();
            if !is_number_literal(text) {
                self.pos = start;
                return self.fail("expected a literal");
            }
        }
        Ok(Literal {
            value: self
                .src
                .get(start..self.pos)
                .unwrap_or_default()
                .to_string(),
            quoted: false,
        })
    }
}
