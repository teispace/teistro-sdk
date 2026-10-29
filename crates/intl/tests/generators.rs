//! Every emitter over a model that holds each parameter shape, a keyword
//! name, an entity, a nested and an empty group: each target's text is
//! held to the line it writes for that shape, and the model the sources
//! give is held to the kinds it seeds and the refusals it makes.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]
#![allow(
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "a test indexes what it built, and the model of everything is one value"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use teistro_intl::analysis::{ParamType, signature};
use teistro_intl::generate::{
    Group, MessageModel, Model, Node, RustPaths, dart, java, javascript, python, rust, typescript,
    typescript_declarations,
};
use teistro_intl::mf2::parse;
use teistro_intl::sdk_root;
use teistro_intl::source::{META_FILE, Meta, Tree};

fn entity(kind: &str) -> ParamType {
    ParamType::Entity(Some(kind.to_owned()))
}

fn message(key: &str, params: &[(&str, ParamType)]) -> Node {
    Node::Message(MessageModel {
        key: key.to_owned(),
        params: params
            .iter()
            .map(|(name, kind)| ((*name).to_owned(), kind.clone()))
            .collect(),
    })
}

fn group(children: Vec<(&str, Node)>) -> Node {
    Node::Group(Group {
        children: children
            .into_iter()
            .map(|(segment, node)| (segment.to_owned(), node))
            .collect(),
    })
}

/// Every parameter shape, in name order.
fn every() -> Vec<(&'static str, ParamType)> {
    vec![
        ("a", ParamType::String),
        ("b", ParamType::Context(String::from("gender"))),
        ("c", ParamType::Integer),
        ("d", ParamType::Number),
        ("e", entity("graha")),
        ("f", entity("rule")),
        ("g", ParamType::Entity(None)),
        ("h", ParamType::List),
        ("i", ParamType::Date),
        ("j", ParamType::Time),
        ("k", ParamType::DateTime),
        ("l", ParamType::Ghati),
        ("m", ParamType::Instant),
    ]
}

/// A model of one of everything, with or without a `gender` context.
fn model(gender: bool) -> Model {
    let mut contexts = BTreeMap::from([(
        String::from("case"),
        vec![String::from("in"), String::from("out")],
    )]);
    if gender {
        contexts.insert(
            String::from("gender"),
            vec![String::from("m"), String::from("f"), String::from("n")],
        );
    }
    let t = group(vec![
        ("empty", message("sdk.t.empty", &[])),
        ("every", message("sdk.t.every", &every())),
        (
            "keyword",
            message(
                "sdk.t.keyword",
                &[("class", ParamType::String), ("in", ParamType::Integer)],
            ),
        ),
        (
            "zoned",
            message(
                "sdk.t.zoned",
                &[("at", ParamType::Instant), ("zone", ParamType::String)],
            ),
        ),
        (
            "short",
            message("sdk.t.short", &[("n", ParamType::Integer)]),
        ),
        (
            "named",
            message(
                "sdk.t.named",
                &[
                    ("b", ParamType::Context(String::from("case"))),
                    ("d", ParamType::Number),
                    ("e", entity("graha")),
                    ("f", entity("rule")),
                    ("g", ParamType::Entity(None)),
                    ("h", ParamType::List),
                    ("j", ParamType::Time),
                    ("l", ParamType::Ghati),
                    ("q", entity("festival")),
                ],
            ),
        ),
        (
            "orphan",
            message(
                "sdk.t.orphan",
                &[("x", ParamType::Context(String::from("nowhere")))],
            ),
        ),
        (
            "BIKRAM_SAMBAT",
            group(vec![(
                "date",
                message("sdk.t.BIKRAM_SAMBAT.date", &[("i", ParamType::Date)]),
            )]),
        ),
    ]);
    let root = Group {
        children: BTreeMap::from([(
            String::from("sdk"),
            group(vec![
                ("t", t),
                ("hollow", group(Vec::new())),
                (
                    "entity",
                    group(vec![(
                        "graha",
                        group(vec![("SUN", Node::Entity(String::from("graha.SUN")))]),
                    )]),
                ),
            ]),
        )]),
    };
    let set = |forms: &[&str]| -> BTreeSet<String> {
        forms.iter().map(|form| (*form).to_owned()).collect()
    };
    Model {
        locale: String::from("xx-Latn"),
        contexts,
        kinds: BTreeMap::from([
            (
                String::from("graha"),
                vec![String::from("SUN"), String::from("MOON")],
            ),
            (String::from("rule"), vec![String::from("FIRST")]),
        ]),
        forms: (set(&["iast", "name", "prose"]), set(&["short"])),
        root,
        keys: [
            "empty",
            "every",
            "keyword",
            "zoned",
            "short",
            "named",
            "orphan",
            "BIKRAM_SAMBAT.date",
        ]
        .iter()
        .map(|key| format!("sdk.t.{key}"))
        .collect(),
    }
}

