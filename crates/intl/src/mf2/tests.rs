#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use proptest::prelude::*;

use super::ast::{
    Attribute, Body, Complex, Declaration, Expression, Function, Identifier, Key, Literal, Markup,
    MarkupKind, Matcher, Message, Operand, Opt, OptValue, Part, Pattern, Variant,
};
use super::parser::{ParseError, is_name, is_number_literal, parse};

fn text(s: &str) -> Part {
    Part::Text(s.to_string())
}

fn var(name: &str) -> Part {
    Part::Expression(Expression {
        operand: Some(Operand::Variable(name.to_string())),
        function: None,
        attributes: Vec::new(),
    })
}

fn unquoted(value: &str) -> Literal {
    Literal {
        value: value.to_string(),
        quoted: false,
    }
}

fn parsed(source: &str) -> Message {
    parse(source).unwrap_or_else(|e| panic!("{source:?}: {e}"))
}

fn failure(source: &str) -> ParseError {
    match parse(source) {
        Ok(message) => panic!("{source:?} parsed as {message:?}"),
        Err(error) => error,
    }
}

#[test]
fn simple_text_with_escapes_and_placeholders() {
    assert_eq!(
        parsed("Hello, {$name}! \\{not a placeholder\\} \\\\ and \\| pipe"),
        Message::Simple(Pattern(vec![
            text("Hello, "),
            var("name"),
            text("! {not a placeholder} \\ and | pipe"),
        ]))
    );
    assert_eq!(parsed(""), Message::Simple(Pattern(Vec::new())));
    assert_eq!(
        parsed("   padded"),
        Message::Simple(Pattern(vec![text("padded")]))
    );
    assert_eq!(
        parsed("trailing   "),
        Message::Simple(Pattern(vec![text("trailing   ")]))
    );
}

#[test]
fn expressions_in_every_form() {
    let message = parsed(
        "{$n :integer select=ordinal minimumFractionDigits=$digits @locale=en} {|quoted \\| bar|} {sdk.reason.appName :msg} {:now @literal}",
    );
    let Message::Simple(Pattern(parts)) = message else {
        panic!("simple expected");
    };
    assert_eq!(
        parts.first(),
        Some(&Part::Expression(Expression {
            operand: Some(Operand::Variable("n".into())),
            function: Some(Function {
                name: Identifier::plain("integer"),
                options: vec![
                    Opt {
                        name: Identifier::plain("select"),
                        value: OptValue::Literal(unquoted("ordinal")),
                    },
                    Opt {
                        name: Identifier::plain("minimumFractionDigits"),
                        value: OptValue::Variable("digits".into()),
                    },
                ],
            }),
            attributes: vec![Attribute {
                name: Identifier::plain("locale"),
                value: Some(unquoted("en")),
            }],
        }))
    );
    assert_eq!(
        parts.get(2),
        Some(&Part::Expression(Expression {
            operand: Some(Operand::Literal(Literal {
                value: "quoted | bar".into(),
                quoted: true,
            })),
            function: None,
            attributes: Vec::new(),
        }))
    );
    assert_eq!(
        parts.get(4),
        Some(&Part::Expression(Expression {
            operand: Some(Operand::Literal(unquoted("sdk.reason.appName"))),
            function: Some(Function {
                name: Identifier::plain("msg"),
                options: Vec::new(),
            }),
            attributes: Vec::new(),
        }))
    );
    assert_eq!(
        parts.get(6),
        Some(&Part::Expression(Expression {
            operand: None,
            function: Some(Function {
                name: Identifier::plain("now"),
                options: Vec::new(),
            }),
            attributes: vec![Attribute {
                name: Identifier::plain("literal"),
                value: None,
            }],
        }))
    );
}

