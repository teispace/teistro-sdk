//! The answer with every catalogue member written as its full key
//! (`03-design/muhurta-at-the-boundary.md` §2.6).
//!
//! Every binding reads a member back as `nakshatra.ROHINI`, and a
//! clause's field name does not settle its kind: `yoga` is a `yoga` in a
//! `YOGA` clause and a `muhurta_yoga` in a `MUHURTA_YOGA` one, `sign` is a
//! `rashi` in four. So the boundary writes the section in full keys from
//! **one** table, [`ClauseKey::members`], rather than each binding
//! respelling thirty variants from a table of its own. The match is
//! exhaustive, so a clause added later does not compile until its members
//! are listed, and the tests below hold the table to serde both ways.
//!
//! The full keys read back: a catalogue enum's reader takes a member's
//! own full key beside its bare one, so the section deserialises into the
//! same [`Answer`] it was written from.

use serde_json::Value;
use teistro_core::catalogue::{Kind, write_in_full};
use teistro_core::error::{Error, Status};

use crate::clause::ClauseKey;
use crate::search::Answer;

impl ClauseKey {
    /// Where a clause of this kind names catalogue members, and of which
    /// kind: a field, or a field of a field (`pada.nakshatra`). A list's
    /// members are each written in full.
    #[must_use]
    pub const fn members(self) -> &'static [(&'static str, Kind)] {
        match self {
            ClauseKey::Tithi => &[("tithi", Kind::Tithi)],
            ClauseKey::Nakshatra => &[("nakshatra", Kind::Nakshatra)],
            ClauseKey::Yoga => &[("yoga", Kind::Yoga)],
            ClauseKey::Karana => &[("karana", Kind::Karana)],
            ClauseKey::Vara => &[("vara", Kind::Vara)],
            ClauseKey::Month => &[("masa", Kind::Masa)],
            ClauseKey::SolarMonth | ClauseKey::Lagna | ClauseKey::LagnaTyajya => {
                &[("sign", Kind::Rashi)]
            }
            ClauseKey::Pada => &[("pada.nakshatra", Kind::Nakshatra)],
            ClauseKey::Kaala => &[("kaala", Kind::Kaala)],
            ClauseKey::Choghadiya => &[("choghadiya", Kind::Choghadiya)],
            ClauseKey::MuhurtaYoga => &[("yoga", Kind::MuhurtaYoga)],
            ClauseKey::Kartari => &[("second", Kind::Graha), ("twelfth", Kind::Graha)],
            ClauseKey::MoonJoined => &[("with", Kind::Graha)],
            ClauseKey::Kunavamsa => &[("navamsa", Kind::Rashi), ("lord", Kind::Graha)],
            ClauseKey::PanchakaRemainder => &[("panchaka", Kind::Panchaka)],
            ClauseKey::SeventhOccupied | ClauseKey::UnwantedPlacement => &[("by", Kind::Graha)],
            ClauseKey::MaleficInLagna
            | ClauseKey::BeneficInLagna
            | ClauseKey::ExaltedInLagna
            | ClauseKey::LuminaryInEleventh
            | ClauseKey::KendraBenefics => &[("grahas", Kind::Graha)],
            ClauseKey::Abhijit
            | ClauseKey::Tarabala
            | ClauseKey::Chandrabala
            | ClauseKey::MoonInDusthana
            | ClauseKey::VenusInSixth
            | ClauseKey::MarsInEighth
            | ClauseKey::AshtamaLagna => &[],
        }
    }
}

/// Where the answer names catalogue members outside its clauses: a
/// baseline factor's graha, a closed day's calendar and era, and the
/// blackouts that closed it.
const OUTSIDE_CLAUSES: [(&str, &str, Kind); 4] = [
    ("windows", "score.factors.graha", Kind::Graha),
    ("closed", "date.calendar", Kind::Calendar),
    ("closed", "date.era.era", Kind::Era),
    ("closed", "by", Kind::BlackoutKind),
];

