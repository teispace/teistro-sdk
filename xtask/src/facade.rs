//! The **typed engine façade**: the adapter's own packages, generated
//! from the engine's own description (ADR-0030).
//!
//! Written by `cargo xtask engine` and gated by `check-engine`, from the
//! same reading that writes the adapter's dispatch and the measured
//! page — so a façade cannot type an argument the dispatch would refuse
//! by name.
//!
//! # What a façade is, and what it is not
//!
//! `sdk.engine.call('tm_delta_t', { jd_ut1: 2451545 })` works for any
//! engine that describes itself, and is typed in its arguments and not
//! in its name. A façade gives the names back:
//!
//! ```ts
//! const engine = teimeris(sdk.engine);
//! engine.tmDeltaT({ jdUt1: 2451545 });        // → number
//! ```
//!
//! It is **a value a consumer takes**, not a promise laid over the one
//! they have. ADR-0030 first said a TypeScript declaration merge and a
//! Python protocol, and both would have promised methods that nothing
//! installs — the amendment on that section says why. Dart's extension
//! was right as written, because an extension method has a body.
//!
//! # What the shape of the answer is measured from
//!
//! **Most callable functions answer with exactly one value** — the
//! measured page counts them, and the count is regenerated with it — so
//! a façade method hands back that one value *as itself* — a `double`, a
//! `String`, a struct's own type — rather than an object a caller has to
//! index; the few with more get a record apiece, and those with none
//! answer with nothing.
//!
//! The same measurement settles a question every target would otherwise
//! have raised: `return`, the key a function's own return value comes
//! back under, **never appears beside another key**. So no record field
//! is ever named `return`, and no target has to rename a keyword.

use std::fmt::Write as _;

use crate::engine::{Declared, Described, Function, Shape, Vocabulary, describe, reached};
use crate::generated::Output;

/// Where the adapter's packages live. One directory per target beside
/// the Rust crate, because a package is what ships the platform binary
/// and its licence, and the façade is what makes installing one worth
/// doing.
const NODE: &str = "adapters/ephemeris-teimeris/node/engine.js";
const NODE_TYPES: &str = "adapters/ephemeris-teimeris/node/engine.d.ts";
/// Under `lib/`, because that is where a Dart package's own code lives
/// and the façade is the package's reason to exist.
const DART: &str = "adapters/ephemeris-teimeris/dart/lib/engine.dart";
/// Inside the importable package, for the same reason.
const PYTHON: &str = "adapters/ephemeris-teimeris/python/teistro_ephemeris_teimeris/engine.py";

/// `snake_case` to `camelCase`, which is what Node and Dart spell a name
/// in. The engine's own name is what crosses; this is only what a
/// consumer writes.
fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The three words a target has for the three kinds of scalar the engine
/// declares, and the one rule for the fourth kind, a struct.
///
/// **There is no fifth kind**, and that is what makes a façade possible
/// without a type table per target: an enum member, a body id and a flag
/// set all cross as integers, and a struct is named for itself in every
/// target — `tm_datetime` is `TmDatetime` everywhere. So one decision is
/// made here, and each target supplies its own vocabulary for it, rather
/// than four closures that could quietly disagree about which kind a
/// type is.
struct Words {
    text: &'static str,
    /// A number with a fractional part.
    fractional: &'static str,
    /// A whole number: a count, an id, an enum member, a flag set.
    integer: &'static str,
    /// A list of one of the above, as an answer hands it back, with `{}`
    /// where the element's word goes.
    list: &'static str,
    /// A list as an argument takes it, which may be wider than the answer
    /// type: Python takes any `Sequence`, because its `list` is invariant
    /// and `list[int]` would not be accepted where `list[float]` is.
    list_taken: &'static str,
}