#[test]
fn markup_in_every_form() {
    let message = parsed("{#b}bold{/b} {#link href=$url class=x /}{ #i}{ /i }");
    let Message::Simple(Pattern(parts)) = message else {
        panic!("simple expected");
    };
    let kinds: Vec<MarkupKind> = parts
        .iter()
        .filter_map(|p| match p {
            Part::Markup(m) => Some(m.kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds,
        [
            MarkupKind::Open,
            MarkupKind::Close,
            MarkupKind::Standalone,
            MarkupKind::Open,
            MarkupKind::Close
        ]
    );
    let Some(Part::Markup(link)) = parts.get(4) else {
        panic!("link expected: {parts:?}");
    };
    assert_eq!(link.name, Identifier::plain("link"));
    assert_eq!(link.options.len(), 2);
}

#[test]
fn complex_messages_with_declarations_and_matchers() {
    let source = ".input {$count :integer}\n.local $g = {$graha :entity kind=graha}\n.match $count $g\n0 * {{none}}\none SUN {{the Sun}}\n* * {{{$count} with {$g}}}";
    let message = parsed(source);
    let Message::Complex(Complex { declarations, body }) = message else {
        panic!("complex expected");
    };
    assert_eq!(declarations.len(), 2);
    assert!(matches!(&declarations[1], Declaration::Local { variable, .. } if variable == "g"));
    let Body::Matcher(Matcher {
        selectors,
        variants,
    }) = body
    else {
        panic!("matcher expected");
    };
    assert_eq!(selectors, ["count", "g"]);
    assert_eq!(variants.len(), 3);
    assert_eq!(
        variants[0].keys,
        [Key::Literal(unquoted("0")), Key::Wildcard]
    );
    assert_eq!(variants[2].pattern.0.len(), 3);
}

#[test]
fn quoted_patterns_keep_leading_whitespace_and_full_stops() {
    assert_eq!(
        parsed("{{  two spaces}}"),
        Message::Complex(Complex {
            declarations: Vec::new(),
            body: Body::Pattern(Pattern(vec![text("  two spaces")])),
        })
    );
    assert_eq!(
        parsed(" {{.starts with a stop}} "),
        Message::Complex(Complex {
            declarations: Vec::new(),
            body: Body::Pattern(Pattern(vec![text(".starts with a stop")])),
        })
    );
}

#[test]
fn syntax_errors_carry_offsets() {
    assert_eq!(failure("a } b").offset, Some(2));
    assert_eq!(failure("bad \\n escape").offset, Some(6));
    assert!(failure("{$x").message.contains("`}`"));
    assert!(failure("{{unterminated").message.contains("unterminated"));
    assert!(
        failure(".match $x * {{a}}")
            .message
            .contains("missing-selector-annotation")
    );
    assert!(failure(".local $x = {1}").message.contains("expected"));
    assert!(
        failure("{$x :f a=1 a=2}")
            .message
            .contains("duplicate-option-name")
    );
    assert!(
        failure("{$x @a @a}")
            .message
            .contains("duplicate-attribute")
    );
    assert!(failure("text\0nul").message.contains("NUL"));
    assert!(failure("{/b /}").message.contains("self-closing"));
}

#[test]
fn data_model_errors_are_refused() {
    assert!(
        failure(".input {$n :integer} .match $n one {{a}}")
            .message
            .contains("missing-fallback-variant")
    );
    assert!(
        failure(".input {$n :integer} .match $n one two {{a}} * {{b}}")
            .message
            .contains("variant-key-mismatch")
    );
    assert!(
        failure(".input {$n :integer} .match $n one {{a}} one {{b}} * {{c}}")
            .message
            .contains("duplicate-variant")
    );
    assert!(
        failure(".input {$n :integer} .input {$n :number} {{a}}")
            .message
            .contains("duplicate-declaration")
    );
    assert!(
        failure(".local $a = {$b} .local $b = {1} {{a}}")
            .message
            .contains("duplicate-declaration")
    );
    assert!(
        failure(".input {1 :integer} {{a}}")
            .message
            .contains("variable expression")
    );
}

#[test]
fn selector_annotation_passes_through_locals() {
    parsed(".input {$n :integer} .local $m = {$n} .match $m one {{a}} * {{b}}");
}

#[test]
fn names_and_numbers_follow_the_grammar() {
    assert!(is_name("sdk.reason.appName"));
    assert!(is_name("_x-1"));
    assert!(is_name("मेष"));
    assert!(!is_name("1abc"));
    assert!(!is_name(""));
    assert!(is_number_literal("0"));
    assert!(is_number_literal("-12.5e+3"));
    assert!(!is_number_literal("01"));
    assert!(!is_number_literal("1."));
    assert!(!is_number_literal("+1"));
}

const ROUND_TRIPS: [&str; 12] = [
    "plain",
    "Hello, {$name}!",
    "{$n :integer select=ordinal} {|q|} {|a \\| b|} {x :f o=$v}",
    "{#b}x{/b}{#br /}",
    "a \\{ b \\} c \\\\ d",
    ".input {$count :integer}\n.match $count\none {{one}}\n* {{{$count}}}",
    "{{  leading}}",
    "{{.stop}}",
    ".local $x = {|literal| :string}\n{{{$x}}}",
    ".input {$a :string} .input {$b :string} .match $a $b x y {{1}} * y {{2}} * * {{3}}",
    "{$x @a @b=c}",
    "{$graha :entity kind=graha form=prose}",
];

#[test]
fn serialisation_round_trips() {
    for source in ROUND_TRIPS {
        let message = parsed(source);
        let again = parsed(&message.to_string());
        assert_eq!(message, again, "{source:?} -> {message}");
    }
}

fn identifier() -> impl Strategy<Value = Identifier> {
    ("[a-z][a-zA-Z0-9]{0,6}", proptest::option::of("[a-z]{1,4}")).prop_map(|(name, ns)| {
        Identifier {
            namespace: ns,
            name,
        }
    })
}

fn literal() -> impl Strategy<Value = Literal> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_.-]{0,8}".prop_map(|value| Literal {
            value,
            quoted: false
        }),
        "-?(0|[1-9][0-9]{0,4})(\\.[0-9]{1,3})?".prop_map(|value| Literal {
            value,
            quoted: false
        }),
        "[^\\\\|\0]{0,12}".prop_map(|value| Literal {
            value,
            quoted: true
        }),
    ]
}

