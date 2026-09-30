//! The clauses of a day: what its panchanga and the native's birth say
//! about it, each over the interval it held.
//!
//! Every clause here reads the day and not an instant in it: the limbs
//! the almanac already cut into spans, the vara, the special yogas, and
//! the Moon counted from the native. What reads the lagna belongs to an
//! instant.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Karana, Nakshatra, Rashi, Tithi, Vara, Yoga};
use teistro_panchanga::{Panchanga, Span};

use crate::clause::{Clause, ClauseKind};
use crate::grade::{Grade, Graded};
use crate::tara::{ChandraBala, chandra_house, tara};

/// The native a day is read against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Native {
    /// The birth nakshatra: the Moon's at birth, or the name's when the
    /// birth is not known (Raman, ch. II).
    pub star: Nakshatra,
    /// The birth Moon sign.
    pub moon_sign: Rashi,
    /// The birth lagna, when the birth time is known: the ashtama lagna
    /// counts from it.
    pub lagna: Option<Rashi>,
}

/// How a day's rules grade its limbs and vara, and whose Chandrabala
/// table they read.
///
/// Data rather than code, so a caller's own tradition is a value and not
/// a fork: [`DayRules::raman`] is one such value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DayRules {
    /// The tithis.
    pub tithis: Graded<Tithi>,
    /// The nakshatras.
    pub nakshatras: Graded<Nakshatra>,
    /// The yogas.
    pub yogas: Graded<Yoga>,
    /// The karanas.
    pub karanas: Graded<Karana>,
    /// The varas.
    pub varas: Graded<Vara>,
    /// The Chandrabala table.
    pub chandrabala: ChandraBala,
}

impl DayRules {
    /// Raman's panchanga shuddhi, the first of the twenty-one Mahadoshas
    /// (*Muhurtha*, ch. V): the 4th, 6th, 8th, 12th and 14th of either
    /// paksha and the full and new Moon; Bharani and Krittika; Atiganda,
    /// Shula, Ganda, Vyatipata and Vaidhriti; Vishti (Bhadra); and
    /// Tuesday and Saturday (ch. II), with his Chandrabala.
    ///
    /// Two of his numbers are not used as printed. The chapter names
    /// "the 16th (Atiganda)" among the yogas, which is the 6th in his own
    /// list on p. 13, so the yogas are taken by name; and chapter II
    /// lists the 4th, 8th, 12th and 14th tithis while chapter V adds the
    /// 6th, full and new Moon, so the fuller list is taken.
    #[must_use]
    pub fn raman() -> DayRules {
        let per_paksha = [4_u16, 6, 8, 12, 14];
        let tithis = per_paksha
            .iter()
            .flat_map(|n| [n - 1, n + 14])
            .chain([14, 29])
            .filter_map(Tithi::from_id)
            .collect();
        DayRules {
            tithis: Graded::rejecting(tithis),
            nakshatras: Graded::rejecting(vec![Nakshatra::Bharani, Nakshatra::Krittika]),
            yogas: Graded::rejecting(vec![
                Yoga::Atiganda,
                Yoga::Shoola,
                Yoga::Ganda,
                Yoga::Vyatipata,
                Yoga::Vaidhriti,
            ]),
            karanas: Graded::rejecting(vec![Karana::Vishti]),
            varas: Graded::rejecting(vec![Vara::Mangalavara, Vara::Shanivara]),
            chandrabala: ChandraBala::raman(),
        }
    }
}

/// Every clause of a day, in the order the kinds are listed on
/// [`ClauseKind`] and, within a kind, the order they held.
///
/// Without a native, Tarabala and Chandrabala are not read: they count
/// from a birth and there is none to count from.
#[must_use]
pub fn clauses(day: &Panchanga, native: Option<&Native>, rules: &DayRules) -> Vec<Clause> {
    let mut found = Vec::new();
    graded(
        &mut found,
        &day.limbs.tithi,
        &rules.tithis,
        |tithi, grade| ClauseKind::Tithi { tithi, grade },
    );
    graded(
        &mut found,
        &day.limbs.nakshatra,
        &rules.nakshatras,
        |nakshatra, grade| ClauseKind::Nakshatra { nakshatra, grade },
    );
    graded(&mut found, &day.limbs.yoga, &rules.yogas, |yoga, grade| {
        ClauseKind::Yoga { yoga, grade }
    });
    graded(
        &mut found,
        &day.limbs.karana,
        &rules.karanas,
        |karana, grade| ClauseKind::Karana { karana, grade },
    );
    let vara = day.day.vara;
    if let Some(grade) = reported(rules.varas.grade(&vara)) {
        found.push(Clause {
            kind: ClauseKind::Vara { vara, grade },
            at: day.window,
        });
    }
    found.extend(day.kaalas.iter().map(|k| Clause {
        kind: ClauseKind::Kaala { kaala: k.kaala },
        at: k.at,
    }));
    found.extend(day.choghadiya.iter().map(|part| Clause {
        kind: ClauseKind::Choghadiya {
            choghadiya: part.choghadiya,
        },
        at: part.at,
    }));
    if day.muhurtas.abhijit_effective {
        found.extend(day.muhurtas.abhijit.map(|at| Clause {
            kind: ClauseKind::Abhijit {},
            at,
        }));
    }
    found.extend(day.omens.yogas.iter().map(|yoga| Clause {
        kind: ClauseKind::MuhurtaYoga { yoga: yoga.yoga },
        at: yoga.at,
    }));
    if let Some(native) = native {
        found.extend(day.limbs.nakshatra.iter().map(|span| Clause {
            kind: ClauseKind::Tarabala {
                reading: tara(native.star, span.member),
            },
            at: span.inside,
        }));
        found.extend(day.moon.signs.iter().map(|span| {
            let house = chandra_house(native.moon_sign, span.member);
            Clause {
                kind: ClauseKind::Chandrabala {
                    house,
                    holds: rules.chandrabala.holds(house),
                },
                at: span.inside,
            }
        }));
    }
    found
}

/// A clause for each span whose member the rules grade best or reject.
fn graded<T: Copy + PartialEq>(
    found: &mut Vec<Clause>,
    spans: &[Span<T>],
    rule: &Graded<T>,
    kind: impl Fn(T, Grade) -> ClauseKind,
) {
    found.extend(spans.iter().filter_map(|span| {
        reported(rule.grade(&span.member)).map(|grade| Clause {
            kind: kind(span.member, grade),
            at: span.inside,
        })
    }));
}

/// The grades a clause is reported for: the best and the rejected. A
/// middling member neither helps nor harms, so nothing says it.
pub(crate) const fn reported(grade: Grade) -> Option<Grade> {
    match grade {
        Grade::Middling => None,
        other => Some(other),
    }
}

#[cfg(test)]
mod tests {
    use super::DayRules;
    use teistro_core::catalogue::{Tithi, Vara};

    #[test]
    fn ramans_tithis_are_twelve_of_thirty() {
        let rules = DayRules::raman();
        let numbers: Vec<u16> = rules.tithis.rejected.iter().map(|t| t.id() + 1).collect();
        let mut sorted = numbers.clone();
        sorted.sort_unstable();
        // 4, 6, 8, 12, 14 of each paksha, the 15th (Purnima) and the 30th.
        assert_eq!(sorted, [4, 6, 8, 12, 14, 15, 19, 21, 23, 27, 29, 30]);
        assert!(rules.tithis.rejected.contains(&Tithi::Amavasya));
        assert!(rules.varas.rejected.contains(&Vara::Mangalavara));
    }
}
