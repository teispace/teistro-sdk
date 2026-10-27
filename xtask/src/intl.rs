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
    Model, RustPaths, dart, java, javascript, python, rust, typescript_declarations,
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
/// Java's, in a source root of its own so neither `gen ffi`'s pruning nor
/// a hand-written file can meet it.
const JAVA_MESSAGES: &str = "bindings/java/messages/com/teispace/teistro/messages/Messages.java";
const JAVA_MESSAGES_PACKAGE: &str = "com.teispace.teistro.messages";
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
        Output::new(JAVA_MESSAGES, java(&model, JAVA_MESSAGES_PACKAGE)),
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
    let unnamed = unnamed_members(&tree, &catalogue_kinds());
    let naming = named_or_listed(&unnamed, UNNAMED);
    let festivals = every_festival_is_named(&tree);
    let keywords = dart_parses(&written);
    let declared = every_key_type_is_declared(&written);
    i32::from(failures != 0 || naming != 0 || festivals != 0 || keywords != 0 || declared != 0)
}

/// That every rule key a shipped festival pack answers with is named in
/// every strict locale's `festival` group: an observance carries its rule's
/// key, and a reader shows the name. A pack's rules are not a catalogue
/// kind, so [`unnamed_members`] does not see them; six fitted rules once
/// shipped with no name in any locale and every gate green.
fn every_festival_is_named(tree: &Tree) -> usize {
    let strict: Vec<_> = tree
        .locales
        .iter()
        .filter(|(_, locale)| locale.meta.completeness == Completeness::Strict)
        .collect();
    let keys: BTreeSet<String> = teistro::FestivalPack::ALL
        .into_iter()
        .flat_map(|pack| {
            let rules = pack.rules().into_iter().map(|rule| rule.key);
            let ekadashis = pack.ekadashis().into_iter().map(|rule| rule.key);
            let following = pack.following().into_iter().map(|rule| rule.key);
            rules.chain(ekadashis).chain(following).collect::<Vec<_>>()
        })
        .collect();
    let mut failures = 0;
    for key in &keys {
        for (tag, locale) in &strict {
            if locale.entity(&format!("festival.{key}")).is_none() {
                println!(
                    "FAIL  festival.{key}: a shipped pack answers with it and {tag} does not name it"
                );
                failures += 1;
            }
        }
    }
    if failures == 0 {
        println!(
            "ok    every rule key of every shipped festival pack ({}) is named in every strict locale",
            keys.len()
        );
    }
    failures
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

/// Every generated file, and how it declares a kind's key type.
const KEY_TYPES: [(&str, &str); 3] = [
    (NODE_MESSAGE_TYPES, "export type "),
    (DART_MESSAGES, "enum "),
    (PYTHON_MESSAGES, "class "),
];

/// That no generated file names a `<Kind>Key` type it does not declare.
///
/// The key types come from `sdk.entity`, which names a kind's **members**;
/// the slot types come from a message's `kind=` option. Those two sets can
/// differ, and on 2026-09-22 they did: `sdk.phala.state` renders the
/// `phala` form of `state.COMBUST`, a closed catalogue kind no locale
/// names, and the TypeScript emitter wrote `state: StateKey` against a
/// type nothing declared. `check-intl` was green on it because the file
/// was **up to date**, and the Node type-check that catches it runs in the
/// verify matrix — 40 minutes later, on five platforms at once.
///
/// So the class is held where the file is written, as the Dart keyword is:
/// the key types are read back out of what was just generated and a
/// reference to an undeclared one fails. The generator's own fix was to
/// seed a closed kind's members from the catalogue, which is where a
/// member list belongs; this is what would have caught it in three
/// seconds.
fn every_key_type_is_declared(written: &[Output]) -> i32 {
    let mut wrong = 0;
    for (path, declares) in KEY_TYPES {
        let Some(file) = written.iter().find(|out| out.path == path) else {
            println!("FAIL  {path} was not generated");
            wrong += 1;
            continue;
        };
        let declared: BTreeSet<&str> = file
            .text
            .lines()
            .filter_map(|line| line.trim().strip_prefix(declares))
            .filter_map(|rest| {
                rest.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .next()
            })
            .filter(|name| name.ends_with("Key"))
            .collect();
        // Every `<Pascal>Key` word the file uses, however it uses it.
        let mut referenced: BTreeSet<&str> = BTreeSet::new();
        for line in file.text.lines() {
            for word in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
                if word.ends_with("Key")
                    && word.len() > 3
                    && word.starts_with(|c: char| c.is_ascii_uppercase())
                {
                    referenced.insert(word);
                }
            }
        }
        for name in referenced.difference(&declared) {
            println!("FAIL  {path} names `{name}` and declares no such type");
            wrong += 1;
        }
    }
    if wrong == 0 {
        println!("ok    every key type a generated file names is declared in it");
    }
    wrong
}