fn opt() -> impl Strategy<Value = Opt> {
    (
        identifier(),
        prop_oneof![
            literal().prop_map(OptValue::Literal),
            "[a-z][a-z0-9]{0,5}".prop_map(OptValue::Variable),
        ],
    )
        .prop_map(|(name, value)| Opt { name, value })
}

fn distinct_opts() -> impl Strategy<Value = Vec<Opt>> {
    proptest::collection::vec(opt(), 0..3).prop_map(|opts| {
        let mut seen = Vec::new();
        opts.into_iter()
            .filter(|o| {
                let key = (o.name.namespace.clone(), o.name.name.clone());
                if seen.contains(&key) {
                    false
                } else {
                    seen.push(key);
                    true
                }
            })
            .collect()
    })
}

fn function() -> impl Strategy<Value = Function> {
    (identifier(), distinct_opts()).prop_map(|(name, options)| Function { name, options })
}

fn expression() -> impl Strategy<Value = Expression> {
    prop_oneof![
        ("[a-z][a-z0-9]{0,5}", proptest::option::of(function())).prop_map(|(v, f)| Expression {
            operand: Some(Operand::Variable(v)),
            function: f,
            attributes: Vec::new(),
        }),
        (literal(), proptest::option::of(function())).prop_map(|(l, f)| Expression {
            operand: Some(Operand::Literal(l)),
            function: f,
            attributes: Vec::new(),
        }),
        function().prop_map(|f| Expression {
            operand: None,
            function: Some(f),
            attributes: Vec::new(),
        }),
    ]
}

