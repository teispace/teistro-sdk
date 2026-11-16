//! Where a festival falls in a year whose months are not ordinary: an
//! adhika month doubling a name, or a kshaya lunation joining two
//! (`03-design/festival-malamasa.md`, SR-93).
//!
//! The texts agree on one principle and read every case from it.
//! *Dharmasindhu* p. 3 (the 1888 edition, page image): a rite with no
//! other time is kept even in an adhika or a kshaya month, and one with
//! another time is not ("अनन्यगतिकं नित्यनैमित्तिककाम्यं चाधिकमासक्षयमासयोः
//! कर्तव्यम् । सगतिकं नित्यं नैमित्तिकं काम्यं च वर्ज्यम्"). So:
//!
//! - **A doubled month.** A festival of the month has another time, the
//!   nija month, and is kept there (p. 4 lists the festivals,
//!   "शयनपरिवर्तनाद्युत्सवः", among what an adhika month bars;
//!   *Nirnayasindhu*, Dhakal's edition p. 28, quotes the *Kaladarsha*:
//!   "पूर्वत्र प्रतिषिद्धानि परत्रान्यच्च दैविकम्", the rites barred in the
//!   first month and the gods' rites kept in the second). A rule names its
//!   own exceptions with [`super::InAdhika`]: Dashahara (p. 38) and the
//!   year's beginning (p. 31) are kept in the adhika month.
//! - **A kshaya lunation.** It holds two sankrantis and is both months
//!   (p. 3: "मार्गशीर्षपौषाख्यमासद्वयात्मक एको मासो ज्ञेयः"), so a festival of
//!   either has no other time and is kept in it on its tithi. p. 4 begins
//!   the Magha bathing on the full moon of the kshaya month that holds the
//!   Sun's entry into Makara, Pausha's full moon, and keeps the annual
//!   shraddhas of both months in it "without dividing the tithi"
//!   (तिथिपूर्वार्धादिविभागं विनैव). The halves of a tithi (Hemadri, in
//!   *Nirnayasindhu* p. 28) name the month of a death or a birth inside the
//!   joined lunation ([`super::Half`]), not a festival's.

use serde::{Deserialize, Serialize};
use teistro_calendar::lunisolar::MonthKind;

use super::{FestivalDay, FestivalRule, InAdhika};
use crate::month::{self, LunarMonth};
use teistro_core::catalogue::Masa;

/// Which of the month rules placed an observance (SR-93): the reason a
/// day differs from one a year without an adhika or a kshaya month would
/// give. Left off an observance in an ordinary month.
///
/// ```
/// use teistro_panchanga::festival::MalamasaClause;
///
/// assert_eq!(serde_json::to_string(&MalamasaClause::KshayaSecond)?, r#""KSHAYA_SECOND""#);
/// assert!(MalamasaClause::Nija.source().contains("Dharmasindhu"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MalamasaClause {
    /// The year doubles the rule's month and keeps it in the nija month,
    /// after the adhika one, as every festival with another time is.
    Nija,
    /// Kept in the adhika month, by the rule's own `inAdhika` (`FIRST`,
    /// `ONLY` or `ALSO`), which its source cites.
    Adhika,
    /// In a kshaya lunation, under the first of the two names it joins.
    KshayaFirst,
    /// In a kshaya lunation, under the second of its two names: the
    /// month the lunation's own name passes over, whose festivals are kept
    /// in it on their tithis.
    KshayaSecond,
}

impl MalamasaClause {
    /// The four, in the order they are written.
    ///
    /// ```
    /// use teistro_panchanga::festival::MalamasaClause;
    ///
    /// assert_eq!(MalamasaClause::ALL.map(MalamasaClause::key), ["NIJA", "ADHIKA", "KSHAYA_FIRST", "KSHAYA_SECOND"]);
    /// ```
    pub const ALL: [MalamasaClause; 4] = [
        MalamasaClause::Nija,
        MalamasaClause::Adhika,
        MalamasaClause::KshayaFirst,
        MalamasaClause::KshayaSecond,
    ];

    /// The clause's key, `"KSHAYA_SECOND"`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MalamasaClause::Nija => "NIJA",
            MalamasaClause::Adhika => "ADHIKA",
            MalamasaClause::KshayaFirst => "KSHAYA_FIRST",
            MalamasaClause::KshayaSecond => "KSHAYA_SECOND",
        }
    }

    /// The texts the clause is read from, as an answer carries them.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            MalamasaClause::Nija => {
                "Dharmasindhu pp. 3-4: a rite with another time is not kept in an adhika month, and the festivals are among what it bars; Nirnayasindhu p. 28 (Kaladarsha): the gods' rites in the second, nija, month"
            }
            MalamasaClause::Adhika => {
                "the rule's own inAdhika, which its source cites: Dharmasindhu p. 38 keeps Dashahara in a doubled Jyeshtha's adhika month, p. 31 the year's beginning in a doubled Chaitra's"
            }
            MalamasaClause::KshayaFirst => {
                "Dharmasindhu p. 3: a kshaya lunation holds two sankrantis and is both months, kept under its first name"
            }
            MalamasaClause::KshayaSecond => {
                "Dharmasindhu pp. 3-4: a kshaya lunation is both months, so the second month's festivals have no other time and are kept in it on their tithis, the Magha bathing begun on its full moon; Nirnayasindhu p. 28: what an adhika month bars a kshaya month bars, and nothing more"
            }
        }
    }
}