fn holds(text: &str, lines: &[&str]) {
    for line in lines {
        assert!(text.contains(line), "lacks {line:?} in\n{text}");
    }
}

fn lacks(text: &str, lines: &[&str]) {
    for line in lines {
        assert!(!text.contains(line), "holds {line:?}");
    }
}

#[test]
fn typescript_writes_each_shape_typed_plain_and_declared() {
    let model = model(true);
    let typed = typescript(&model);
    let plain = javascript(&model);
    let declared = typescript_declarations(&model);
    let fields = "a: string; b: Gender; c: number; d: number; e: GrahaKey; f: EntityKey; g: EntityKey; h: readonly (string | EntityKey)[]; i: DateValue; j: TimeValue; k: DateTimeValue; l: GhatiValue; m: InstantValue";
    let values = "a: p.a, b: p.b, c: p.c, d: p.d, e: { '$entity': p.e }, f: { '$entity': p.f }, g: { '$entity': p.g }, h: p.h, i: { '$date': p.i }, j: { '$time': p.j }, k: { '$datetime': p.k }, l: { '$ghati': p.l }, m: { '$instant': p.m }";
    holds(
        &typed,
        &[
            "// Generated by teistro-intl from i18n/xx-Latn. Do not edit.",
            "export type Case = 'in' | 'out';",
            "export type Gender = 'm' | 'f' | 'n';",
            "export type GrahaKey = 'graha.SUN' | 'graha.MOON';",
            "export type RuleKey = 'rule.FIRST';",
            "export type EntityKey = GrahaKey | RuleKey;",
            "  readonly iast: string;",
            "  readonly short?: string;",
            "  readonly gender?: Gender;",
            &format!("every: (p: {{ {fields} }}) => r.render('sdk.t.every', {{ {values} }}),"),
            "empty: () => r.render('sdk.t.empty'),",
            "bikramSambat: {",
            "sun: () => r.entity('graha.SUN'),",
        ],
    );
    holds(
        &plain,
        &[
            &format!("every: (p) => r.render('sdk.t.every', {{ {values} }}),"),
            "empty: () => r.render('sdk.t.empty'),",
            "    iast: forms.iast ?? '',",
            "    short: forms.short,",
            "    gender: forms.gender,",
        ],
    );
    lacks(&plain, &[": string", "export type"]);
    holds(
        &declared,
        &[
            &format!("readonly every: (p: {{ {fields} }}) => string;"),
            "readonly empty: () => string;",
            "readonly bikramSambat: {",
            "readonly sun: () => EntityForms;",
            "export declare function messages(r: Renderer): Messages;",
        ],
    );
    assert!(
        declared.contains("  };\n"),
        "a declared group closes with `}};`"
    );
}

#[test]
fn dart_writes_each_shape_and_renames_a_keyword() {
    let text = dart(&model(true));
    holds(
        &text,
        &[
            "enum Case {\n  in$('in'),\n  out('out');",
            "enum GrahaKey {\n  sun('graha.SUN'),\n  moon('graha.MOON');",
            "  String every({required String a, required Gender b, required int c, required num d, required GrahaKey e, required String f, required String g, required List<Object> h, required DateValue i, required TimeValue j, required DateTimeValue k, required GhatiValue l, required double m}) =>\n      _r.render('sdk.t.every', {'a': a, 'b': b.key, 'c': c, 'd': d, 'e': {r'$entity': e.key}, 'f': {r'$entity': f}, 'g': {r'$entity': g}, 'h': h, 'i': {r'$date': i.json}, 'j': {r'$time': j.json}, 'k': {r'$datetime': k.json}, 'l': {r'$ghati': l.json}, 'm': {r'$instant': m}});",
            "  String keyword({required String class$, required int in$}) =>\n      _r.render('sdk.t.keyword', {'class': class$, 'in': in$});",
            "  String empty() => _r.render('sdk.t.empty');",
            "  EntityForms get sun => _r.entity('graha.SUN');",
            "  MessagesSdkT get t => MessagesSdkT(_r);",
            "    required this.iast,",
            "    this.short,",
            "      short: text('short'),",
            "  final String? short;",
            "  final Gender? gender;",
        ],
    );
}

