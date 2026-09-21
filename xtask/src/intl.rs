//! The intl generator and its gate: `i18n/` in, the SDK's typed messages
//! (`crates/intl/src/messages.rs`) out. `gen intl` validates the sources
//! and writes the accessors; `check-intl` validates and regenerates in
//! memory, failing on any validation error or any difference, so the
//! checked-in surface can never drift from the sources.

use std::collections::BTreeSet;
use std::path::Path;

use teistro_core::catalogue::{Kind, key_of};
use teistro_core::key::KeyId;

use teistro_intl::derive::{Casing, derive, overrides_of};
use teistro_intl::generate::{
    Model, RustPaths, dart, javascript, python, rust, typescript_declarations,
};
use teistro_intl::source::{Completeness, Tree};
use teistro_intl::validate;

use crate::generated::{Output, check, write};

const MESSAGES: &str = "crates/intl/src/messages.rs";
/// The typed accessors each binding ships: the same model, in the shape
/// its language reads (`03-design/intl-engine-and-packs.md`).
const NODE_MESSAGES: &str = "bindings/node/lib/messages.js";
const NODE_MESSAGE_TYPES: &str = "bindings/node/lib/messages.d.ts";
const DART_MESSAGES: &str = "bindings/dart/lib/src/messages.dart";
const PYTHON_MESSAGES: &str = "bindings/python/teistro/messages.py";
/// The locale derived from another by transliteration, and the one it is
/// derived from (`03-design/intl-engine-and-packs.md`, §3).
const DERIVED: (&str, &str) = ("sa-Deva", "sa-Latn");

fn outputs(root: &Path) -> Vec<Output> {
    let tree = Tree::load(&root.join("i18n")).expect("the i18n/ sources load");
    let report = validate::validate(&tree);
    if report.passed() {
        eprintln!("{}", report.markdown().lines().next().unwrap_or_default());
    } else {
        eprintln!("{}", report.markdown());
        panic!("the i18n/ sources do not validate");
    }
    let base = tree.base().expect("the base locale");
    let model = Model::of(base).expect("every base message parses");
    let overrides = overrides_of(&root.join("i18n"), DERIVED.1)
        .unwrap_or_else(|e| panic!("the derived locale's overrides load: {e}"));
    let derived = derive(&tree, DERIVED.0, DERIVED.1, &overrides, Casing::Names)
        .unwrap_or_else(|e| panic!("{} derives from {}: {e}", DERIVED.1, DERIVED.0));
    if !derived.stale.is_empty() {
        println!(
            "FAIL  i18n/{}/_overrides.json corrects entities the sources no longer have: {}",
            DERIVED.1,
            derived.stale.join(", ")
        );
    }
    let mut outputs: Vec<Output> = derived
        .files
        .iter()
        .map(|(path, text)| Output::new(format!("i18n/{}", path.display()), text.clone()))
        .collect();
    outputs.extend([
        Output::new(MESSAGES, rust(&model, RustPaths::SDK)),
        Output::new(NODE_MESSAGES, javascript(&model)),
        Output::new(NODE_MESSAGE_TYPES, typescript_declarations(&model)),
        Output::new(DART_MESSAGES, dart(&model)),
        Output::new(PYTHON_MESSAGES, python(&model)),
    ]);
    outputs
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let written = outputs(root);
    let failures = check(root, &written, "cargo xtask gen intl");
    let tree = Tree::load(&root.join("i18n")).expect("the i18n/ sources load");
    let unnamed = unnamed_members(&tree, &document_kinds());
    let naming = named_or_listed(&unnamed, UNNAMED);
    let keywords = dart_parses(&written);
    i32::from(failures != 0 || naming != 0 || keywords != 0)
}

/// Every Dart keyword, as the language lists them: a word from this list
/// cannot be an identifier, whatever else it is.
const DART_KEYWORDS: [&str; 33] = [
    "assert", "break", "case", "catch", "class", "const", "continue", "default", "do", "else",
    "enum", "extends", "false", "final", "finally", "for", "if", "in", "is", "new", "null",
    "rethrow", "return", "super", "switch", "this", "throw", "true", "try", "var", "void", "while",
    "with",
];