/// The month rule an observance was placed by, and its source (SR-93).
///
/// ```
/// use teistro_panchanga::festival::{Malamasa, MalamasaClause};
///
/// let placed = Malamasa::of(MalamasaClause::KshayaSecond);
/// assert_eq!(placed.clause, MalamasaClause::KshayaSecond);
/// assert!(placed.source.contains("Magha bathing"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Malamasa {
    /// Which clause.
    pub clause: MalamasaClause,
    /// Where the texts say it.
    pub source: String,
}

impl Malamasa {
    /// The clause with its source.
    #[must_use]
    pub fn of(clause: MalamasaClause) -> Malamasa {
        Malamasa {
            clause,
            source: clause.source().to_owned(),
        }
    }
}

/// The month an occurrence is kept in, with the clause that placed it.
pub(super) type Placed = (LunarMonth, Option<MalamasaClause>);

/// The month a rule keeps an occurrence in, read at `at`, a sunrise inside
/// it; `None` when the rule does not keep it, and an error when whether it
/// does turns on a month the days do not reach.
pub(super) fn place(
    rule: &FestivalRule,
    member: teistro_core::catalogue::Tithi,
    days: &[FestivalDay],
    at: usize,
) -> Option<Result<Placed, String>> {
    let day = days.get(at)?;
    let month = month::of(day.month.amanta, member, rule.convention, day.month.kind);
    let name = month.under(rule.convention);
    let kshaya = month.kind == MonthKind::Kshaya;
    // A kshaya lunation answers to both names (p. 3), the second carried
    // as the month the observance is kept in.
    let (month, name, joined) = match rule.month {
        Some(wanted) if wanted == name => {
            (month, name, kshaya.then_some(MalamasaClause::KshayaFirst))
        }
        Some(wanted) if kshaya && wanted == month::next(name) => {
            (second(month), wanted, Some(MalamasaClause::KshayaSecond))
        }
        Some(_) => return None,
        None => (month, name, None),
    };
    let adhika = month.kind == MonthKind::Adhika;
    // Whether the year doubles the name: never the second name of a
    // kshaya lunation, whose adhika month would have taken the first.
    let is_doubled = || match joined {
        Some(MalamasaClause::KshayaSecond) => Some(false),
        _ => doubled(days, at, name),
    };
    // The clause a nija day takes: a doubling's over the first name's. A
    // rule kept every month names no month, so nothing placed it.
    let nija = |doubled: bool| {
        if doubled {
            Some(MalamasaClause::Nija)
        } else {
            joined
        }
    };
    match rule.in_adhika {
        InAdhika::Also if adhika => Some(Ok((month, Some(MalamasaClause::Adhika)))),
        InAdhika::Nija if adhika => None,
        InAdhika::Also | InAdhika::Nija if rule.month.is_none() => Some(Ok((month, None))),
        InAdhika::Also | InAdhika::Nija => Some(Ok((month, nija(is_doubled().unwrap_or(false))))),
        InAdhika::Only | InAdhika::First if adhika => {
            Some(Ok((month, Some(MalamasaClause::Adhika))))
        }
        InAdhika::Only => None,
        InAdhika::First => match is_doubled() {
            Some(true) => None,
            Some(false) => Some(Ok((month, joined))),
            None => Some(Err(format!(
                "the days do not reach the month next to this {}, which says whether it is doubled",
                name.key()
            ))),
        },
    }
}

/// The joined lunation under its second name: every reading a month on.
fn second(month: LunarMonth) -> LunarMonth {
    LunarMonth {
        month: month::next(month.month),
        amanta: month::next(month.amanta),
        purnimanta: month::next(month.purnimanta),
        ..month
    }
}

/// Whether the year doubles the month a nija day's occurrence is named,
/// read from the lunation beside the day's own: an adhika month is new
/// moon to new moon under both conventions, so it comes just before the
/// amanta nija month of its name, and so just after the dark fortnight a
/// purnimanta almanac names after it. `None` when the days stop first.
fn doubled(days: &[FestivalDay], at: usize, name: Masa) -> Option<bool> {
    let own = &days.get(at)?.month;
    let other = |day: &&FestivalDay| day.month.amanta != own.amanta || day.month.kind != own.kind;
    let beside = if own.amanta == name {
        days.get(..at)?.iter().rev().find(other)
    } else {
        days.get(at..)?.iter().find(other)
    }?;
    Some(beside.month.amanta == name && beside.month.kind == MonthKind::Adhika)
}