impl Words {
    /// TypeScript, whose one `number` covers both numeric kinds — said
    /// twice on purpose, so the collapse is visible rather than implied
    /// by a shorter branch.
    const TYPESCRIPT: Self = Self {
        text: "string",
        fractional: "number",
        integer: "number",
        list: "readonly {}[]",
        list_taken: "readonly {}[]",
    };
    const DART: Self = Self {
        text: "String",
        fractional: "double",
        integer: "int",
        list: "List<{}>",
        list_taken: "List<{}>",
    };
    const PYTHON: Self = Self {
        text: "str",
        fractional: "float",
        integer: "int",
        list: "list[{}]",
        list_taken: "Sequence[{}]",
    };

    /// What one declared type is called here.
    ///
    /// **Refuses** a type the engine's own vocabulary does not classify,
    /// rather than calling it an integer and generating a façade that
    /// type-checks a wrong call. The Dart constructor emitter refuses an
    /// unknown *role* for the same reason and says so: a generator that
    /// quietly guesses writes code that compiles and is wrong.
    fn of(&self, declared: &Declared<'_>, vocabulary: &Vocabulary, whose: &str) -> String {
        self.word(declared, vocabulary, whose, self.list)
    }

    /// What an argument of a declared type is called here.
    fn taken(&self, declared: &Declared<'_>, vocabulary: &Vocabulary, whose: &str) -> String {
        self.word(declared, vocabulary, whose, self.list_taken)
    }

    fn word(
        &self,
        declared: &Declared<'_>,
        vocabulary: &Vocabulary,
        whose: &str,
        list: &str,
    ) -> String {
        let one = match kind(declared.base, vocabulary, whose) {
            Kind::Text => self.text.to_string(),
            Kind::Fractional => self.fractional.to_string(),
            Kind::Integer => self.integer.to_string(),
            Kind::Struct => pascal(declared.base),
        };
        if declared.list {
            list.replace("{}", &one)
        } else {
            one
        }
    }
}

/// Which of the four kinds a declared type is.
///
/// The same in every target; only the words for each kind differ.
fn kind(declared: &str, vocabulary: &Vocabulary, whose: &str) -> Kind {
    if declared == "string" {
        return Kind::Text;
    }
    if vocabulary.shape(declared).is_some() {
        return Kind::Struct;
    }
    assert!(
        vocabulary.is_number(declared),
        "the façade generator cannot type `{declared}` on `{whose}`: the engine's \
             description does not classify it as a number or a struct, and typing it as \
             one would make a call that type-checks and fails. Teach `Words` that kind, or \
             leave the function out of the callable set."
    );
    if vocabulary.is_float(declared) {
        Kind::Fractional
    } else {
        Kind::Integer
    }
}

/// The four kinds a value crosses as.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Fractional,
    Integer,
    Struct,
}

/// The façades, from one reading of the description.
pub(crate) fn outputs(
    version: &str,
    callable: &[&Function],
    vocabulary: &Vocabulary,
) -> Vec<Output> {
    let described: Vec<Described<'_>> = callable.iter().map(|f| describe(f)).collect();
    // Every struct a callable function passes either way, with what it
    // nests, in declaration order. One list for all four targets, so no
    // façade names a struct another does not.
    let structs = reached(
        callable,
        &["struct_in", "struct_out", "array_in", "array_out"],
        vocabulary,
    );
    vec![
        Output::new(NODE, node(version, &described, &structs, vocabulary)),
        Output::new(
            NODE_TYPES,
            node_types(version, &described, &structs, vocabulary),
        ),
        Output::new(DART, dart(version, &described, &structs, vocabulary)),
        Output::new(PYTHON, python(version, &described, &structs, vocabulary)),
    ]
}

/// The header every façade carries, in that language's comment.
fn header(version: &str, comment: &str) -> String {
    let mut out = String::new();
    for line in [
        "The engine's own operations, typed. **Generated by",
        "`cargo xtask engine`. Do not edit.**",
        "",
        &format!("From the engine's own description at version `{version}`."),
        "Which of its functions are here is a measurement, not a choice:",
        "`docs/03-design/engine-passthrough-measured.md` classifies all of",
        "them into what is callable, what this adapter will never hand over,",
        "and what is queued behind one more shape.",
        "",
        "A method's arguments are named the way this language names things;",
        "the keys that cross are the engine's own, which is what",
        "`sdk.engine.signature(name)` reads. A call through here is to a",
        "named engine and does not survive changing it — which is why it is",
        "taken as a value rather than laid over `sdk.engine` (ADR-0030).",
    ] {
        if line.is_empty() {
            let _ = writeln!(out, "{}", comment.trim_end());
        } else {
            let _ = writeln!(out, "{comment} {line}");
        }
    }
    out
}

