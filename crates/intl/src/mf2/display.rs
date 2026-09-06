//! Serialisation back to source. `parse(message.to_string())` yields the
//! same tree; a simple pattern that would not parse back as simple is
//! written as a quoted pattern.

use core::fmt::{self, Write};

use super::ast::{
    Attribute, Body, Declaration, Expression, Function, Identifier, Key, Literal, Markup,
    MarkupKind, Message, Operand, Opt, OptValue, Part, Pattern,
};
use super::parser::{is_name, is_number_literal};

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Simple(pattern) => {
                let text = pattern_source(pattern);
                if text.starts_with(|c: char| c == '.' || c.is_whitespace()) || text.is_empty() {
                    write!(f, "{{{{{text}}}}}")
                } else {
                    f.write_str(&text)
                }
            }
            Message::Complex(complex) => {
                for declaration in &complex.declarations {
                    match declaration {
                        Declaration::Input { expression, .. } => {
                            write!(f, ".input {expression} ")?;
                        }
                        Declaration::Local {
                            variable,
                            expression,
                        } => write!(f, ".local ${variable} = {expression} ")?,
                    }
                }
                match &complex.body {
                    Body::Pattern(pattern) => write!(f, "{{{{{}}}}}", pattern_source(pattern)),
                    Body::Matcher(matcher) => {
                        f.write_str(".match")?;
                        for selector in &matcher.selectors {
                            write!(f, " ${selector}")?;
                        }
                        for variant in &matcher.variants {
                            f.write_char('\n')?;
                            for (i, key) in variant.keys.iter().enumerate() {
                                if i > 0 {
                                    f.write_char(' ')?;
                                }
                                write!(f, "{key}")?;
                            }
                            write!(f, " {{{{{}}}}}", pattern_source(&variant.pattern))?;
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}

/// A pattern as source text, escaped.
#[must_use]
pub fn pattern_source(pattern: &Pattern) -> String {
    let mut out = String::new();
    for part in &pattern.0 {
        match part {
            Part::Text(text) => {
                for c in text.chars() {
                    match c {
                        '\\' | '{' | '}' => {
                            out.push('\\');
                            out.push(c);
                        }
                        _ => out.push(c),
                    }
                }
            }
            Part::Expression(expression) => {
                let _ = write!(out, "{expression}");
            }
            Part::Markup(markup) => {
                let _ = write!(out, "{markup}");
            }
        }
    }
    out
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Literal(literal) => write!(f, "{literal}"),
            Key::Wildcard => f.write_char('*'),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.quoted && (is_name(&self.value) || is_number_literal(&self.value)) {
            return f.write_str(&self.value);
        }
        f.write_char('|')?;
        for c in self.value.chars() {
            if matches!(c, '\\' | '|') {
                f.write_char('\\')?;
            }
            f.write_char(c)?;
        }
        f.write_char('|')
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespace {
            Some(namespace) => write!(f, "{namespace}:{}", self.name),
            None => f.write_str(&self.name),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('{')?;
        match &self.operand {
            Some(Operand::Literal(literal)) => write!(f, "{literal}")?,
            Some(Operand::Variable(variable)) => write!(f, "${variable}")?,
            None => {}
        }
        if let Some(function) = &self.function {
            if self.operand.is_some() {
                f.write_char(' ')?;
            }
            write!(f, "{function}")?;
        }
        write_attributes(f, &self.attributes)?;
        f.write_char('}')
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.name)?;
        write_options(f, &self.options)
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sigil = if self.kind == MarkupKind::Close {
            '/'
        } else {
            '#'
        };
        write!(f, "{{{sigil}{}", self.name)?;
        write_options(f, &self.options)?;
        write_attributes(f, &self.attributes)?;
        if self.kind == MarkupKind::Standalone {
            f.write_str(" /")?;
        }
        f.write_char('}')
    }
}

fn write_options(f: &mut fmt::Formatter<'_>, options: &[Opt]) -> fmt::Result {
    for option in options {
        write!(f, " {}=", option.name)?;
        match &option.value {
            OptValue::Literal(literal) => write!(f, "{literal}")?,
            OptValue::Variable(variable) => write!(f, "${variable}")?,
        }
    }
    Ok(())
}

fn write_attributes(f: &mut fmt::Formatter<'_>, attributes: &[Attribute]) -> fmt::Result {
    for attribute in attributes {
        write!(f, " @{}", attribute.name)?;
        if let Some(value) = &attribute.value {
            write!(f, "={value}")?;
        }
    }
    Ok(())
}
