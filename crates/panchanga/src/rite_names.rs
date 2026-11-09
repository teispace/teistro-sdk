//! That `sdk.rite` names every member of the answers it is written for, in
//! every locale that carries it, by the spelling serde gives the member,
//! and names nothing else (SR-31).
//!
//! The members are listed here, inside the crate, so that a `match` with
//! no wildcard holds each list to its enum: a member added to a
//! `#[non_exhaustive]` enum fails this build until it is listed, and so
//! until its name is written.

#![allow(clippy::expect_used, clippy::panic, reason = "tests fail by panicking")]

use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;

use crate::ashauch::{
    AshauchKind, Clause, KinKeeps, Length, Relation, Rite, RiteBeginning, RiteVerdict,
};
use crate::festival::AnniversaryKind;
use crate::first_year::{Rite as FirstYearRite, RiteForm, SapindanaAt};

/// The locales that carry `sdk.rite`.
const LOCALES: [&str; 2] = ["en-Latn", "ne-Deva-NP"];

/// A member's spelling on the wire: the string itself, or an internally
/// tagged variant's tag.
fn spelling(value: impl Serialize, tag: Option<&str>) -> String {
    let value = serde_json::to_value(value).expect("an answer's member serialises");
    let spelled = match tag {
        Some(tag) => value.get(tag).cloned(),
        None => Some(value),
    };
    spelled
        .and_then(|spelled| spelled.as_str().map(str::to_owned))
        .expect("a member spells as a string")
}

fn spellings<T: Serialize + Copy>(members: &[T]) -> BTreeSet<String> {
    members
        .iter()
        .map(|member| spelling(*member, None))
        .collect()
}

fn ashauch_kinds() -> BTreeSet<String> {
    use AshauchKind::{Birth, Death};
    let listed = |kind: AshauchKind| match kind {
        Birth | Death => kind,
    };
    spellings(&[Birth, Death].map(listed))
}

fn lengths() -> BTreeSet<String> {
    let all = [
        Length::Bath,
        Length::Sajyotih,
        Length::Pakshini,
        Length::Days { days: 1 },
        Length::Full,
    ];
    let listed = |length: Length| match length {
        Length::Bath | Length::Sajyotih | Length::Pakshini | Length::Days { .. } | Length::Full => {
            length
        }
    };
    all.map(listed)
        .into_iter()
        .map(|length| spelling(length, Some("length")))
        .collect()
}

fn relations() -> BTreeSet<String> {
    use Relation::{
        AcharyasFamily, Adoption, Bandhu, Child, CoWife, DaughtersSon, Father, MarriedDaughter,
        MarriedSibling, MaternalAunt, MaternalGrandfather, MaternalGrandmother, MaternalUncle,
        MaternalUnclesWife, Mother, OfAscetic, ParentInLaw, PaternalAunt, Pupil, Sagotra,
        Samanodaka, Sapinda, SistersSon, SonInLaw, Spouse, StepChild, Subject, WifesBrother,
    };
    let listed = |relation: Relation| match relation {
        Mother | Father | Child | Spouse | CoWife | StepChild | Sapinda | Samanodaka | Sagotra
        | MarriedDaughter | MarriedSibling | MaternalGrandfather | MaternalGrandmother
        | MaternalUncle | SonInLaw | ParentInLaw | Pupil | AcharyasFamily | Adoption | Bandhu
        | Subject | OfAscetic | MaternalAunt | PaternalAunt | MaternalUnclesWife | SistersSon
        | DaughtersSon | WifesBrother => relation,
    };
    spellings(
        &[
            Mother,
            Father,
            Child,
            Spouse,
            CoWife,
            StepChild,
            Sapinda,
            Samanodaka,
            Sagotra,
            MarriedDaughter,
            MarriedSibling,
            MaternalGrandfather,
            MaternalGrandmother,
            MaternalUncle,
            SonInLaw,
            ParentInLaw,
            Pupil,
            AcharyasFamily,
            Adoption,
            Bandhu,
            Subject,
            OfAscetic,
            MaternalAunt,
            PaternalAunt,
            MaternalUnclesWife,
            SistersSon,
            DaughtersSon,
            WifesBrother,
        ]
        .map(listed),
    )
}

fn clauses() -> BTreeSet<String> {
    let said = String::new;
    let all = [
        Clause::Relation {
            row: 0,
            said: said(),
        },
        Clause::Band {
            row: 0,
            said: said(),
        },
        Clause::Loss {
            row: 0,
            said: said(),
        },
        Clause::BirthPeriodRuns,
        Clause::Remainder,
        Clause::Late {
            within_days: 0,
            abroad: false,
        },
        Clause::LateBath,
        Clause::RowHeardLate {
            row: 0,
            said: said(),
        },
        Clause::ShortHeardLate,
        Clause::BirthHeardLate,
    ];
    let listed = |clause: Clause| match clause {
        Clause::Relation { .. }
        | Clause::Band { .. }
        | Clause::Loss { .. }
        | Clause::BirthPeriodRuns
        | Clause::Remainder
        | Clause::Late { .. }
        | Clause::LateBath
        | Clause::RowHeardLate { .. }
        | Clause::ShortHeardLate
        | Clause::BirthHeardLate => clause,
    };
    all.map(listed)
        .into_iter()
        .map(|clause| spelling(clause, Some("clause")))
        .collect()
}