/// That no identifier in the generated Dart is a Dart keyword.
///
/// A message's slot names the Dart parameter, and `sdk.reading.lifeClass`
/// selects on a slot called `class`: the emitter wrote `required String
/// class`, which does not parse. Nothing here saw it, because a generated
/// file being **up to date** says nothing about its compiling, and the gate
/// that compiles Dart runs in the verify matrix rather than in fast-check.
/// So the class is held where the file is written: the identifiers are read
/// back out of what was just generated, and a keyword among them fails.
fn dart_parses(written: &[Output]) -> i32 {
    let Some(dart) = written.iter().find(|out| out.path == DART_MESSAGES) else {
        println!("FAIL  {DART_MESSAGES} was not generated");
        return 1;
    };
    let mut wrong = Vec::new();
    let mut read = 0usize;
    for line in dart.text.lines() {
        // `required <type> <name>` in a parameter list, and `get <name>`.
        let names = line
            .split("required ")
            .skip(1)
            .filter_map(|piece| piece.split([',', '}']).next())
            .filter_map(|decl| decl.split_whitespace().next_back())
            .chain(
                line.split(" get ")
                    .skip(1)
                    .filter_map(|after| after.split_whitespace().next()),
            );
        for name in names {
            read += 1;
            if DART_KEYWORDS.contains(&name) {
                wrong.push(format!("`{name}` in `{}`", line.trim()));
            }
        }
    }
    if wrong.is_empty() {
        println!("ok    {DART_MESSAGES}: {read} identifiers, none a Dart keyword");
        return 0;
    }
    println!("FAIL  {DART_MESSAGES} writes Dart keywords as identifiers:");
    for one in &wrong {
        println!("      {one}");
    }
    1
}

/// The members a chart document can carry that no strict locale names
/// yet, and why each kind has none: every one of them lacks a vetted
/// source, and the policy refuses a translated stub
/// (`docs/03-design/entity-names.md` §4).
///
/// The list fails both ways. A member missing a name that is not here
/// fails, and so does one here that has since been named, so it can only
/// shrink.
const UNNAMED: &[(&str, &str, &[&str])] = &[
    (
        "dasha_system",
        "the baseline engine vets names for the 18 systems it implements, and none for these",
        &[
            "SHODASHOTTARI",
            "SHATTRIMSHA_SAMA",
            "SHASHTIHAYANI",
            "TITHI_ASHTOTTARI",
            "TITHI_YOGINI",
            "YOGA_VIMSHOTTARI",
            "KARANA_CHATURASHITI",
            "NAISARGIKA",
            "TARA",
            "KARAKA",
            "AAYU",
            "ASHTAKAVARGA",
            "PANCHASWARA",
            "STHIRA",
            "SUDASA",
            "VARNADA",
            "YOGARDHA",
            "SUDARSHANA_CHAKRA",
            "PATYAYINI",
            "MUDDA",
            "VARSHA_NARAYANA",
            "VARSHA_YOGINI",
        ],
    ),
    (
        "avastha_cheshta",
        "no vetted name table for the Sayanadi's sub-states",
        &["DRISHTI", "CHESHTA", "VICHESHTA"],
    ),
    (
        "avastha_sayanadi",
        "the vetted avastha tables stop at the Baladi, Jagradadi, Deeptadi and Lajjitadi",
        &[
            "SHAYANA",
            "UPAVESHANA",
            "NETRAPANI",
            "PRAKASHANA",
            "GAMANA",
            "AGAMANA",
            "SABHA",
            "AGAMA",
            "BHOJANA",
            "NRITYALIPSA",
            "KAUTUKA",
            "NIDRA",
        ],
    ),
    (
        "calendar",
        "no vetted name table for the calendar systems",
        &[
            "GREGORIAN",
            "JULIAN",
            "MIXED",
            "ISO_WEEK",
            "BIKRAM_SAMBAT",
            "INDIAN_LUNISOLAR",
        ],
    ),
    (
        "chart_kind",
        "no vetted name table for the kinds of chart",
        &[
            "NATAL",
            "TRANSIT",
            "EVENT",
            "PRASHNA",
            "RETURN",
            "RELOCATED",
            "COMPOSITE",
        ],
    ),
    (
        "graha",
        "the vetted graha table stops at the nine grahas",
        &["URANUS", "NEPTUNE", "PLUTO"],
    ),
    (
        "nature",
        "no vetted name table for a graha's benefic, malefic or neutral nature",
        &["BENEFIC", "MALEFIC", "NEUTRAL"],
    ),
    (
        "point",
        "the vetted table names the upagrahas only; the angles, special lagnas, sphutas, yogi points and arudhas have none",
        &[
            "MC",
            "DESCENDANT",
            "IC",
            "BHAVA_LAGNA",
            "HORA_LAGNA",
            "GHATI_LAGNA",
            "VIGHATI_LAGNA",
            "VARNADA_LAGNA",
            "SREE_LAGNA",
            "INDU_LAGNA",
            "PRANAPADA_LAGNA",
            "KALA",
            "MRITYU",
            "ARDHAPRAHARA",
            "YAMAGHANTAKA",
            "BHRIGU_BINDU",
            "YOGI",
            "AVAYOGI",
            "SAHAYOGI",
            "TRISPHUTA",
            "CHATUSSPHUTA",
            "PANCHASPHUTA",
            "PRANA_SPHUTA",
            "DEHA_SPHUTA",
            "MRITYU_SPHUTA",
            "BEEJA_SPHUTA",
            "KSHETRA_SPHUTA",
            "KARAKAMSHA",
            "SWAMSHA",
            "A1",
            "A2",
            "A3",
            "A4",
            "A5",
            "A6",
            "A7",
            "A8",
            "A9",
            "A10",
            "A11",
            "A12",
        ],
    ),
    (
        "vaiseshikamsa",
        "no vetted name table for the Vaiseshikamsa's designations",
        &[
            "KIMSHUKA",
            "VYANJANA",
            "CHAMARA",
            "CHATRA",
            "KUNDALA",
            "MUKUTA",
            "PARIJATA",
            "UTTAMA",
            "GOPURA",
            "SIMHASANA",
            "PARAVATA",
            "DEVALOKA",
            "BRAHMALOKA",
            "SHAKRAVAHANA",
            "SHRIDHAMA",
            "BHEDAKA",
            "KUSUMA",
            "NAGAPUSHPA",
            "KANDUKA",
            "KERALA",
            "KALPAVRIKSHA",
            "CHANDANAVANA",
            "PURNACHANDRA",
            "UCHCHAISHRAVA",
            "DHANVANTARI",
            "SURYAKANTA",
            "VIDRUMA",
            "CHAKRASIMHASANA",
            "GOLOKA",
            "SHRIVALLABHA",
        ],
    ),
    (
        "varga",
        "no vetted name table for the divisional charts",
        &[
            "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "D10", "D11", "D12", "D16",
            "D20", "D24", "D27", "D30", "D40", "D45", "D60", "D150",
        ],
    ),
];

