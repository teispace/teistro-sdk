//! CLDR's rule-based number formats, for integers: the rules parsed from
//! a file's `rbnfRules` text, and an interpreter of them
//! (`docs/03-design/amounts-in-words.md` §1, after UTS #35 Part 3 §6,
//! "Rule-Based Number Formatting").
//!
//! `cargo xtask gen intl` keeps of each CLDR file the text of the sets a
//! cardinal reaches ([`reached_text`]), and the engine parses that text
//! once, on first use: one parser for the generator's check and the
//! engine, so the tables the generator accepted are the ones spelt from.
//!
//! ```
//! use teistro_intl::words::rbnf::{parse, spell};
//!
//! let sets = parse("%spellout:\n0: zero;\n1: one;\n2: two;\n10: ten[ and >>];")?;
//! assert_eq!(spell(&sets, 0, 12, &['0'; 10])?, "ten and two");
//! assert_eq!(spell(&sets, 0, 10, &['0'; 10])?, "ten");
//! # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
//! ```

use std::collections::VecDeque;
use std::fmt;

/// A rule set: its name, its rule for a negative number, and its rules in
/// ascending base order.
///
/// ```
/// use teistro_intl::words::rbnf::parse;
///
/// let sets = parse("%spellout:\n-x: minus >>;\n0: nought;")?;
/// assert_eq!(sets[0].name, "spellout");
/// assert!(sets[0].negative.is_some());
/// assert_eq!(sets[0].rules.len(), 1);
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSet {
    /// The name without its `%` or `%%`.
    pub name: String,
    /// Whether the name began `%%`: a set only other sets name.
    pub private: bool,
    /// The `-x` rule's body, if the set has one.
    pub negative: Option<Vec<Part>>,
    /// The rules for a number not below zero, ascending by base.
    pub rules: Vec<Rule>,
}

/// One rule: the base value it starts at, the divisor its substitutions
/// divide by, and its body.
///
/// ```
/// use teistro_intl::words::rbnf::parse;
///
/// let sets = parse("%s:\n0: x;\n1000/100: << hundred[ >>];")?;
/// // A radix of 100 makes 1000's divisor 100, not 1000.
/// assert_eq!(sets[0].rules[1].divisor, 100);
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// The smallest number the rule applies to.
    pub base: u64,
    /// The highest power of the radix not above `base` (1 for a base
    /// below the radix).
    pub divisor: u64,
    /// The text and substitutions.
    pub body: Vec<Part>,
}

/// A piece of a rule's body.
///
/// ```
/// use teistro_intl::words::rbnf::{Part, Target, parse};
///
/// let sets = parse("%s:\n0: x;\n20: twenty[->>];")?;
/// assert_eq!(sets[0].rules[1].body[0], Part::Text("twenty".into()));
/// assert!(matches!(sets[0].rules[1].body[1], Part::Optional(_)));
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// Literal text.
    Text(String),
    /// `<<`: the number divided by the divisor.
    Quotient(Target),
    /// `>>`: the remainder after dividing by the divisor; in a negative
    /// rule, the number's magnitude.
    Remainder(Target),
    /// `=…=`: the number itself.
    Same(Target),
    /// `[…]`: left out when the remainder is zero.
    Optional(Vec<Part>),
}

/// Where a substitution's number is written.
///
/// ```
/// use teistro_intl::words::rbnf::{Part, Target, parse};
///
/// let sets = parse("%s:\n0: =#,##,##0=;")?;
/// assert_eq!(sets[0].rules[0].body[0], Part::Same(Target::Digits { primary: 3, secondary: 2 }));
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The rule's own set.
    Own,
    /// Another set, by its index in the parsed list.
    Set(usize),
    /// A decimal pattern: digits grouped by `primary` and then
    /// `secondary` (`#,##,##0` is 3 then 2).
    Digits {
        /// The first group from the right.
        primary: u8,
        /// Every group after it.
        secondary: u8,
    },
}

