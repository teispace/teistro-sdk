//! The data-model checks the specification requires beyond the grammar:
//! a parsed message that fails one is not a valid message.

use std::collections::HashSet;

use super::ast::{
    Attribute, Body, Declaration, Expression, Key, Message, Operand, Opt, Part, Pattern,
};

/// Checks a parsed message; the error is the specification's name for the
/// rule broken, with the detail.
///
/// # Errors
///
/// The first data-model error found.
pub fn check(message: &Message) -> Result<(), String> {
    for expression in message.expressions() {
        check_expression(expression)?;
    }
    for pattern in message.patterns() {
        check_pattern(pattern)?;
    }
    let Message::Complex(complex) = message else {
        return Ok(());
    };
    let mut declared: HashSet<&str> = HashSet::new();
    let mut annotated: HashSet<&str> = HashSet::new();
    let mut referenced: HashSet<&str> = HashSet::new();
    for declaration in &complex.declarations {
        let (variable, expression, is_input) = match declaration {
            Declaration::Input {
                variable,
                expression,
            } => (variable.as_str(), expression, true),
            Declaration::Local {
                variable,
                expression,
            } => (variable.as_str(), expression, false),
        };
        if !declared.insert(variable) {
            return Err(format!(
                "duplicate-declaration: `${variable}` is declared twice"
            ));
        }
        if !is_input && referenced.contains(variable) {
            return Err(format!(
                "duplicate-declaration: `.local ${variable}` follows a reference to `${variable}`"
            ));
        }
        let operand_annotated = match &expression.operand {
            Some(Operand::Variable(v)) => {
                if !is_input {
                    referenced.insert(v.as_str());
                }
                annotated.contains(v.as_str())
            }
            Some(Operand::Literal(_)) | None => false,
        };
        for option in expression.function.iter().flat_map(|f| f.options.iter()) {
            if let super::ast::OptValue::Variable(v) = &option.value {
                referenced.insert(v.as_str());
            }
        }
        if expression.function.is_some() || operand_annotated {
            annotated.insert(variable);
        }
    }
    let Body::Matcher(matcher) = &complex.body else {
        return Ok(());
    };
    for selector in &matcher.selectors {
        if !annotated.contains(selector.as_str()) {
            return Err(format!(
                "missing-selector-annotation: `${selector}` is not declared with a function"
            ));
        }
    }
    let mut seen: Vec<&[Key]> = Vec::new();
    for variant in &matcher.variants {
        if variant.keys.len() != matcher.selectors.len() {
            return Err(format!(
                "variant-key-mismatch: {} keys for {} selectors",
                variant.keys.len(),
                matcher.selectors.len()
            ));
        }
        if seen.contains(&variant.keys.as_slice()) {
            return Err(String::from(
                "duplicate-variant: the same keys appear twice",
            ));
        }
        seen.push(&variant.keys);
    }
    if !matcher
        .variants
        .iter()
        .any(|v| v.keys.iter().all(|k| *k == Key::Wildcard))
    {
        return Err(String::from(
            "missing-fallback-variant: no variant has `*` for every selector",
        ));
    }
    Ok(())
}

fn check_expression(expression: &Expression) -> Result<(), String> {
    if let Some(function) = &expression.function {
        check_options(&function.options)?;
    }
    check_attributes(&expression.attributes)
}

fn check_pattern(pattern: &Pattern) -> Result<(), String> {
    for part in &pattern.0 {
        if let Part::Markup(markup) = part {
            check_options(&markup.options)?;
            check_attributes(&markup.attributes)?;
        }
    }
    Ok(())
}

fn check_options(options: &[Opt]) -> Result<(), String> {
    let mut names = HashSet::new();
    for option in options {
        if !names.insert((&option.name.namespace, &option.name.name)) {
            return Err(format!(
                "duplicate-option-name: `{}` is given twice",
                option.name.name
            ));
        }
    }
    Ok(())
}

fn check_attributes(attributes: &[Attribute]) -> Result<(), String> {
    let mut names = HashSet::new();
    for attribute in attributes {
        if !names.insert((&attribute.name.namespace, &attribute.name.name)) {
            return Err(format!(
                "duplicate-attribute: `@{}` is given twice",
                attribute.name.name
            ));
        }
    }
    Ok(())
}