#[test]
fn python_writes_each_shape_outermost_class_first() {
    let text = python(&model(true));
    holds(
        &text,
        &[
            "class Case(enum.StrEnum):",
            "    IN_ = \"in\"",
            "    SUN = \"graha.SUN\"",
            "    def every(self, *, a: str, b: Gender, c: int, d: float, e: GrahaKey, f: str, g: str, h: Sequence[object], i: DateValue, j: TimeValue, k: DateTimeValue, l: GhatiValue, m: float) -> str:",
            "{\"a\": a, \"b\": b.value, \"c\": c, \"d\": d, \"e\": {\"$entity\": e.value}, \"f\": {\"$entity\": f}, \"g\": {\"$entity\": g}, \"h\": h, \"i\": {\"$date\": i}, \"j\": {\"$time\": j}, \"k\": {\"$datetime\": k}, \"l\": {\"$ghati\": l}, \"m\": {\"$instant\": m}},",
            "    def keyword(self, *, class_: str, in_: int) -> str:",
            "{\"class\": class_, \"in\": in_},",
            "    def empty(self) -> str:\n        \"\"\"`sdk.t.empty`\"\"\"\n        return self._r.render(\"sdk.t.empty\")",
            "    def sun(self) -> EntityForms:",
            "class MessagesSdkHollow:",
            "    short: Optional[str] = None",
            "            short=text(\"short\"),",
            "            iast=text(\"iast\") or \"\",",
            "    gender: Optional[Gender] = None",
        ],
    );
    // An empty group still has a body after its constructor.
    assert!(text.contains("        self._r = renderer\n\n\n\n"));
    let at = |class: &str| text.find(&format!("class {class}:")).unwrap();
    assert!(at("Messages") < at("MessagesSdk"));
    assert!(at("MessagesSdk") < at("MessagesSdkT"));
    assert!(at("MessagesSdkT") < at("MessagesSdkTBikramSambat"));
}

#[test]
fn java_writes_each_shape_and_only_closed_kinds() {
    let text = java(&model(true), "com.example.messages");
    holds(
        &text,
        &[
            "\npackage com.example.messages;\n",
            "public enum Case {",
            "        /** `in`. */\n        IN_(\"in\")",
            "public enum GrahaKey {",
            "SUN(\"graha.SUN\")",
            "public String every(String a, Gender b, long c, double d, GrahaKey e, String f, String g, List<?> h, DateValue i, TimeValue j, DateTimeValue k, GhatiValue l, double m) {",
            "Map.entry(\"b\", b.value())",
            "Map.entry(\"e\", Map.of(\"$entity\", e.value()))",
            "Map.entry(\"f\", Map.of(\"$entity\", f))",
            "Map.entry(\"g\", Map.of(\"$entity\", g))",
            "Map.entry(\"h\", h)",
            "Map.entry(\"i\", Map.of(\"$date\", i.json()))",
            "Map.entry(\"j\", Map.of(\"$time\", j.json()))",
            "Map.entry(\"k\", Map.of(\"$datetime\", k.json()))",
            "Map.entry(\"l\", Map.of(\"$ghati\", l.json()))",
            "Map.entry(\"m\", Map.of(\"$instant\", m))",
            "public String keyword(String class_, long in) {",
            "return r.render(\"sdk.t.empty\", Map.of());",
            "public EntityForms sun() {",
            "public SdkTBikramSambat bikramSambat() {",
            "String iast, String name, String prose, String short_, String glyph, Gender gender, Map<String, String> forms",
            "text(forms, \"short\", null)",
            "private static Gender gender(String value) {",
        ],
    );
    // An open kind's members are a consumer's packs: no enum names them.
    lacks(&text, &["enum RuleKey"]);
}