/// Why rules did not parse or a number could not be spelt.
///
/// ```
/// use teistro_intl::words::rbnf::parse;
///
/// let refused = parse("%s:\n0: =%nowhere=;").unwrap_err();
/// assert!(refused.to_string().contains("nowhere"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RbnfError(pub String);

impl fmt::Display for RbnfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RbnfError {}

/// The descriptors of rules for what is not an integer, which an integer
/// speller never reaches and so does not read.
const NOT_INTEGER: [&str; 5] = ["x.x", "0.x", "x.0", "Inf", "NaN"];

/// The deepest a substitution may nest: CLDR's own sets nest a handful of
/// levels, so this only stops a set that names itself in a loop.
const DEPTH: u8 = 32;

/// Parses a file's `rbnfRules` text: `%name:` starting each set, each
/// rule `descriptor: body;`.
///
/// ```
/// use teistro_intl::words::rbnf::parse;
///
/// let sets = parse("%a:\n0: =%%b=;\n%%b:\n0: b;")?;
/// assert_eq!(sets.len(), 2);
/// assert!(sets[1].private);
/// // A rule for fractions is not an integer's, and is left out.
/// assert_eq!(parse("%a:\nx.x: << point >>;\n0: z;")?[0].rules.len(), 1);
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
///
/// # Errors
///
/// A rule before any set, a descriptor or a substitution the interpreter
/// does not read (`>>>`, `$(…)$`, an exponent marker), a set named and
/// not defined, or bases out of order.
pub fn parse(text: &str) -> Result<Vec<RuleSet>, RbnfError> {
    let raw = raw_sets(text)?;
    let names: Vec<&str> = raw.iter().map(|set| set.name.as_str()).collect();
    raw.iter().map(|set| set.resolved(&names)).collect()
}

/// The text of the sets `roots` name and every set they reach, in the
/// order first reached, each rule exactly as the source wrote it: what
/// the generator keeps of a CLDR file, which names sets an integer
/// speller never reaches (ordinals, years) in syntax it does not read.
///
/// ```
/// use teistro_intl::words::rbnf::{parse, reached_text};
///
/// let file = "%year:\n0: =0.0=;\n%cardinal:\n0: =%%digit=;\n%%digit:\n0: nought;";
/// let kept = reached_text(file, &["cardinal"])?;
/// assert_eq!(kept, "%cardinal:\n0: =%%digit=;\n%%digit:\n0: nought;\n");
/// assert_eq!(parse(&kept)?.len(), 2);
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
///
/// # Errors
///
/// A root the text does not define, or a reached set [`parse`] refuses.
pub fn reached_text(text: &str, roots: &[&str]) -> Result<String, RbnfError> {
    let raw = raw_sets(text)?;
    let names: Vec<&str> = raw.iter().map(|set| set.name.as_str()).collect();
    let mut queue = roots
        .iter()
        .map(|root| {
            names
                .iter()
                .position(|name| name == root)
                .ok_or_else(|| RbnfError(format!("no set `{root}`")))
        })
        .collect::<Result<VecDeque<usize>, RbnfError>>()?;
    let mut seen: Vec<usize> = Vec::new();
    while let Some(next) = queue.pop_front() {
        if seen.contains(&next) {
            continue;
        }
        seen.push(next);
        let Some(set) = raw.get(next) else {
            continue;
        };
        let set = set.resolved(&names)?;
        let mut found = Vec::new();
        for parts in set
            .negative
            .iter()
            .chain(set.rules.iter().map(|rule| &rule.body))
        {
            names_in(parts, &mut found);
        }
        queue.extend(found);
    }
    let mut out = String::new();
    for set in seen.into_iter().filter_map(|index| raw.get(index)) {
        out.push_str(if set.private { "%%" } else { "%" });
        out.push_str(&set.name);
        out.push_str(":\n");
        for rule in &set.source {
            out.push_str(rule);
            out.push_str(";\n");
        }
    }
    Ok(out)
}