fn markup() -> impl Strategy<Value = Markup> {
    (
        prop_oneof![
            Just(MarkupKind::Open),
            Just(MarkupKind::Close),
            Just(MarkupKind::Standalone)
        ],
        identifier(),
        distinct_opts(),
    )
        .prop_map(|(kind, name, options)| Markup {
            kind,
            name,
            options,
            attributes: Vec::new(),
        })
}

fn pattern() -> impl Strategy<Value = Pattern> {
    proptest::collection::vec(
        prop_oneof![
            "[^\\\\{}\0]{1,10}".prop_map(Part::Text),
            expression().prop_map(Part::Expression),
            markup().prop_map(Part::Markup),
        ],
        0..4,
    )
    .prop_map(|parts| {
        // Adjacent text parts merge when parsed; merge them here too.
        let mut merged: Vec<Part> = Vec::new();
        for part in parts {
            match (merged.last_mut(), part) {
                (Some(Part::Text(a)), Part::Text(b)) => a.push_str(&b),
                (_, part) => merged.push(part),
            }
        }
        Pattern(merged)
    })
}

fn message() -> impl Strategy<Value = Message> {
    prop_oneof![
        pattern().prop_map(|p| {
            let starts_badly = match p.0.first() {
                Some(Part::Text(t)) => t.starts_with(|c: char| c == '.' || c.is_whitespace()),
                None => true,
                _ => false,
            };
            if starts_badly {
                Message::Complex(Complex {
                    declarations: Vec::new(),
                    body: Body::Pattern(p),
                })
            } else {
                Message::Simple(p)
            }
        }),
        (
            proptest::collection::vec(("[a-z][a-z0-9]{0,4}", function()), 1..3),
            proptest::collection::vec((literal(), pattern()), 0..3),
            pattern(),
        )
            .prop_map(|(inputs, cases, fallback)| {
                let mut declarations: Vec<Declaration> = Vec::new();
                for (v, f) in inputs {
                    if declarations
                        .iter()
                        .any(|d| matches!(d, Declaration::Input { variable, .. } if *variable == v))
                    {
                        continue;
                    }
                    declarations.push(Declaration::Input {
                        variable: v.clone(),
                        expression: Expression {
                            operand: Some(Operand::Variable(v)),
                            function: Some(f),
                            attributes: Vec::new(),
                        },
                    });
                }
                let selectors: Vec<String> = declarations
                    .iter()
                    .map(|d| match d {
                        Declaration::Input { variable, .. }
                        | Declaration::Local { variable, .. } => variable.clone(),
                    })
                    .collect();
                let mut variants: Vec<Variant> = Vec::new();
                for (key, p) in cases {
                    let keys: Vec<Key> = (0..selectors.len())
                        .map(|i| {
                            if i == 0 {
                                Key::Literal(key.clone())
                            } else {
                                Key::Wildcard
                            }
                        })
                        .collect();
                    if variants.iter().any(|v| v.keys == keys) {
                        continue;
                    }
                    variants.push(Variant { keys, pattern: p });
                }
                variants.push(Variant {
                    keys: vec![Key::Wildcard; selectors.len()],
                    pattern: fallback,
                });
                Message::Complex(Complex {
                    declarations,
                    body: Body::Matcher(Matcher {
                        selectors,
                        variants,
                    }),
                })
            }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn any_message_round_trips(message in message()) {
        let source = message.to_string();
        let again = parse(&source).unwrap_or_else(|e| panic!("{source:?}: {e}"));
        prop_assert_eq!(again, message);
    }

    #[test]
    fn any_text_parses_or_fails_without_panicking(source in "\\PC{0,64}") {
        let _ = parse(&source);
    }

    #[test]
    fn any_bytes_of_a_valid_message_are_safe(index in 0usize..12, cut in 0usize..80) {
        let source = ROUND_TRIPS[index];
        let end = source
            .char_indices()
            .map(|(i, _)| i)
            .nth(cut)
            .unwrap_or(source.len());
        let _ = parse(&source[..end]);
    }
}