#[test]
fn rust_writes_each_shape_with_its_field_and_value() {
    let text = rust(&model(true), RustPaths::CONSUMER);
    holds(
        &text,
        &[
            "pub enum Case {",
            "            Case::In => \"in\",",
            "impl From<Case> for teistro_intl::Value {",
            "pub mod bikram_sambat {",
            "pub struct Empty;",
            "teistro_intl::Params::new()",
            "pub a: String,",
            "pub b: crate::messages::Gender,",
            "pub c: i64,",
            "pub d: f64,",
            "pub e: teistro_core::catalogue::Graha,",
            "pub f: String,",
            "pub g: String,",
            "pub h: Vec<teistro_intl::Value>,",
            "pub i: teistro_calendar::CalendarDate,",
            "pub j: teistro_intl::ClockTime,",
            "pub k: (teistro_calendar::CalendarDate, teistro_intl::ClockTime),",
            "pub l: teistro_intl::Ghati,",
            "pub m: teistro_core::quantity::JulianDay<teistro_core::quantity::Utc>,",
            "(\"a\", teistro_intl::Value::Str(self.a.clone())),",
            "(\"b\", teistro_intl::Value::Str(self.b.key().to_string())),",
            "(\"c\", teistro_intl::Value::Int(self.c)),",
            "(\"d\", teistro_intl::Value::Num(self.d)),",
            "(\"e\", teistro_intl::Value::catalogued(self.e)),",
            "(\"f\", teistro_intl::Value::entity(&self.f)),",
            "(\"h\", teistro_intl::Value::List(self.h.clone())),",
            "(\"i\", teistro_intl::Value::Date(self.i.clone())),",
            "(\"j\", teistro_intl::Value::Time(self.j)),",
            "(\"k\", teistro_intl::Value::DateTime(self.k.0.clone(), self.k.1)),",
            "(\"l\", teistro_intl::Value::Ghati(self.l)),",
            "(\"m\", teistro_intl::Value::Instant(self.m)),",
            "pub in_: i64,",
        ],
    );
    // A consumer's module carries no examples.
    lacks(&text, &["```"]);
}

#[test]
fn the_sdk_s_rust_carries_a_run_example_where_a_sample_can_be_made() {
    let text = rust(&model(true), RustPaths::SDK);
    holds(
        &text,
        &[
            "/// use teistro_intl::messages::Case;\n///\n/// assert_eq!(Case::In.key(), \"in\");",
            "    /// assert_eq!(Case::In.key(), \"in\");",
            "/// use teistro_intl::messages::sdk::t::Short;",
            "/// let message = Short { n: 2 };",
            "/// let message = Empty;",
            "/// assert!(rendered.warnings.is_empty(), \"{:?}\", rendered.warnings);",
            // Text beside an instant is the zone, and the fields wrap.
            "        /// let message = Zoned {\n        ///     at: teistro_core::quantity::JulianDay::J2000,\n        ///     zone: String::from(\"+05:45\"),\n        /// };",
            "///     b: teistro_intl::messages::Case::In,",
            "///     d: 2.5,",
            "///     e: teistro_core::catalogue::Graha::ALL[0],",
            "///     f: String::from(\"rule.FIRST\"),",
            "///     g: String::from(\"g\"),",
            "///     h: vec![teistro_intl::Value::Str(String::from(\"one\")), teistro_intl::Value::Str(String::from(\"two\"))],",
            "///     j: teistro_intl::ClockTime { hour: 6, minute: 5, second: 0 },",
            "///     l: teistro_intl::Ghati { ghati: 12, pala: 30, vipala: 0 },",
            "///     q: String::from(\"q\"),",
            "/// // An entity's record may be a pack's; without it the message says what is missing.",
        ],
    );
    // A date has no sample, and nor has a context the model does not
    // declare: those messages go without an example.
    for bare in ["every", "orphan"] {
        assert!(
            text.contains(&format!(
                "        /// The message `sdk.t.{bare}`.\n        #[derive(Clone, Debug, PartialEq)]"
            )),
            "{bare}"
        );
    }
}