fn names_in(parts: &[Part], named: &mut Vec<usize>) {
    for part in parts {
        match part {
            Part::Quotient(Target::Set(index))
            | Part::Remainder(Target::Set(index))
            | Part::Same(Target::Set(index)) => named.push(*index),
            Part::Optional(inner) => names_in(inner, named),
            Part::Text(_) | Part::Quotient(_) | Part::Remainder(_) | Part::Same(_) => {}
        }
    }
}

/// A set as written, its bodies not yet parsed.
struct Raw {
    name: String,
    private: bool,
    /// Every rule as the source wrote it, trimmed, without its `;`.
    source: Vec<String>,
    negative: Option<String>,
    rules: Vec<(u64, u64, String)>,
}

impl Raw {
    fn resolved(&self, names: &[&str]) -> Result<RuleSet, RbnfError> {
        Ok(RuleSet {
            name: self.name.clone(),
            private: self.private,
            negative: self
                .negative
                .as_deref()
                .map(|body| parts(body, names))
                .transpose()?,
            rules: self
                .rules
                .iter()
                .map(|(base, divisor, body)| {
                    Ok(Rule {
                        base: *base,
                        divisor: *divisor,
                        body: parts(body, names)?,
                    })
                })
                .collect::<Result<_, RbnfError>>()?,
        })
    }
}

fn raw_sets(text: &str) -> Result<Vec<Raw>, RbnfError> {
    let mut raw: Vec<Raw> = Vec::new();
    for piece in text.split(';') {
        let mut rest = piece.trim();
        if rest.is_empty() {
            continue;
        }
        if let Some(named) = rest.strip_prefix('%') {
            let (name, after) = named
                .split_once(':')
                .ok_or_else(|| RbnfError(format!("a set name without a colon: `{rest}`")))?;
            let (private, name) = match name.strip_prefix('%') {
                Some(inner) => (true, inner),
                None => (false, name),
            };
            raw.push(Raw {
                name: name.trim().to_owned(),
                private,
                source: Vec::new(),
                negative: None,
                rules: Vec::new(),
            });
            rest = after.trim();
        }
        let set = raw
            .last_mut()
            .ok_or_else(|| RbnfError(format!("a rule before any set: `{rest}`")))?;
        set.source.push(rest.to_owned());
        let (descriptor, body) = rest
            .split_once(':')
            .ok_or_else(|| RbnfError(format!("a rule without a descriptor: `{rest}`")))?;
        let descriptor = descriptor.trim();
        let body = body.trim_start();
        let body = body.strip_prefix('\'').unwrap_or(body).to_owned();
        if descriptor == "-x" {
            set.negative = Some(body);
        } else if !NOT_INTEGER.contains(&descriptor) {
            let (base, radix) = match descriptor.split_once('/') {
                Some((base, radix)) => (number(base)?, number(radix)?),
                None => (number(descriptor)?, 10),
            };
            if set.rules.last().is_some_and(|(last, _, _)| *last >= base) {
                return Err(RbnfError(format!(
                    "`{}`'s rule {base} is not above the one before it",
                    set.name
                )));
            }
            set.rules.push((base, divisor(base, radix)?, body));
        }
    }
    Ok(raw)
}

fn number(text: &str) -> Result<u64, RbnfError> {
    text.trim()
        .parse()
        .map_err(|_| RbnfError(format!("a descriptor this does not read: `{text}`")))
}

/// The highest power of `radix` not above `base`.
fn divisor(base: u64, radix: u64) -> Result<u64, RbnfError> {
    if radix < 2 {
        return Err(RbnfError(format!("a radix below 2: {radix}")));
    }
    let mut power = 1u64;
    while let Some(next) = power.checked_mul(radix) {
        if next > base {
            break;
        }
        power = next;
    }
    Ok(power)
}