/// The answer as JSON, every catalogue member written as its full key.
///
/// # Errors
///
/// `INTERNAL` if the answer does not serialise, which a value this crate
/// built cannot do.
pub fn in_full(answer: &Answer) -> Result<Value, Error> {
    let mut value = serde_json::to_value(answer).map_err(|error| {
        Error::new(
            Status::Internal,
            format!("a muhurta answer did not serialise: {error}"),
        )
    })?;
    for window in items(&mut value, "windows") {
        for clause in items(window, "clauses") {
            clause_in_full(clause)?;
        }
        // A bar is a clause's key, which names no member, or a clause.
        for bar in items(window, "barredBy") {
            if bar.is_object() {
                clause_in_full(bar)?;
            }
        }
    }
    for (list, path, kind) in OUTSIDE_CLAUSES {
        for item in items(&mut value, list) {
            write_in_full(item, path, kind);
        }
    }
    Ok(value)
}

/// A clause's members, by its tag.
fn clause_in_full(clause: &mut Value) -> Result<(), Error> {
    let key: ClauseKey = clause
        .get("clause")
        .cloned()
        .and_then(|tag| serde_json::from_value(tag).ok())
        .ok_or_else(|| {
            Error::new(
                Status::Internal,
                format!("a muhurta clause carried no tag a key reads: {clause}"),
            )
        })?;
    for (path, kind) in key.members() {
        write_in_full(clause, path, *kind);
    }
    Ok(())
}