#[test]
fn a_model_without_gender_types_no_gender_anywhere() {
    let model = model(false);
    let surfaces = [
        typescript(&model),
        javascript(&model),
        dart(&model),
        python(&model),
        java(&model, "p"),
    ];
    for surface in &surfaces {
        lacks(
            surface,
            &[
                "gender?: Gender",
                "gender: forms.gender",
                "this.gender",
                "Gender? gender",
                "Optional[Gender]",
                "Gender gender",
            ],
        );
    }
    // A model naming no kind declares no `EntityKey` union.
    let bare = Model {
        kinds: BTreeMap::new(),
        ..model
    };
    lacks(&typescript(&bare), &["export type EntityKey ="]);
}

// ── The model from sources ───────────────────────────────────────────────

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("intl-generators")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A tree of the locales named, each with the SDK's own meta file and the
/// files given.
fn tree(name: &str, locales: &[(&str, &[(&str, &str)])]) -> PathBuf {
    let root = scratch(name);
    for (tag, files) in locales {
        let dir = root.join(tag);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(sdk_root().join(tag).join(META_FILE), dir.join(META_FILE)).unwrap();
        for (file, text) in *files {
            std::fs::write(dir.join(file), text).unwrap();
        }
    }
    root
}

#[test]
fn a_closed_kind_a_message_names_is_seeded_from_the_catalogue() {
    let root = tree(
        "seeded",
        &[(
            "en-Latn",
            &[
                (
                    "sdk.t.json",
                    r#"{"state": "{$s :entity kind=state}", "rule": "{$r :entity kind=rule}", "odd": "{$o :entity kind=nonsense}", "graha": "{$g :entity kind=graha}"}"#,
                ),
                ("sdk.entity.json", r#"{"graha": {"SUN": {"name": "Sun"}}}"#),
            ],
        )],
    );
    let tree = Tree::load(&root).unwrap();
    let model = Model::of(tree.base().unwrap()).unwrap();
    // The catalogue's members, though the locale names none of them.
    let state = &model.kinds["state"];
    assert!(state.iter().any(|key| key == "COMBUST"), "{state:?}");
    // The locale's own list stands for a kind it names.
    assert_eq!(model.kinds["graha"], ["SUN"]);
    // An open kind and a name that is no kind are left alone.
    assert!(!model.kinds.contains_key("rule"));
    assert!(!model.kinds.contains_key("nonsense"));
    assert_eq!(
        model.keys,
        ["sdk.t.state", "sdk.t.rule", "sdk.t.odd", "sdk.t.graha"]
    );
}

#[test]
fn a_message_that_does_not_parse_is_refused_with_its_key() {
    let root = tree(
        "unparsed",
        &[("en-Latn", &[("sdk.t.json", r#"{"bad": "{$x"}"#)])],
    );
    let tree = Tree::load(&root).unwrap();
    let error = Model::of(tree.base().unwrap()).unwrap_err();
    assert!(error.starts_with("sdk.t.bad: "), "{error}");
    assert!(
        Model::of_tree(&tree)
            .unwrap_err()
            .starts_with("sdk.t.bad: ")
    );
}

#[test]
fn a_tree_widens_only_from_another_locale_s_engine_patterns() {
    let base = [(
        "sdk.calendar.json",
        r#"{"time": {"clock": "{$hour}:{$minute}"}, "plain": "{$x}"}"#,
    )];
    // Nepali reads the part of the day and a name the engine does not
    // supply; only the first is taken. A plain message and an entity are
    // never read.
    let nepali = [
        (
            "sdk.calendar.json",
            r#"{"time": {"clock": "{$dayPeriod} {$hour}:{$minute} {$extra}"}, "plain": "{$x} {$y}"}"#,
        ),
        ("sdk.entity.json", r#"{"graha": {"SUN": {"name": "सूर्य"}}}"#),
    ];
    let root = tree("widened", &[("en-Latn", &base), ("ne-Deva-NP", &nepali)]);
    let model = Model::of_tree(&Tree::load(&root).unwrap()).unwrap();
    let Some(Node::Group(sdk)) = model.root.children.get("sdk") else {
        unreachable!("an `sdk` group")
    };
    let Some(Node::Group(calendar)) = sdk.children.get("calendar") else {
        unreachable!("a `calendar` group")
    };
    let Some(Node::Group(time)) = calendar.children.get("time") else {
        unreachable!("a `time` group")
    };
    let Some(Node::Message(clock)) = time.children.get("clock") else {
        unreachable!("the clock")
    };
    let names: Vec<&str> = clock.params.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["dayPeriod", "hour", "minute"]);
    let Some(Node::Message(plain)) = calendar.children.get("plain") else {
        unreachable!("the plain message")
    };
    assert_eq!(plain.params.len(), 1);

    // Another locale's engine pattern that does not parse names the locale.
    let broken = [("sdk.calendar.json", r#"{"time": {"clock": "{$hour"}}"#)];
    let root = tree(
        "widened-broken",
        &[("en-Latn", &base), ("ne-Deva-NP", &broken)],
    );
    let error = Model::of_tree(&Tree::load(&root).unwrap()).unwrap_err();
    assert!(
        error.starts_with("ne-Deva-NP: sdk.calendar.time.clock: "),
        "{error}"
    );

    // And a tree with no base locale has no model.
    let root = tree("baseless", &[("ne-Deva-NP", &[])]);
    assert_eq!(
        Model::of_tree(&Tree::load(&root).unwrap()).unwrap_err(),
        "no base locale"
    );
}

// ── Signatures ───────────────────────────────────────────────────────────

fn meta() -> Meta {
    let text = std::fs::read_to_string(sdk_root().join("en-Latn").join(META_FILE)).unwrap();
    let mut meta: Meta = serde_json::from_str(&text).unwrap();
    meta.contexts.insert(
        String::from("tense"),
        vec![String::from("past"), String::from("future")],
    );
    meta
}

#[test]
fn a_signature_reads_markup_options_literals_and_locals() {
    let meta = meta();
    let sig = |source: &str| signature(&parse(source).unwrap(), &meta);

    // A markup option that is a variable is a text parameter.
    let markup = sig("{#link href=$url}here{/link}");
    assert_eq!(markup.params["url"], ParamType::String);
    assert!(markup.markup.contains("link"));

    // A local declared from another local takes nothing from the caller.
    let locals = sig(".input {$n :integer} .local $a = {$n} .local $b = {$a} {{{$b}}}");
    assert_eq!(locals.params.len(), 1);
    assert_eq!(locals.params["n"], ParamType::Integer);

    // A literal under a function that is neither `:msg` nor `:entity`
    // links nothing and names no entity.
    let literal = sig("{|3| :number} {|graha.SUN| :entity} {|sdk.t.x| :msg}");
    assert!(literal.params.is_empty());
    assert_eq!(literal.links, ["sdk.t.x"]);
    assert_eq!(literal.entities, ["graha.SUN"]);
}

#[test]
fn a_selector_is_a_context_only_where_its_keys_fit_one() {
    let meta = meta();
    let sig = |source: &str| signature(&parse(source).unwrap(), &meta);

    // Named for no context, a selector whose keys fit one is that context.
    let fitted = sig(".input {$when :string} .match $when past {{a}} future {{b}} * {{c}}");
    assert_eq!(
        fitted.params["when"],
        ParamType::Context(String::from("tense"))
    );
    // Keys that fit no context, and no keys at all, leave it text.
    let loose = sig(".input {$x :string} .match $x past {{a}} m {{b}} * {{c}}");
    assert_eq!(loose.params["x"], ParamType::String);
    let catch_all = sig(".input {$x :string} .match $x * {{c}}");
    assert_eq!(catch_all.params["x"], ParamType::String);
    // A selector on a local made from a literal is a context with no
    // parameter behind it.
    let literal = sig(".local $g = {|m| :string} .match $g m {{a}} * {{b}}");
    assert!(literal.params.is_empty(), "{:?}", literal.params);
    assert_eq!(
        literal.selectors[0].kind,
        ParamType::Context(String::from("gender"))
    );
}

#[test]
fn an_entity_of_a_kind_agrees_only_with_that_kind_or_any() {
    assert!(entity("graha").agrees_with(&entity("graha")));
    assert!(!entity("graha").agrees_with(&entity("rashi")));
    assert!(entity("graha").agrees_with(&ParamType::Entity(None)));
    assert!(ParamType::String.agrees_with(&ParamType::Context(String::from("gender"))));
    assert!(!ParamType::List.agrees_with(&ParamType::String));
}