/// A body's parts, the sets it names resolved to their indices.
fn parts(body: &str, names: &[&str]) -> Result<Vec<Part>, RbnfError> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '<' | '>' | '=' => {
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some(end) if end == c => break,
                        Some(other) => inner.push(other),
                        None => {
                            return Err(RbnfError(format!(
                                "an unclosed `{c}` substitution in `{body}`"
                            )));
                        }
                    }
                }
                if c == '>' && chars.as_str().starts_with('>') {
                    return Err(RbnfError(format!("`>>>` is not read, in `{body}`")));
                }
                if !text.is_empty() {
                    out.push(Part::Text(core::mem::take(&mut text)));
                }
                let target = target(&inner, names, body)?;
                out.push(match c {
                    '<' => Part::Quotient(target),
                    '>' => Part::Remainder(target),
                    _ => Part::Same(target),
                });
            }
            '[' => {
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some('[') => {
                            return Err(RbnfError(format!("a nested `[` in `{body}`")));
                        }
                        Some(other) => inner.push(other),
                        None => return Err(RbnfError(format!("an unclosed `[` in `{body}`"))),
                    }
                }
                if !text.is_empty() {
                    out.push(Part::Text(core::mem::take(&mut text)));
                }
                out.push(Part::Optional(parts(&inner, names)?));
            }
            '$' => {
                return Err(RbnfError(format!(
                    "a plural `$(…)$` is not read, in `{body}`"
                )));
            }
            other => text.push(other),
        }
    }
    if !text.is_empty() {
        out.push(Part::Text(text));
    }
    Ok(out)
}

fn target(inner: &str, names: &[&str], body: &str) -> Result<Target, RbnfError> {
    if inner.is_empty() {
        return Ok(Target::Own);
    }
    if let Some(name) = inner.strip_prefix('%') {
        let name = name.strip_prefix('%').unwrap_or(name);
        return names
            .iter()
            .position(|known| *known == name)
            .map(Target::Set)
            .ok_or_else(|| {
                RbnfError(format!(
                    "`{body}` names the set `{name}`, which is not defined"
                ))
            });
    }
    if inner.chars().all(|c| matches!(c, '#' | ',' | '0')) && inner.ends_with('0') {
        let groups: Vec<usize> = inner.split(',').map(str::len).collect();
        let size = |n: usize| {
            u8::try_from(n).map_err(|_| RbnfError(format!("a group too wide in `{inner}`")))
        };
        return match groups.as_slice() {
            [] | [_] => Ok(Target::Digits {
                primary: 0,
                secondary: 0,
            }),
            [_, last] => Ok(Target::Digits {
                primary: size(*last)?,
                secondary: size(*last)?,
            }),
            [.., before, last] => Ok(Target::Digits {
                primary: size(*last)?,
                secondary: size(*before)?,
            }),
        };
    }
    Err(RbnfError(format!(
        "a substitution this does not read: `{inner}` in `{body}`"
    )))
}

/// Spells `n` with the set at `index`, writing a decimal pattern's digits
/// in `digits`.
///
/// ```
/// use teistro_intl::words::rbnf::{parse, spell};
///
/// let sets = parse("%s:\n-x: minus >>;\n0: zero;\n1: one;\n2: =#,##0=;")?;
/// let latin = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
/// assert_eq!(spell(&sets, 0, -1, &latin)?, "minus one");
/// assert_eq!(spell(&sets, 0, 1234, &latin)?, "1,234");
/// # Ok::<(), teistro_intl::words::rbnf::RbnfError>(())
/// ```
///
/// # Errors
///
/// A set with no rule for the number, or substitutions nesting past
/// their limit (a set that names itself in a loop).
pub fn spell(
    sets: &[RuleSet],
    index: usize,
    n: i128,
    digits: &[char; 10],
) -> Result<String, RbnfError> {
    let mut out = String::new();
    write(sets, index, n, digits, &mut out, 0)?;
    Ok(out)
}