/// The catalogue kinds a chart document can carry, read from its schema's
/// `x-teistro-kind` keywords rather than kept as a second list.
fn document_kinds() -> BTreeSet<String> {
    let schema: serde_json::Value = serde_json::from_str(teistro_serial::schema::DOCUMENT)
        .expect("the document schema is JSON");
    schema["$defs"]
        .as_object()
        .into_iter()
        .flat_map(|defs| defs.values())
        .filter_map(|def| def.get(teistro_core::catalogue::KIND_KEYWORD)?.as_str())
        .map(str::to_owned)
        .collect()
}

/// Every `(kind, key)` of `kinds` that some strict locale leaves unnamed.
fn unnamed_members(tree: &Tree, kinds: &BTreeSet<String>) -> BTreeSet<(String, String)> {
    let strict: Vec<_> = tree
        .locales
        .values()
        .filter(|locale| locale.meta.completeness == Completeness::Strict)
        .collect();
    let mut unnamed = BTreeSet::new();
    for name in kinds {
        let Some(kind) = Kind::from_name(name) else {
            continue;
        };
        for id in 0..u16::try_from(kind.count()).unwrap_or(u16::MAX) {
            let Some(key) = key_of(KeyId::new(kind, id)) else {
                continue;
            };
            let full = format!("{name}.{key}");
            if strict.iter().any(|locale| locale.entity(&full).is_none()) {
                unnamed.insert((name.clone(), key.to_owned()));
            }
        }
    }
    unnamed
}

/// Holds the unnamed members to the list, both ways; the number of
/// disagreements.
fn named_or_listed(
    unnamed: &BTreeSet<(String, String)>,
    listed: &[(&str, &str, &[&str])],
) -> usize {
    let listed: BTreeSet<(String, String)> = listed
        .iter()
        .flat_map(|(kind, _, keys)| {
            keys.iter()
                .map(|key| ((*kind).to_owned(), (*key).to_owned()))
        })
        .collect();
    let mut failures = 0;
    for (kind, key) in unnamed.difference(&listed) {
        println!(
            "FAIL  {kind}.{key}: a document can carry it and a strict locale does not name it; \
             name it from a vetted source, or list it in xtask/src/intl.rs with the reason"
        );
        failures += 1;
    }
    for (kind, key) in listed.difference(unnamed) {
        println!("FAIL  {kind}.{key}: named now, so take it off the unnamed list");
        failures += 1;
    }
    if failures == 0 {
        println!(
            "ok    every catalogue member a document carries is named in every strict locale, \
             but {} listed",
            listed.len()
        );
    }
    failures
}