/// The catalogue members no strict locale names yet, and why each kind has
/// none: every one of them lacks a vetted source, and the policy refuses a
/// translated stub (`docs/03-design/entity-names.md` §4). Every kind is
/// held, not only those a chart document carries: a day page, a match or
/// a star list hands a reader its keys as surely (C682).
///
/// The list fails both ways. A member missing a name that is not here
/// fails, and so does one here that has since been named, so it can only
/// shrink.
/// Readable from [`crate::interpret`], where the composers' queue cites a
/// kind as the reason a section cannot be said: a reason that names a kind
/// no longer on this list fails there, so the two cannot drift.
pub(crate) const UNNAMED: &[(&str, &str, &[&str])] = &[
    (
        "dasha_system",
        "BPHS ch. 46 names the 36-year system only by its length (\"षड्त्रिंशवत्सरा\", v. 5), \
         lists the three lifespan dashas apart (v. 10) rather than one Aayu, and has no tithi, \
         yoga, karana or Sudasa system; no text naming those has been read",
        &[
            "SHATTRIMSHA_SAMA",
            "TITHI_ASHTOTTARI",
            "TITHI_YOGINI",
            "YOGA_VIMSHOTTARI",
            "KARANA_CHATURASHITI",
            "AAYU",
            "SUDASA",
        ],
    ),
    (
        "dasha_system",
        "the Tajika annual dashas are outside BPHS, and no Tajika text naming them in \
         Devanagari has been read",
        &["PATYAYINI", "MUDDA", "VARSHA_NARAYANA", "VARSHA_YOGINI"],
    ),
    (
        "dasha_system",
        "The Hellenistic time lords have no vetted Nepali or Hindi name, and a strict \
         locale binds the English to one",
        &[
            "RELEASING_FORTUNE",
            "RELEASING_DAIMON",
            "PROFECTION",
            "FIRDARIA",
            "DECENNIALS",
        ],
    ),
    (
        "calendar",
        "no vetted name table for the calendar systems: Bikram Sambat's Devanagari is \
         printed several ways (संवत्, सम्वत्, वि.सं.) and the committee's own masthead has \
         not been read, and the others have no Nepali source at all (the regional \
         solar calendars' month names neither: their months render by number)",
        &[
            "GREGORIAN",
            "JULIAN",
            "MIXED",
            "ISO_WEEK",
            "BIKRAM_SAMBAT",
            "INDIAN_LUNISOLAR",
            "INDIAN_NATIONAL",
            "HIJRI_TABULAR",
            "TIBETAN_PHUGPA",
            "BANGLADESHI",
            "TAMIL_SOLAR",
            "BENGAL_SOLAR",
            "ODIA_SOLAR",
            "MALAYALAM_SOLAR",
        ],
    ),
    (
        "era",
        "no vetted name for the Hijri era in Nepali or Hindi; the English \
         date forms write AH themselves, as CLDR's do",
        &["HIJRI"],
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
        "the vetted graha table stops at the nine grahas, and Nepali usage for the outer \
         three splits (Nepali Wikipedia's अरुणग्रह, वरुणग्रह, यमग्रह against the \
         transliterated युरेनस, नेप्च्युन, प्लुटो) with no committee, dictionary or \
         textbook source found to decide it",
        &["URANUS", "NEPTUNE", "PLUTO"],
    ),
    (
        "nature",
        "BPHS 3.11 sorts the grahas into क्रूर and सौम्य only, and no text read names a \
         neutral class",
        &["NEUTRAL"],
    ),
    (
        "point",
        "the angles other than the lagna: BPHS names the kendras as houses, not as points, \
         and no Nepali source for them has been read",
        &["MC", "DESCENDANT", "IC"],
    ),
    (
        "point",
        "absent from BPHS, and the texts that teach them (Prasna Marga for the sphutas, \
         the Jaimini commentaries, the yogi point's sources) have not been read",
        &[
            "VIGHATI_LAGNA",
            "SREE_LAGNA",
            "INDU_LAGNA",
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
        ],
    ),
    (
        "point",
        "the BPHS e-text misprints it (\"यमघण्ट्कः\", 3.69), so a second print must give \
         the spelling",
        &["YAMAGHANTAKA"],
    ),
    (
        "point",
        "BPHS uses स्वांश for a graha's own division, not as the name of a point",
        &["SWAMSHA"],
    ),
    (
        "point",
        "BPHS ch. 29 calls each of these the pada of its bhava and prints a name only for \
         the first (लग्नपद) and the twelfth (उपपद, 30.3)",
        &["A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11"],
    ),
    (
        "vaiseshikamsa",
        "the BPHS e-text prints कुमुम (6.48), शक्रसिंहासन (6.51) and स्रीवल्लभ (6.51) \
         against the catalogue's Kusuma, Chakrasimhasana and Shrivallabha, so a second \
         print must decide each spelling",
        &["KUSUMA", "CHAKRASIMHASANA", "SHRIVALLABHA"],
    ),
    (
        "varga",
        "BPHS 6.3-4 lists sixteen vargas and these five are not among them; D1 is, as \
         क्षेत्र, while a chart prints it as the rashi or lagna chart, which is a reader's \
         word rather than a source's",
        &["D1", "D5", "D6", "D8", "D11", "D150"],
    ),
    (
        "tithi_class",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["BHADRA", "JAYA", "NANDA", "PURNA", "RIKTA"],
    ),
    (
        "modality",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["CHARA", "DWISWABHAVA", "STHIRA"],
    ),
    (
        "muhurta_nature",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &[
            "CHARA", "DHRUVA", "KSHIPRA", "MISHRA", "MRIDU", "TIKSHNA", "UGRA",
        ],
    ),
    (
        "guna",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["RAJAS", "SATTVA", "TAMAS"],
    ),
    (
        "gender",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["FEMALE", "MALE", "NEUTRAL"],
    ),
    (
        "rising",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["PRISHTODAYA", "SIRSHODAYA", "UBHAYODAYA"],
    ),
    (
        "blackout_kind",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["ECLIPSE_STAR", "ECLIPSE_VEDHA", "KHARMAS"],
    ),
    (
        "lunar_eclipse_kind",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["PENUMBRAL"],
    ),
    (
        "solar_eclipse_kind",
        "a day page shows these; names are proposed in `amrit-kalam-and-lagnas.md` §3.2 from the \
         texts' own Sanskrit terms, and wait there for a reviewer of each language",
        &["ANNULAR", "HYBRID"],
    ),
    (
        "auspiciousness",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &["AUSPICIOUS", "HIGHLY_INAUSPICIOUS", "INAUSPICIOUS", "MIXED"],
    ),
    (
        "sex",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &["FEMALE", "MALE"],
    ),
    (
        "parity",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &["EVEN", "ODD"],
    ),
    (
        "deity",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &[
            "CHANDRA",
            "DURGA",
            "KAMADEVA",
            "KARTIKEYA",
            "SHANI",
            "SHIVA",
            "SHUKRA",
            "SURYA",
            "TVASHTA",
            "VIDHATA",
        ],
    ),
    (
        "koota",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &[
            "BHAKOOT",
            "GANA",
            "GRAHA_MAITRI",
            "MAHENDRA",
            "NADI",
            "RAJJU",
            "STREE_DEERGHA",
            "TARA",
            "VARNA",
            "VASHYA",
            "VEDHA",
            "YONI",
        ],
    ),
    (
        "state",
        "a reading or a match shows these, and no name table in Devanagari from a rank-1 or \
         rank-2 source has been read for them",
        &[
            "COMBUST",
            "ECLIPSED",
            "GANDANTA",
            "MARANA_KARAKA_STHANA",
            "MRITYU_BHAGA",
            "PLANETARY_WAR",
            "PUSHKARA_BHAGA",
            "PUSHKARA_NAVAMSA",
            "RETROGRADE",
            "SANDHI",
            "STATIONARY",
            "VARGOTTAMA",
        ],
    ),
    (
        "ayanamsha_category",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &[
            "EPOCH",
            "GALACTIC",
            "HISTORICAL",
            "STAR_ANCHORED",
            "VEDIC_CLASSICAL",
            "VEDIC_STANDARD",
            "WESTERN_SIDEREAL",
        ],
    ),
    (
        "bala_scheme",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &["PARASHARA", "PARASHARA_EXTENDED"],
    ),
    (
        "body_class",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &["LUMINARY", "NODE", "OUTER", "PLANET"],
    ),
    (
        "chart_layout",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &[
            "EAST_INDIAN",
            "NEPALI_LOTUS",
            "NORTH_INDIAN",
            "SOUTH_INDIAN",
            "SUDARSHAN_CHAKRA",
            "WESTERN_WHEEL",
        ],
    ),
    (
        "dasha_family",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &[
            "COMPOSITION",
            "DECENNIALS",
            "FIRDARIA",
            "GRAHA",
            "KALACHAKRA",
            "OWN",
            "PROFECTION",
            "RASHI",
            "RELEASING",
            "SCALE",
            "UDU",
        ],
    ),
    (
        "degeneracy",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &["NONE", "POLAR_UNDEFINED"],
    ),
    (
        "point_family",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &[
            "ANGLE",
            "ARUDHA",
            "JAIMINI",
            "MIDPOINT",
            "SPECIAL_LAGNA",
            "SPHUTA",
            "UPAGRAHA_DAY",
            "UPAGRAHA_SOLAR",
        ],
    ),
    (
        "star_class",
        "the SDK's own classifications rather than any text's: an English name would be the \
         SDK's coinage, and a strict locale would then bind a Nepali one to it",
        &["DIRECTION", "RADIO_SOURCE", "STAR"],
    ),
    (
        "star",
        "the IAU's English proper names would bind a Nepali name to each star, since a strict \
         locale carries every key the base names, and no star catalogue in Nepali has been read",
        &[
            "ACHERNAR",
            "ACRUX",
            "ADHARA",
            "AIN",
            "ALBALDAH",
            "ALBIREO",
            "ALCYONE",
            "ALDEBARAN",
            "ALDERAMIN",
            "ALGEDI",
            "ALGENIB",
            "ALGIEBA",
            "ALGOL",
            "ALGORAB",
            "ALHENA",
            "ALIOTH",
            "ALJANAH",
            "ALKAID",
            "ALMACH",
            "ALNAIR",
            "ALNASL",
            "ALNILAM",
            "ALNITAK",
            "ALPHARD",
            "ALPHECCA",
            "ALPHERATZ",
            "ALSEPHINA",
            "ALSHAIN",
            "ALTAIR",
            "ALUDRA",
            "ANKAA",
            "ANTARES",
            "ARCTURUS",
            "ASCELLA",
            "ASELLUS_AUSTRALIS",
            "ASHLESHA",
            "ASPIDISKE",
            "ATRIA",
            "AVIOR",
            "BELLATRIX",
            "BETELGEUSE",
            "BHARANI",
            "CANOPUS",
            "CAPELLA",
            "CAPH",
            "CASTOR",
            "CHAMUKUY",
            "DENEB",
            "DENEBOLA",
            "DIPHDA",
            "DSCHUBBA",
            "DUBHE",
            "ELNATH",
            "ENIF",
            "ERRAI",
            "FANG",
            "FOMALHAUT",
            "GACRUX",
            "GALACTIC_POLE",
            "GALACTIC_POLE_IAU1958",
            "HADAR",
            "HAMAL",
            "HEZE",
            "HYDOR",
            "IZAR",
            "KAUS_AUSTRALIS",
            "KAUS_BOREALIS",
            "KAUS_MEDIA",
            "KITALPHA",
            "KORNEPHOROS",
            "MARFIK",
            "MARKAB",
            "MEISSA",
            "MENKALINAN",
            "MENKAR",
            "MENKENT",
            "MERAK",
            "MIAPLACIDUS",
            "MIMOSA",
            "MINTAKA",
            "MIRACH",
            "MIRFAK",
            "MIZAR",
            "NAOS",
            "NIHAL",
            "NUNKI",
            "PEACOCK",
            "PHECDA",
            "POLARIS",
            "POLLUX",
            "PROCYON",
            "RASALGETHI",
            "RASALHAGUE",
            "REGULUS",
            "REVATI",
            "RIGEL",
            "RIGIL_KENTAURUS",
            "ROTANEV",
            "SABIK",
            "SADALBARI",
            "SADALMELIK",
            "SADALSUUD",
            "SADR",
            "SAIPH",
            "SARGAS",
            "SCHEAT",
            "SCHEDAR",
            "SEGINUS",
            "SGR_A_STAR",
            "SHAULA",
            "SHELIAK",
            "SHERATAN",
            "SIRIUS",
            "SKAT",
            "SPICA",
            "SUHAIL",
            "SYRMA",
            "THUBAN",
            "TUREIS",
            "UNUKALHAI",
            "VEGA",
            "VINDEMIATRIX",
            "WEZEN",
            "YED_PRIOR",
            "ZANIAH",
            "ZOSMA",
            "ZUBENELGENUBI",
            "ZUBENESCHAMALI",
        ],
    ),
    (
        "tibetan_element",
        "the Tibetan calendar's elements and animals; no source for the Lhosar calendar as \
         printed in Nepal has been read",
        &["EARTH", "FIRE", "IRON", "WATER", "WOOD"],
    ),
    (
        "tibetan_animal",
        "the Tibetan calendar's elements and animals; no source for the Lhosar calendar as \
         printed in Nepal has been read",
        &[
            "BIRD", "DOG", "DRAGON", "HORSE", "MONKEY", "MOUSE", "OX", "PIG", "RABBIT", "SHEEP",
            "SNAKE", "TIGER",
        ],
    ),
];

/// Every catalogue kind, by name, from the catalogue itself.
fn catalogue_kinds() -> BTreeSet<String> {
    Kind::ALL
        .iter()
        .map(|kind| kind.name().to_owned())
        .collect()
}

/// The catalogue kinds a chart document can carry, read from its schema's
/// `x-teistro-kind` keywords rather than kept as a second list.
pub(crate) fn document_kinds() -> BTreeSet<String> {
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
            "FAIL  {kind}.{key}: a strict locale does not name it; \
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
            "ok    every catalogue member is named in every strict locale, \
             but {} listed",
            listed.len()
        );
    }
    failures
}