fn write(
    sets: &[RuleSet],
    index: usize,
    n: i128,
    digits: &[char; 10],
    out: &mut String,
    depth: u8,
) -> Result<(), RbnfError> {
    if depth > DEPTH {
        return Err(RbnfError(String::from(
            "substitutions nest past their limit",
        )));
    }
    let set = sets
        .get(index)
        .ok_or_else(|| RbnfError(format!("no set {index}")))?;
    if n < 0 {
        if let Some(negative) = &set.negative {
            // `>>` in a negative rule is the number's magnitude.
            let magnitude = n
                .checked_abs()
                .ok_or_else(|| RbnfError(format!("{n} has no magnitude in range")))?;
            return body(
                sets, index, negative, magnitude, magnitude, magnitude, digits, out, depth,
            );
        }
    }
    let magnitude = n.unsigned_abs();
    let position = set
        .rules
        .iter()
        .rposition(|rule| u128::from(rule.base) <= magnitude)
        .ok_or_else(|| RbnfError(format!("`{}` has no rule for {n}", set.name)))?;
    let mut rule = set.rules.get(position);
    // ICU's roll-back: a rule whose base is not a multiple of its divisor
    // gives way to the one before it for a number that is.
    let rolls_back = rule.is_some_and(|found| {
        found.base % found.divisor != 0
            && magnitude % u128::from(found.divisor) == 0
            && has_remainder(&found.body)
    });
    if rolls_back {
        rule = position
            .checked_sub(1)
            .and_then(|before| set.rules.get(before))
            .or(rule);
    }
    let rule = rule.ok_or_else(|| RbnfError(format!("`{}` has no rule for {n}", set.name)))?;
    let divisor = i128::from(rule.divisor);
    body(
        sets,
        index,
        &rule.body,
        n / divisor,
        n % divisor,
        n,
        digits,
        out,
        depth,
    )
}