/// The elements of an object's list field, none when it has none.
fn items<'v>(value: &'v mut Value, field: &str) -> impl Iterator<Item = &'v mut Value> {
    value
        .get_mut(field)
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, reason = "tests fail by panicking")]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;
    use teistro_calendar::{CalendarDate, EraNumber};
    use teistro_core::catalogue::{
        Calendar, Choghadiya, Era, Graha, Kaala, Karana, Kind, Masa, MuhurtaYoga, Nakshatra,
        Panchaka, Rashi, Tithi, Vara, Yoga,
    };
    use teistro_core::interval::Interval;
    use teistro_core::quantity::JulianDay;

    use super::in_full;
    use crate::activity::{Bar, Pada, Unjudged};
    use crate::baseline::{Dimension, Factor, Score};
    use crate::clause::{Clause, ClauseKey, ClauseKind};
    use crate::grade::Grade;
    use crate::judge::{Judgement, Ranking};
    use crate::search::{Answer, ClosedDay};
    use crate::season::BlackoutKind;
    use crate::tara::{Tara, TaraReading};

    /// One clause of a kind, every list in it non-empty, so each listed
    /// path has a member to write. Exhaustive, so a new kind is sampled
    /// before it compiles.
    fn sample(key: ClauseKey) -> ClauseKind {
        let two = || vec![Graha::Jupiter, Graha::Venus];
        match key {
            ClauseKey::Tithi => ClauseKind::Tithi {
                tithi: Tithi::ShuklaPratipada,
                grade: Grade::Best,
            },
            ClauseKey::Nakshatra => ClauseKind::Nakshatra {
                nakshatra: Nakshatra::Rohini,
                grade: Grade::Best,
            },
            ClauseKey::Yoga => ClauseKind::Yoga {
                yoga: Yoga::Siddhi,
                grade: Grade::Middling,
            },
            ClauseKey::Karana => ClauseKind::Karana {
                karana: Karana::Bava,
                grade: Grade::Best,
            },
            ClauseKey::Vara => ClauseKind::Vara {
                vara: Vara::Guruvara,
                grade: Grade::Best,
            },
            ClauseKey::Month => ClauseKind::Month {
                masa: Masa::Margashirsha,
                grade: Grade::Best,
            },
            ClauseKey::SolarMonth => ClauseKind::SolarMonth {
                sign: Rashi::Scorpio,
                grade: Grade::Rejected,
            },
            ClauseKey::Lagna => ClauseKind::Lagna {
                sign: Rashi::Taurus,
                grade: Grade::Best,
            },
            ClauseKey::Pada => ClauseKind::Pada {
                pada: Pada {
                    nakshatra: Nakshatra::Mula,
                    pada: 1,
                },
            },
            ClauseKey::Kaala => ClauseKind::Kaala {
                kaala: Kaala::RahuKaala,
            },
            ClauseKey::Choghadiya => ClauseKind::Choghadiya {
                choghadiya: Choghadiya::Amrit,
            },
            ClauseKey::Abhijit => ClauseKind::Abhijit {},
            ClauseKey::MuhurtaYoga => ClauseKind::MuhurtaYoga {
                yoga: MuhurtaYoga::SarvarthaSiddhi,
            },
            ClauseKey::Tarabala => ClauseKind::Tarabala {
                reading: TaraReading {
                    count: 3,
                    tara: Tara::Vipat,
                    cycle: 1,
                },
            },
            ClauseKey::Chandrabala => ClauseKind::Chandrabala {
                house: 8,
                holds: false,
            },
            ClauseKey::Kartari => ClauseKind::Kartari {
                second: vec![Graha::Mars],
                twelfth: vec![Graha::Saturn],
            },
            ClauseKey::MoonInDusthana => ClauseKind::MoonInDusthana { house: 6 },
            ClauseKey::MoonJoined => ClauseKind::MoonJoined { with: two() },
            ClauseKey::VenusInSixth => ClauseKind::VenusInSixth {},
            ClauseKey::MarsInEighth => ClauseKind::MarsInEighth {},
            ClauseKey::AshtamaLagna => ClauseKind::AshtamaLagna {},
            ClauseKey::Kunavamsa => ClauseKind::Kunavamsa {
                navamsa: Rashi::Cancer,
                lord: Graha::Moon,
            },
            ClauseKey::PanchakaRemainder => ClauseKind::PanchakaRemainder {
                panchaka: Panchaka::Mrityu,
            },
            ClauseKey::LagnaTyajya => ClauseKind::LagnaTyajya { sign: Rashi::Leo },
            ClauseKey::SeventhOccupied => ClauseKind::SeventhOccupied { by: two() },
            ClauseKey::MaleficInLagna => ClauseKind::MaleficInLagna { grahas: two() },
            ClauseKey::BeneficInLagna => ClauseKind::BeneficInLagna { grahas: two() },
            ClauseKey::ExaltedInLagna => ClauseKind::ExaltedInLagna { grahas: two() },
            ClauseKey::LuminaryInEleventh => ClauseKind::LuminaryInEleventh { grahas: two() },
            ClauseKey::KendraBenefics => ClauseKind::KendraBenefics { grahas: two() },
            ClauseKey::UnwantedPlacement => ClauseKind::UnwantedPlacement {
                house: 8,
                by: two(),
            },
        }
    }

    fn at() -> Interval {
        Interval {
            from: JulianDay::literal(2_461_374.5),
            to: JulianDay::literal(2_461_374.6),
        }
    }

    /// An answer holding every clause kind, a bar of each shape, a
    /// baseline factor with a graha and one without, and a closed day
    /// with an era.
    fn answer() -> Answer {
        let clauses = ClauseKey::ALL
            .into_iter()
            .map(|key| Clause {
                kind: sample(key),
                at: at(),
            })
            .collect();
        let mut date = CalendarDate::defined(Calendar::Gregorian, 2026, 12, 1);
        date.era = Some(EraNumber {
            era: Era::CommonEra,
            year: 2026,
        });
        Answer {
            windows: vec![Judgement {
                at: at(),
                clauses,
                barred_by: vec![
                    Bar::Key(ClauseKey::Kaala),
                    Bar::Clause(sample(ClauseKey::Kunavamsa)),
                ],
                score: Some(Score {
                    value: 71,
                    factors: vec![
                        Factor {
                            dimension: Dimension::TaraBala,
                            weight: 5,
                            graha: Some(Graha::Moon),
                        },
                        Factor {
                            dimension: Dimension::Panchaka,
                            weight: -3,
                            graha: None,
                        },
                    ],
                    capped_at: None,
                }),
            }],
            closed: vec![ClosedDay {
                date,
                by: vec![BlackoutKind::ShukraAsta],
            }],
            days_judged: 1,
            days_cut: 1,
            windows_blacked_out: 0,
            ranking: Ranking::Texts,
            unjudged: vec![Unjudged {
                what: "YOGA_TEXT".to_owned(),
                why: "a reason".to_owned(),
            }],
        }
    }

    /// Every string leaf with its path, lists written `[]`.
    fn leaves(value: &Value, path: &str, out: &mut Vec<(String, String)>) {
        match value {
            Value::String(text) => out.push((path.to_owned(), text.clone())),
            Value::Array(list) => {
                for item in list {
                    leaves(item, &format!("{path}[]"), out);
                }
            }
            Value::Object(fields) => {
                for (field, inner) in fields {
                    let joined = if path.is_empty() {
                        field.clone()
                    } else {
                        format!("{path}.{field}")
                    };
                    leaves(inner, &joined, out);
                }
            }
            _ => {}
        }
    }

    /// The string leaves that are **not** catalogue members, and why: a
    /// crate-local enum, or text. Refused both ways: a leaf here written
    /// in full, or one listed here the sample never reaches, fails.
    const LOCAL: [(&str, &str); 10] = [
        ("windows[].clauses[].clause", "the tag, a `ClauseKey`"),
        ("windows[].clauses[].grade", "a `Grade`"),
        ("windows[].clauses[].reading.tara", "a `Tara`"),
        ("windows[].barredBy[]", "a bar by key, a `ClauseKey`"),
        ("windows[].barredBy[].clause", "a barring clause's tag"),
        (
            "windows[].score.factors[].dimension",
            "a baseline `Dimension`",
        ),
        (
            "closed[].date.resolution.kind",
            "a `CalendarResolution`'s tag",
        ),
        ("ranking", "a `Ranking`"),
        ("unjudged[].what", "text the rules carry"),
        ("unjudged[].why", "text the rules carry"),
    ];

    #[test]
    fn every_key_is_sampled_as_itself() {
        for key in ClauseKey::ALL {
            assert_eq!(sample(key).key(), key);
        }
    }

    #[test]
    fn every_member_is_written_in_full_and_nothing_else_is() {
        let written = in_full(&answer()).expect("it respells");
        let mut found = Vec::new();
        leaves(&written, "", &mut found);
        let local: BTreeSet<&str> = LOCAL.iter().map(|(path, _)| *path).collect();
        let kinds: BTreeSet<&str> = Kind::ALL.iter().map(|kind| kind.name()).collect();
        let mut reached = BTreeSet::new();
        for (path, text) in &found {
            let prefix = text.split_once('.').map(|(kind, _)| kind);
            if local.contains(path.as_str()) {
                reached.insert(path.as_str());
                assert!(
                    prefix.is_none_or(|kind| !kinds.contains(kind)),
                    "`{path}` is local and was written in full: {text}"
                );
            } else {
                assert!(
                    prefix.is_some_and(|kind| kinds.contains(kind)),
                    "`{path}` holds `{text}`, neither a full key nor a local leaf"
                );
            }
        }
        let never: Vec<&str> = local
            .iter()
            .copied()
            .filter(|path| !reached.contains(path))
            .collect();
        assert_eq!(
            never,
            Vec::<&str>::new(),
            "local leaves the sample never reached"
        );
    }

    #[test]
    fn every_listed_member_exists_and_reads_back() {
        for key in ClauseKey::ALL {
            let clause = serde_json::to_value(sample(key)).expect("a clause serialises");
            for (path, _) in key.members() {
                let mut at = &clause;
                for field in path.split('.') {
                    at = at
                        .get(field)
                        .unwrap_or_else(|| panic!("{key:?} lists `{path}`, which serde omits"));
                }
                assert!(!at.is_null(), "{key:?}'s `{path}` is null in its sample");
            }
        }
        // Both ways: the full keys deserialise into the answer they were
        // written from, so no member was given the wrong kind's prefix.
        let answer = answer();
        let written = in_full(&answer).expect("it respells");
        let back: Answer = serde_json::from_value(written).expect("full keys read back");
        assert_eq!(back, answer);
    }
}