fn rites() -> BTreeSet<String> {
    use Rite::{
        Chudakarana, DailyRules, EclipseRites, FoodCharity, KotiHoma, Marriage, ParthivaWorship,
        Penance, Pratishtha, PriestsOffice, Purashcharana, Sacrifice, SankrantiBath, Shanti,
        Shraddha, TankDedication, TulapurushaGift, Upanayana, Vow, Worship,
    };
    let listed = |rite: Rite| match rite {
        Marriage | Upanayana | Chudakarana | Pratishtha | TankDedication | KotiHoma
        | TulapurushaGift | Vow | Purashcharana | Penance | Sacrifice | PriestsOffice | Shanti
        | FoodCharity | Shraddha | EclipseRites | SankrantiBath | ParthivaWorship | DailyRules
        | Worship => rite,
    };
    spellings(
        &[
            Marriage,
            Upanayana,
            Chudakarana,
            Pratishtha,
            TankDedication,
            KotiHoma,
            TulapurushaGift,
            Vow,
            Purashcharana,
            Penance,
            Sacrifice,
            PriestsOffice,
            Shanti,
            FoodCharity,
            Shraddha,
            EclipseRites,
            SankrantiBath,
            ParthivaWorship,
            DailyRules,
            Worship,
        ]
        .map(listed),
    )
}

fn verdicts() -> BTreeSet<String> {
    use RiteVerdict::{Barred, GoesOn, InPart};
    let listed = |verdict: RiteVerdict| match verdict {
        GoesOn | InPart | Barred => verdict,
    };
    spellings(&[GoesOn, InPart, Barred].map(listed))
}

fn beginnings() -> BTreeSet<String> {
    use RiteBeginning::{Cooking, NandiShraddha, Sankalpa, Varana};
    let listed = |beginning: RiteBeginning| match beginning {
        NandiShraddha | Sankalpa | Varana | Cooking => beginning,
    };
    spellings(&[NandiShraddha, Sankalpa, Varana, Cooking].map(listed))
}

fn kin_keeps() -> BTreeSet<String> {
    use KinKeeps::{Bath, Nothing, Period};
    let listed = |keeps: KinKeeps| match keeps {
        Period | Bath | Nothing => keeps,
    };
    spellings(&[Period, Bath, Nothing].map(listed))
}

fn first_year_rites() -> BTreeSet<String> {
    use FirstYearRite::{
        Abdika, AdyaMasika, Masika, Sapindana, Traipakshika, UnaAbdika, UnaMasika, UnaShanmasika,
    };
    let listed = |rite: FirstYearRite| match rite {
        AdyaMasika | UnaMasika | Masika | Traipakshika | UnaShanmasika | UnaAbdika | Abdika
        | Sapindana => rite,
    };
    spellings(
        &[
            AdyaMasika,
            UnaMasika,
            Masika,
            Traipakshika,
            UnaShanmasika,
            UnaAbdika,
            Abdika,
            Sapindana,
        ]
        .map(listed),
    )
}

fn rite_forms() -> BTreeSet<String> {
    use RiteForm::{Ekoddishta, Parvana};
    let listed = |form: RiteForm| match form {
        Ekoddishta | Parvana => form,
    };
    spellings(&[Ekoddishta, Parvana].map(listed))
}

fn sapindana_days() -> BTreeSet<String> {
    use SapindanaAt::{EleventhDay, Traipakshika, TwelfthDay, YearEnd};
    let listed = |at: SapindanaAt| match at {
        EleventhDay | TwelfthDay | Traipakshika | YearEnd => at,
    };
    spellings(&[EleventhDay, TwelfthDay, Traipakshika, YearEnd].map(listed))
}

fn anniversary_kinds() -> BTreeSet<String> {
    use AnniversaryKind::{Ekoddishta, Event, JanmaTithi, Parvana, Shraddha};
    let listed = |kind: AnniversaryKind| match kind {
        Shraddha | Ekoddishta | Parvana | JanmaTithi | Event => kind,
    };
    spellings(&[Shraddha, Ekoddishta, Parvana, JanmaTithi, Event].map(listed))
}

/// Each message of `sdk.rite` that selects on a member, with the members.
fn groups() -> [(&'static str, BTreeSet<String>); 12] {
    [
        ("ashauchKind", ashauch_kinds()),
        ("ashauchLength", lengths()),
        ("relation", relations()),
        ("clause", clauses()),
        ("rite", rites()),
        ("riteVerdict", verdicts()),
        ("riteBeginning", beginnings()),
        ("kinKeeps", kin_keeps()),
        ("firstYearRite", first_year_rites()),
        ("riteForm", rite_forms()),
        ("sapindanaAt", sapindana_days()),
        ("anniversaryKind", anniversary_kinds()),
    ]
}

/// The keys a one-selector `.match` names, its catch-all left out.
fn variant_keys(message: &str) -> BTreeSet<String> {
    message
        .split("}}")
        .filter_map(|piece| piece.split_once("{{"))
        .filter_map(|(head, _)| head.split_whitespace().last())
        .filter(|key| *key != "*")
        .map(str::to_owned)
        .collect()
}

#[test]
fn every_member_an_answer_carries_is_named_and_nothing_else() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
    for locale in LOCALES {
        let path = root.join(locale).join("sdk.rite.json");
        let text = std::fs::read_to_string(&path).expect("the pack is read");
        let messages: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&text).expect("the pack is JSON");
        for (group, members) in groups() {
            let message = messages
                .get(group)
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{locale} has no sdk.rite.{group}"));
            assert_eq!(
                variant_keys(message),
                members,
                "{locale}'s sdk.rite.{group} names its members by their serde spelling"
            );
        }
    }
}

#[test]
fn a_variant_key_is_read_from_its_match() {
    let keys = variant_keys(".input {$k :string} .match $k A {{a}} B_C {{b {$x} c}} * {{{$k}}}");
    assert_eq!(keys, BTreeSet::from(["A".to_owned(), "B_C".to_owned()]));
}