fn has_remainder(parts: &[Part]) -> bool {
    parts.iter().any(|part| match part {
        Part::Remainder(_) => true,
        Part::Optional(inner) => has_remainder(inner),
        _ => false,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "one rule's whole state, passed down once"
)]
fn body(
    sets: &[RuleSet],
    index: usize,
    parts: &[Part],
    quotient: i128,
    remainder: i128,
    n: i128,
    digits: &[char; 10],
    out: &mut String,
    depth: u8,
) -> Result<(), RbnfError> {
    for part in parts {
        let (target, value) = match part {
            Part::Text(text) => {
                out.push_str(text);
                continue;
            }
            Part::Optional(inner) => {
                if remainder != 0 {
                    body(
                        sets, index, inner, quotient, remainder, n, digits, out, depth,
                    )?;
                }
                continue;
            }
            Part::Quotient(target) => (target, quotient),
            Part::Remainder(target) => (target, remainder),
            Part::Same(target) => (target, n),
        };
        match *target {
            Target::Own => write(sets, index, value, digits, out, depth + 1)?,
            Target::Set(other) => write(sets, other, value, digits, out, depth + 1)?,
            Target::Digits { primary, secondary } => {
                out.push_str(&grouped(value, primary, secondary, digits));
            }
        }
    }
    Ok(())
}

/// The digit `c` (an ASCII digit) in `digits`; anything else as it is.
pub(crate) fn digit(c: char, digits: &[char; 10]) -> char {
    c.to_digit(10)
        .and_then(|d| digits.get(d as usize).copied())
        .unwrap_or(c)
}

/// An integer in `digits`, grouped by `primary` from the right and then
/// by `secondary`, with `,` between groups; ungrouped where `primary` is
/// zero.
///
/// ```
/// use teistro_intl::words::rbnf::grouped;
///
/// let latin = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
/// assert_eq!(grouped(1234567, 3, 2, &latin), "12,34,567");
/// assert_eq!(grouped(1234567, 3, 3, &latin), "1,234,567");
/// assert_eq!(grouped(-1234, 0, 0, &latin), "-1234");
/// ```
#[must_use]
pub fn grouped(n: i128, primary: u8, secondary: u8, digits: &[char; 10]) -> String {
    let ascii = n.unsigned_abs().to_string();
    let mut reversed: Vec<char> = Vec::with_capacity(ascii.len() * 2);
    let (mut size, mut in_group) = (usize::from(primary), 0usize);
    for c in ascii.chars().rev() {
        if size > 0 && in_group == size {
            reversed.push(',');
            in_group = 0;
            size = usize::from(secondary);
        }
        reversed.push(digit(c, digits));
        in_group += 1;
    }
    let mut out = String::with_capacity(reversed.len() + 1);
    if n < 0 {
        out.push('-');
    }
    out.extend(reversed.iter().rev());
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking"
    )]

    use super::*;

    const LATIN: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    #[test]
    fn every_malformed_rule_is_refused_by_what_is_wrong() {
        for (text, says) in [
            ("0: x;", "before any set"),
            ("%s\n0 x;", "without a colon"),
            ("%s:\nzero x;", "without a descriptor"),
            ("%s:\nten: x;", "does not read"),
            ("%s:\n10/1: x;", "radix below 2"),
            ("%s:\n10: x;\n5: y;", "not above"),
            ("%s:\n0: <a;", "unclosed"),
            ("%s:\n0: >>>;", "`>>>`"),
            ("%s:\n0: [a[b]];", "nested"),
            ("%s:\n0: [a;", "unclosed `[`"),
            ("%s:\n0: =#,##0=$(ordinal,one{st})$;", "plural"),
            ("%s:\n0: =0.0=;", "does not read"),
            ("%s:\n0: =%missing=;", "not defined"),
        ] {
            let refused = parse(text).unwrap_err();
            assert!(refused.to_string().contains(says), "{text}: {refused}");
        }
    }

    #[test]
    fn a_leading_apostrophe_keeps_the_space_after_it() {
        let sets = parse("%s:\n0: =%%and=;\n%%and:\n0: ' and;").unwrap();
        assert_eq!(spell(&sets, 0, 0, &LATIN).unwrap(), " and");
    }

    #[test]
    fn a_negative_without_its_rule_keeps_its_sign_for_the_set_it_names() {
        let sets = parse("%a:\n0: =%b=;\n%b:\n-x: minus >>;\n0: zero;\n1: one;").unwrap();
        assert_eq!(spell(&sets, 0, -1, &LATIN).unwrap(), "minus one");
    }

    #[test]
    fn roll_back_takes_the_rule_before_for_an_even_multiple() {
        // 100000/1000's divisor is 1000, a multiple the base is; 1500/1000 is not.
        let sets = parse("%s:\n0: n;\n1000: << k[ >>];\n1500/1000: << q[ >>];").unwrap();
        assert_eq!(spell(&sets, 0, 2000, &LATIN).unwrap(), "n k");
        assert_eq!(spell(&sets, 0, 2001, &LATIN).unwrap(), "n q n");
    }

    #[test]
    fn a_number_below_every_rule_and_a_loop_are_refused() {
        let sets = parse("%s:\n5: five;").unwrap();
        assert!(spell(&sets, 0, 1, &LATIN).is_err());
        let looped = parse("%s:\n0: =%s=;").unwrap();
        assert!(
            spell(&looped, 0, 1, &LATIN)
                .unwrap_err()
                .to_string()
                .contains("limit")
        );
        assert!(spell(&looped, 3, 1, &LATIN).is_err());
    }

    #[test]
    fn digits_group_and_a_pattern_reads_its_group_sizes() {
        assert_eq!(grouped(0, 3, 2, &LATIN), "0");
        assert_eq!(grouped(100_000, 3, 2, &LATIN), "1,00,000");
        let sets = parse("%s:\n0: =#,##0=;\n10: =0=;").unwrap();
        assert_eq!(
            sets[0].rules[0].body[0],
            Part::Same(Target::Digits {
                primary: 3,
                secondary: 3
            })
        );
        assert_eq!(
            sets[0].rules[1].body[0],
            Part::Same(Target::Digits {
                primary: 0,
                secondary: 0
            })
        );
        assert!(reached_text("%s:\n0: z;", &["t"]).is_err());
    }
}