/// The note a mutating operation carries, so a consumer chooses it
/// knowingly.
const MUTATES: &str = "Changes engine state the SDK's provenance does not record: a chart cast \
                       afterwards says it was computed one way and was computed another.";

/// The argument list of one method, as `(what the consumer writes, the
/// key that crosses)`.
fn arguments<'a>(described: &'a Described<'a>) -> Vec<(String, &'a str)> {
    described
        .takes
        .iter()
        .map(|(name, _)| (camel(name), *name))
        .collect()
}

// ── Node ───────────────────────────────────────────────────────────────

/// The Node façade's runtime: a class whose methods call the engine by
/// name, so a consumer never spells one.
///
/// A class with prototype methods rather than an object of closures, for
/// the reason the SDK's own areas are classes: the methods are shared by
/// every wrapper rather than rebuilt for each.
fn node(
    version: &str,
    described: &[Described<'_>],
    structs: &[&Shape],
    vocabulary: &Vocabulary,
) -> String {
    let mut out = header(version, "//");
    out.push_str(&node_structs(structs, vocabulary));
    // Exported, because the `.d.ts` beside this declares it and a
    // consumer holding one wants its type to have a name.
    let _ = writeln!(
        out,
        "\nexport class TeimerisEngine {{\n  #engine;\n\n  constructor(engine) {{\n    this.#engine = engine;\n    Object.freeze(this);\n  }}\n"
    );
    for one in described {
        let args = arguments(one);
        let (params, keys) = if args.is_empty() {
            (String::new(), String::from("{}"))
        } else {
            (
                format!(
                    "{{ {} }}",
                    args.iter()
                        .map(|(spelling, _)| spelling.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                format!(
                    "{{ {} }}",
                    args.iter()
                        .zip(one.takes.iter())
                        .map(|((spelling, key), (_, declared))| {
                            let written = node_write(spelling, declared, vocabulary);
                            if one.nullable(key) {
                                // Left out and null both cross as null.
                                format!("{key}: {spelling} == null ? null : {written}")
                            } else if written != *spelling {
                                format!("{key}: {written}")
                            } else if spelling == key {
                                // The engine's own key and this language's
                                // spelling of it agree, so the shorthand is
                                // the honest form.
                                spelling.clone()
                            } else {
                                format!("{key}: {spelling}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        };
        let call = format!("this.#engine.call('{}', {keys})", one.name);
        let body = match one.gives.as_slice() {
            [] => format!("    {call};"),
            [(only, declared)] => format!(
                "    return {};",
                node_read(&format!("{call}.{only}"), declared, vocabulary)
            ),
            many => {
                let fields = many
                    .iter()
                    .map(|(key, declared)| {
                        format!(
                            "{}: {}",
                            camel(key),
                            node_read(&format!("answered.{key}"), declared, vocabulary)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("    const answered = {call};\n    return {{ {fields} }};")
            }
        };
        let note = if one.mutates {
            format!("\n   *\n   * {MUTATES}")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  /**\n   * `{}`.{note}\n   */\n  {}({params}) {{\n{body}\n  }}\n",
            one.name,
            camel(one.name)
        );
    }
    let _ = writeln!(
        out,
        "}}\n\n/**\n * The engine's own operations, typed.\n *\n * Pass `sdk.engine`; what comes back has a method per operation this\n * adapter offers, each calling the engine by its own name.\n */\nexport const teimeris = (engine) => new TeimerisEngine(engine);"
    );
    out
}

/// A value read out of an answer, in Node: a struct is converted to the
/// spelling a consumer writes, a list of them element by element, and
/// anything else is itself.
fn node_read(expression: &str, declared: &Declared<'_>, vocabulary: &Vocabulary) -> String {
    node_convert(expression, declared, vocabulary, &node_reader)
}

/// A value written into a call, in Node: the reverse of [`node_read`].
fn node_write(expression: &str, declared: &Declared<'_>, vocabulary: &Vocabulary) -> String {
    node_convert(expression, declared, vocabulary, &node_writer)
}

/// One conversion, applied to a struct or to each struct of a list.
fn node_convert(
    expression: &str,
    declared: &Declared<'_>,
    vocabulary: &Vocabulary,
    converter: &dyn Fn(&str) -> String,
) -> String {
    match (vocabulary.shape(declared.base).is_some(), declared.list) {
        (false, _) => expression.to_string(),
        (true, false) => format!("{}({expression})", converter(declared.base)),
        (true, true) => format!("{expression}.map({})", converter(declared.base)),
    }
}

/// The name of the function that reads a struct out of an answer.
fn node_reader(declared: &str) -> String {
    format!("read{}", pascal(declared))
}

/// The name of the function that writes a struct into a call.
fn node_writer(declared: &str) -> String {
    format!("write{}", pascal(declared))
}

/// The converters between a struct's engine keys and the camelCase a
/// Node consumer writes.
///
/// Generated per struct rather than a generic key-mapper, because a
/// generic one would camel-case whatever it was handed: a key the engine
/// does not declare would cross silently and the engine would never see
/// the field the caller meant. These read exactly the declared fields,
/// and a missing one reaches the dispatch as missing, which refuses it by
/// its whole path.
fn node_structs(structs: &[&Shape], vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    for shape in structs {
        let _ = writeln!(
            out,
            "\n/** `{0}` as the engine answers it, spelled as a consumer reads it. */\nconst {1} = (value) => ({{ {2} }});\n\n/** `{0}` as a consumer writes it, keyed as the engine reads it. */\nconst {3} = (value) => ({{ {4} }});",
            shape.name,
            node_reader(&shape.name),
            node_fields(shape, vocabulary, Toward::Consumer),
            node_writer(&shape.name),
            node_fields(shape, vocabulary, Toward::Engine),
        );
    }
    out
}

/// Which way a Node converter runs.
#[derive(Clone, Copy)]
enum Toward {
    /// Engine keys in, camelCase out.
    Consumer,
    /// camelCase in, engine keys out.
    Engine,
}

/// One converter's object literal: every declared field, renamed, with a
/// nested struct converted by its own converter.
fn node_fields(shape: &Shape, vocabulary: &Vocabulary, toward: Toward) -> String {
    shape
        .crossing()
        .map(|field| {
            let (from, to) = match toward {
                Toward::Consumer => (field.name.clone(), camel(&field.name)),
                Toward::Engine => (camel(&field.name), field.name.clone()),
            };
            let value = format!("value.{from}");
            let value = match (vocabulary.shape(&field.base).is_some(), toward) {
                (false, _) => value,
                (true, Toward::Consumer) => format!("{}({value})", node_reader(&field.base)),
                (true, Toward::Engine) => format!("{}({value})", node_writer(&field.base)),
            };
            format!("{to}: {value}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The Node façade's types, which are the half a consumer reads.
fn node_types(
    version: &str,
    described: &[Described<'_>],
    structs: &[&Shape],
    vocabulary: &Vocabulary,
) -> String {
    let mut out = header(version, "//");
    let _ = writeln!(out, "\nimport type {{ Engine }} from '@teistro/sdk';");
    for shape in structs {
        let fields = shape
            .crossing()
            .map(|field| {
                format!(
                    "  readonly {}: {};",
                    camel(&field.name),
                    Words::TYPESCRIPT.of(&Declared::one(&field.base), vocabulary, &shape.name)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = writeln!(
            out,
            "\n/** `{}`, as the engine declares it. Every field is required. */\nexport interface {} {{\n{fields}\n}}",
            shape.name,
            pascal(&shape.name)
        );
    }
    let _ = writeln!(
        out,
        "\n/** The engine's own operations, typed. */\nexport declare class TeimerisEngine {{"
    );
    for one in described {
        let args = arguments(one);
        let params = if args.is_empty() {
            String::new()
        } else {
            format!(
                "args: {{ {} }}",
                args.iter()
                    .zip(one.takes.iter())
                    .map(|((spelling, key), (_, declared))| {
                        let named = Words::TYPESCRIPT.taken(declared, vocabulary, one.name);
                        if one.nullable(key) {
                            format!("readonly {spelling}?: {named} | null")
                        } else {
                            format!("readonly {spelling}: {named}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        let returns = match one.gives.as_slice() {
            [] => String::from("void"),
            [(_, declared)] => Words::TYPESCRIPT.of(declared, vocabulary, one.name),
            many => format!(
                "{{ {} }}",
                many.iter()
                    .map(|(key, declared)| format!(
                        "readonly {}: {}",
                        camel(key),
                        Words::TYPESCRIPT.of(declared, vocabulary, one.name)
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        };
        let note = if one.mutates {
            format!("\n   *\n   * {MUTATES}")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  /**\n   * `{}`.{note}\n   */\n  {}({params}): {returns};",
            one.name,
            camel(one.name)
        );
    }
    let _ = writeln!(
        out,
        "}}\n\n/** The engine's own operations, typed. Pass `sdk.engine`. */\nexport declare function teimeris(engine: Engine): TeimerisEngine;"
    );
    out
}

/// Reading one value out of the answer, in Dart, from an expression
/// that looks it up.
///
/// A number goes through `num` rather than straight to `double`: JSON
/// carries no distinction the engine's `double` and `int32_t` keep, so a
/// whole-valued double would decode as an `int` and `as double` would
/// throw on it. `toDouble()` accepts either and costs nothing.
fn dart_read(
    lookup: &str,
    declared: &Declared<'_>,
    vocabulary: &Vocabulary,
    whose: &str,
) -> String {
    let one = |value: &str| match kind(declared.base, vocabulary, whose) {
        Kind::Text => format!("{value} as String"),
        Kind::Fractional => format!("({value} as num).toDouble()"),
        Kind::Integer => format!("({value} as num).toInt()"),
        Kind::Struct => format!(
            "{}.fromJson({value} as Map<String, Object?>)",
            pascal(declared.base)
        ),
    };
    if declared.list {
        format!(
            "({lookup}! as List<Object?>).map((one) => {}).toList()",
            one("one!")
        )
    } else {
        one(&format!("{lookup}!"))
    }
}

/// Writing one argument into a call, in Dart: a struct as its object, a
/// list of them element by element, anything else as itself.
fn dart_write(
    spelling: &str,
    declared: &Declared<'_>,
    nullable: bool,
    vocabulary: &Vocabulary,
) -> String {
    if vocabulary.shape(declared.base).is_none() {
        return spelling.to_string();
    }
    match (declared.list, nullable) {
        (true, _) => format!("{spelling}.map((one) => one.toJson()).toList()"),
        (false, true) => format!("{spelling}?.toJson()"),
        (false, false) => format!("{spelling}.toJson()"),
    }
}

// ── Dart ───────────────────────────────────────────────────────────────

/// The Dart façade: an **extension** on the SDK's own `Engine`.
///
/// The one target where an augmentation is honest, because a Dart
/// extension method has a body and is resolved statically: it is typed
/// and installed at once, and needs no wrapper to take.
fn dart(
    version: &str,
    described: &[Described<'_>],
    structs: &[&Shape],
    vocabulary: &Vocabulary,
) -> String {
    let mut out = header(version, "//");
    // The same directive the SDK's own generated Dart carries, for the
    // same reason: the generator lays the file out, so `dart format .`
    // leaves it alone and `check-engine` can regenerate it on a machine
    // with no Dart toolchain and still compare it byte for byte.
    let _ = writeln!(
        out,
        "// dart format off\n\nimport 'package:teistro/teistro.dart';\n\n/// The engine's own operations, typed.\n///\n/// An extension rather than a wrapper, because a Dart extension method\n/// has a body: importing this file is what makes the names exist, and\n/// they are as typed and as real as any method.\nextension TeimerisEngine on Engine {{"
    );
    for one in described {
        let args = arguments(one);
        let params = dart_parameters(one, &args, vocabulary);
        let keys = if args.is_empty() {
            String::from("const <String, Object?>{}")
        } else {
            format!(
                "<String, Object?>{{{}}}",
                args.iter()
                    .zip(one.takes.iter())
                    .map(|((spelling, key), (_, declared))| {
                        format!(
                            "'{key}': {}",
                            dart_write(spelling, declared, one.nullable(key), vocabulary)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let call = format!("call('{}', {keys})", one.name);
        let (returns, body) = match one.gives.as_slice() {
            [] => (String::from("void"), format!("    {call};")),
            [(only, declared)] => (
                Words::DART.of(declared, vocabulary, one.name),
                format!(
                    "    final answered = ({call}) as Map<String, Object?>;\n    return {};",
                    dart_read(
                        &format!("answered['{only}']"),
                        declared,
                        vocabulary,
                        one.name
                    )
                ),
            ),
            many => {
                let record = many
                    .iter()
                    .map(|(key, declared)| {
                        format!(
                            "{} {}",
                            Words::DART.of(declared, vocabulary, one.name),
                            camel(key)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let fields = many
                    .iter()
                    .map(|(key, declared)| {
                        format!(
                            "{}: {}",
                            camel(key),
                            dart_read(
                                &format!("answered['{key}']"),
                                declared,
                                vocabulary,
                                one.name
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    // Braces: `(int major, int minor)` is a record of two
                    // POSITIONAL fields whose names Dart ignores, and
                    // `({int major, int minor})` is the named one the
                    // body below builds. Without them the generated file
                    // does not compile, which is how this was found.
                    format!("({{{record}}})"),
                    format!(
                        "    final answered = ({call}) as Map<String, Object?>;\n    return ({fields});"
                    ),
                )
            }
        };
        let note = if one.mutates {
            format!("\n  ///\n  /// {MUTATES}")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  /// `{}`.{note}\n  {returns} {}({params}) {{\n{body}\n  }}\n",
            one.name,
            camel(one.name)
        );
    }
    let _ = writeln!(out, "}}");
    out.push_str(&dart_structs(structs, vocabulary));
    out
}

/// A Dart method's named parameters: required, or nullable and
/// omittable where the engine takes a null.
fn dart_parameters(
    one: &Described<'_>,
    args: &[(String, &str)],
    vocabulary: &Vocabulary,
) -> String {
    if args.is_empty() {
        String::new()
    } else {
        format!(
            "{{{}}}",
            args.iter()
                .zip(one.takes.iter())
                .map(|((spelling, key), (_, declared))| {
                    let named = Words::DART.taken(declared, vocabulary, one.name);
                    if one.nullable(key) {
                        format!("{named}? {spelling}")
                    } else {
                        format!("required {named} {spelling}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// A class per struct, in Dart: named fields, and the two conversions to
/// and from the object that crosses.
///
/// A class rather than a record, because a record cannot carry a
/// `fromJson` and every struct has to be built from an answer as well as
/// written into a call.
fn dart_structs(structs: &[&Shape], vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    for shape in structs {
        let class = pascal(&shape.name);
        let whose = shape.name.as_str();
        let fields: Vec<(&str, String)> = shape
            .crossing()
            .map(|field| (field.name.as_str(), camel(&field.name)))
            .collect();
        let bases: Vec<&str> = shape.crossing().map(|field| field.base.as_str()).collect();
        let parameters = fields
            .iter()
            .map(|(_, spelling)| format!("required this.{spelling}"))
            .collect::<Vec<_>>()
            .join(", ");
        let reads = fields
            .iter()
            .zip(&bases)
            .map(|((key, spelling), base)| {
                format!(
                    "        {spelling}: {},",
                    dart_read(
                        &format!("json['{key}']"),
                        &Declared::one(base),
                        vocabulary,
                        whose
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let declarations = fields
            .iter()
            .zip(&bases)
            .map(|((key, spelling), base)| {
                format!(
                    "  /// `{key}`.\n  final {} {spelling};",
                    Words::DART.of(&Declared::one(base), vocabulary, whose)
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let writes = fields
            .iter()
            .zip(&bases)
            .map(|((key, spelling), base)| {
                if vocabulary.shape(base).is_some() {
                    format!("        '{key}': {spelling}.toJson(),")
                } else {
                    format!("        '{key}': {spelling},")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = writeln!(
            out,
            "\n/// `{whose}`, as the engine declares it. Every field is required.\nfinal class {class} {{\n  /// Every field, named.\n  const {class}({{{parameters}}});\n\n  /// Read from the object the engine answers with.\n  factory {class}.fromJson(Map<String, Object?> json) => {class}(\n{reads}\n      );\n\n{declarations}\n\n  /// The object the engine reads, keyed by its own field names.\n  Map<String, Object?> toJson() => <String, Object?>{{\n{writes}\n      }};\n}}"
        );
    }
    out
}

// ── Python ─────────────────────────────────────────────────────────────

/// The Python façade: a class wrapping the SDK's own engine.
///
/// A class and not a `Protocol`, which ADR-0030 first said: a protocol
/// types a call that nothing installs, and a typed call that fails at
/// run time is worse than an untyped one that works. The method names
/// are the engine's own, because Python spells them the same way the
/// engine does and a translation nobody needs is a translation to get
/// wrong.
fn python(
    version: &str,
    described: &[Described<'_>],
    structs: &[&Shape],
    vocabulary: &Vocabulary,
) -> String {
    let mut out = header(version, "#");
    let _ = writeln!(
        out,
        "\nfrom __future__ import annotations\n\nfrom collections.abc import Sequence\nfrom typing import TypedDict, cast\n\nfrom teistro import Engine\n"
    );
    out.push_str(&python_structs(structs, vocabulary));
    // The answers with more than one value, as the types they are.
    // Named for the operation, because that is the only thing they have
    // in common with each other.
    for one in described {
        if one.gives.len() < 2 {
            continue;
        }
        // A record is named for its operation and a struct for itself, in
        // one namespace; the engine has no struct named like an operation
        // that answers a record, and this is where the day it does is
        // caught rather than shadowed.
        assert!(
            !structs
                .iter()
                .any(|shape| pascal(&shape.name) == pascal(one.name)),
            "`{}` answers a record whose Python name is also a struct's",
            one.name
        );
        let _ = writeln!(
            out,
            "\nclass {}(TypedDict):\n    \"\"\"What `{}` answers with.\"\"\"\n",
            pascal(one.name),
            one.name
        );
        for (key, declared) in &one.gives {
            let _ = writeln!(
                out,
                "    {key}: {}",
                Words::PYTHON.of(declared, vocabulary, one.name)
            );
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(
        out,
        "\n\nclass TeimerisEngine:\n    \"\"\"The engine's own operations, typed.\n\n    Pass `sdk.engine`; every method calls the engine by its own name.\n    \"\"\"\n\n    __slots__ = (\"_engine\",)\n\n    def __init__(self, engine: Engine) -> None:\n        self._engine = engine"
    );
    for one in described {
        let params = python_parameters(one, vocabulary);
        let keys = one
            .takes
            .iter()
            .map(|(name, _)| format!("{name}={name}"))
            .collect::<Vec<_>>()
            .join(", ");
        let comma = if keys.is_empty() { "" } else { ", " };
        let call = format!("self._engine.call(\"{}\"{comma}{keys})", one.name);
        let (returns, body) = match one.gives.as_slice() {
            [] => (String::from("None"), format!("        {call}")),
            [(only, declared)] => {
                let named = Words::PYTHON.of(declared, vocabulary, one.name);
                let read = python_read(
                    &format!("answered[\"{only}\"]"),
                    declared,
                    &named,
                    vocabulary,
                    one.name,
                );
                (
                    named,
                    format!(
                        "        answered = cast(dict[str, object], {call})\n        return {read}"
                    ),
                )
            }
            _ => (
                pascal(one.name),
                format!("        return cast({}, {call})", pascal(one.name)),
            ),
        };
        let note = if one.mutates {
            format!("\n\n        {MUTATES}")
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "\n    def {}(self{params}) -> {returns}:\n        \"\"\"`{}`.{note}\n        \"\"\"\n{body}",
            one.name, one.name
        );
    }
    let _ = writeln!(
        out,
        "\n\ndef teimeris(engine: Engine) -> TeimerisEngine:\n    \"\"\"The engine's own operations, typed. Pass `sdk.engine`.\"\"\"\n    return TeimerisEngine(engine)"
    );
    out
}

/// A `TypedDict` per struct, in Python.
///
/// First in the file, because a record may hold one and a `TypedDict`
/// must be declared before it is named. The engine's declaration order
/// is already an order that works, and `reached` asserts it.
fn python_structs(structs: &[&Shape], vocabulary: &Vocabulary) -> String {
    let mut out = String::new();
    for shape in structs {
        let _ = writeln!(
            out,
            "\nclass {}(TypedDict):\n    \"\"\"`{}`, as the engine declares it. Every field is required.\"\"\"\n",
            pascal(&shape.name),
            shape.name
        );
        for field in shape.crossing() {
            let _ = writeln!(
                out,
                "    {}: {}",
                field.name,
                Words::PYTHON.of(&Declared::one(&field.base), vocabulary, &shape.name)
            );
        }
        let _ = writeln!(out);
    }
    out
}

/// Reading one value out of the answer, in Python.
///
/// `float(...)` rather than a cast, for the reason `dart_read` explains:
/// JSON keeps no distinction between the engine's `double` and its
/// `int32_t`, so a whole-valued double arrives as an `int` and a cast
/// would be a lie a type checker believed.
fn python_read(
    value: &str,
    declared: &Declared<'_>,
    named: &str,
    vocabulary: &Vocabulary,
    whose: &str,
) -> String {
    match (kind(declared.base, vocabulary, whose), declared.list) {
        (Kind::Text | Kind::Struct, _) => format!("cast({named}, {value})"),
        (Kind::Fractional, false) => format!("float(cast(float, {value}))"),
        (Kind::Integer, false) => format!("int(cast(int, {value}))"),
        (Kind::Fractional, true) => format!("[float(one) for one in cast(list[float], {value})]"),
        (Kind::Integer, true) => format!("[int(one) for one in cast(list[int], {value})]"),
    }
}

/// A Python method's parameters after `self`.
///
/// Built rather than collected: Python's parameters are the engine's own
/// names, so there is nothing to join between them and each carries its
/// own leading comma.
fn python_parameters(one: &Described<'_>, vocabulary: &Vocabulary) -> String {
    let mut params = String::new();
    for (at, (name, declared)) in one.takes.iter().enumerate() {
        let named = Words::PYTHON.taken(declared, vocabulary, one.name);
        // A default only where every parameter after it has one too,
        // which is the one place Python allows it; before a required
        // parameter an optional struct is still `None`-able, only not
        // omittable.
        let rest_nullable = one.takes.iter().skip(at).all(|(key, _)| one.nullable(key));
        let _ = match (one.nullable(name), rest_nullable) {
            (true, true) => write!(params, ", {name}: {named} | None = None"),
            (true, false) => write!(params, ", {name}: {named} | None"),
            _ => write!(params, ", {name}: {named}"),
        };
    }
    params
}

/// `snake_case` to `PascalCase`, for the name of a result type.
fn pascal(name: &str) -> String {
    let camelled = camel(name);
    let mut chars = camelled.chars();
    chars.next().map_or_else(String::new, |first| {
        format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
    })
}
