//! The annual chart's five office-bearers, the **panchadhikaris**, one of
//! whom becomes the lord of the year (`03-design/muntha.md`, "The
//! office-bearers").
//!
//! Four of the five are a sign's lord, and which sign is the whole of the
//! rule; the fifth, the Tri-Rashi lord, is a table the source prints and
//! that turns out to be a rule too (crux C108). What they need between
//! them is two charts — the birth for the Muntha and the natal lagna, the
//! annual chart for the rest — which is why they are asked of an annual
//! chart the **caller** has founded: three of them depend on where it was
//! cast, and the SDK does not decide that for anyone.
//!
//! Choosing among the five is not here. It needs the Panchavargiya bala
//! and the Tajika aspects, neither built yet; what is here is everything
//! the choice is made from, including how many portfolios each planet
//! holds, which is the choice's own tie-break (crux C106).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi, Tatwa};
use teistro_core::error::Error;

use crate::muntha::{MunthaDegree, muntha};

/// The five portfolios, in the order the source lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Office {
    /// The **Munthesha**: the lord of the Muntha's sign.
    Muntha,
    /// The **Janmesha**: the lord of the birth chart's lagna.
    JanmaLagna,
    /// The **Varsha Lagnesha**: the lord of the annual chart's lagna.
    VarshaLagna,
    /// The **Tri-Rashi Pati**: the annual lagna's triplicity lord, by day
    /// or by night.
    TriRashi,
    /// The **Dina-Ratri Pati**: the lord of the Sun's sign when the year
    /// opens by day, of the Moon's when it opens by night.
    DinaRatri,
}

impl Office {
    /// All five, in the source's order.
    pub const ALL: [Office; 5] = [
        Office::Muntha,
        Office::JanmaLagna,
        Office::VarshaLagna,
        Office::TriRashi,
        Office::DinaRatri,
    ];
}

/// What the five are read from: two charts, reduced to the six numbers
/// the rules touch.
///
/// A struct rather than six positional arguments, because four of them
/// are longitudes and a caller swapping two would get an answer.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YearCharts {
    /// The birth chart's lagna, sidereal degrees.
    pub natal_lagna_deg: f64,
    /// The years complete at the return: a [`Pravesha::year`](crate::Pravesha::year).
    pub completed_years: u16,
    /// The annual chart's lagna, sidereal degrees.
    pub annual_lagna_deg: f64,
    /// The Sun in the annual chart, sidereal degrees.
    pub annual_sun_deg: f64,
    /// The Moon in the annual chart, sidereal degrees.
    pub annual_moon_deg: f64,
    /// Whether the return falls between sunrise and sunset at the place
    /// the annual chart was cast for — the source's own definition of
    /// "by day", and a founded chart's `day.part`.
    pub by_day: bool,
}

/// The five office-bearers of one year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OfficeBearers {
    /// The lord of the Muntha's sign.
    pub muntha: Graha,
    /// The lord of the birth lagna.
    pub janma_lagna: Graha,
    /// The lord of the annual lagna.
    pub varsha_lagna: Graha,
    /// The annual lagna's triplicity lord for the part of the day.
    pub tri_rashi: Graha,
    /// The lord of the Sun's sign by day or the Moon's by night.
    pub dina_ratri: Graha,
    /// Whether the year opened by day, which chose the last two.
    pub by_day: bool,
}

impl OfficeBearers {
    /// Who holds one portfolio.
    #[must_use]
    pub const fn holder(&self, office: Office) -> Graha {
        match office {
            Office::Muntha => self.muntha,
            Office::JanmaLagna => self.janma_lagna,
            Office::VarshaLagna => self.varsha_lagna,
            Office::TriRashi => self.tri_rashi,
            Office::DinaRatri => self.dina_ratri,
        }
    }

    /// The portfolios one planet holds, in the source's order; empty when
    /// it holds none.
    ///
    /// One planet may hold several — the source's own worked year gives
    /// the Sun two and Mars two — and the count is the year lord's
    /// tie-break when strength and aspect cannot decide.
    #[must_use]
    pub fn portfolios(&self, graha: Graha) -> Vec<Office> {
        Office::ALL
            .into_iter()
            .filter(|office| self.holder(*office) == graha)
            .collect()
    }

    /// The distinct planets holding a portfolio, each once, in the order
    /// of the first portfolio each holds.
    ///
    /// These are the **claimants** to the year's lordship: between one
    /// and five of them.
    #[must_use]
    pub fn claimants(&self) -> Vec<Graha> {
        let mut out: Vec<Graha> = Vec::with_capacity(5);
        for office in Office::ALL {
            let graha = self.holder(office);
            if !out.contains(&graha) {
                out.push(graha);
            }
        }
        out
    }
}

/// The triplicity lords of one element: by day, by night, and the one
/// that participates in both (Dorotheus, *Carmen Astrologicum* I.1, in
/// the order the Hellenistic sources give them).
struct Triplicity {
    day: Graha,
    night: Graha,
    participating: Graha,
}

const fn triplicity(element: Tatwa) -> Option<Triplicity> {
    let (day, night, participating) = match element {
        Tatwa::Agni => (Graha::Sun, Graha::Jupiter, Graha::Saturn),
        Tatwa::Prithvi => (Graha::Venus, Graha::Moon, Graha::Mars),
        Tatwa::Vayu => (Graha::Saturn, Graha::Mercury, Graha::Jupiter),
        Tatwa::Jala => (Graha::Venus, Graha::Mars, Graha::Moon),
        // Akasha, and whatever the catalogue adds: no sign has either.
        _ => return None,
    };
    Some(Triplicity {
        day,
        night,
        participating,
    })
}

/// The Tri-Rashi lord of a lagna, by day or by night.
///
/// The source prints this as a table of 24 lords (K.S. Charak, *A
/// Textbook of Varshaphala*, Table VII-1). Every cell of it is one rule
/// over the Dorothean triplicities (crux C108): the **first** sign of a
/// triplicity takes its day and night lords in that order, the **second**
/// takes them the other way round, and the **third** takes the
/// participating lord by day and by night alike. The rule ships and the
/// printed table is its test, which is the only way either can fail.
#[must_use]
pub fn tri_rashi_lord(lagna: Rashi, by_day: bool) -> Graha {
    // Every sign has one of the four elements a triplicity has, which a
    // test holds, so this arm is unreachable; it answers the sign's own
    // lord rather than panicking in a consumer's process if the catalogue
    // ever gives a sign a fifth.
    let Some(lords) = triplicity(lagna.attributes().element) else {
        return lagna.attributes().lord;
    };
    let (by_day_lord, by_night_lord) = match lagna.id() / 4 {
        0 => (lords.day, lords.night),
        1 => (lords.night, lords.day),
        _ => (lords.participating, lords.participating),
    };
    if by_day { by_day_lord } else { by_night_lord }
}

/// The five office-bearers of one year.
///
/// # Errors
///
/// A longitude that is not a number, named by its field; a
/// `completed_years` past the cap, named `completed_years`.
///
/// # Examples
///
/// The source's worked year: Leo rising at birth, forty years complete,
/// Scorpio rising in the annual chart by day with the Sun in Leo.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{Office, YearCharts, office_bearers};
///
/// let year = office_bearers(&YearCharts {
///     natal_lagna_deg: 134.6,
///     completed_years: 40,
///     annual_lagna_deg: 219.43,
///     annual_sun_deg: 123.83,
///     annual_moon_deg: 39.67,
///     by_day: true,
/// })?;
/// assert_eq!(year.muntha, Graha::Jupiter);
/// assert_eq!(year.portfolios(Graha::Mars).len(), 2);
/// assert_eq!(year.claimants(), [Graha::Jupiter, Graha::Sun, Graha::Mars]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub fn office_bearers(year: &YearCharts) -> Result<OfficeBearers, Error> {
    let sign = |field: &str, longitude: f64| -> Result<Rashi, Error> {
        if !longitude.is_finite() {
            return Err(
                Error::invalid_arg(format!("{field} is a longitude that is not a number"))
                    .with_field(field.to_owned()),
            );
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a longitude folded into 0..360 divides by thirty into 0..12"
        )]
        let index = (longitude.rem_euclid(360.0) / 30.0) as u16 % 12;
        Rashi::from_id(index).ok_or_else(|| Error::internal("a sign index below twelve is a sign"))
    };
    let natal = sign("natal_lagna_deg", year.natal_lagna_deg)?;
    let annual = sign("annual_lagna_deg", year.annual_lagna_deg)?;
    let sun = sign("annual_sun_deg", year.annual_sun_deg)?;
    let moon = sign("annual_moon_deg", year.annual_moon_deg)?;
    // The degree reading moves the Muntha's longitude and never its sign,
    // so the default is as good as either for its lord.
    let found = muntha(
        year.natal_lagna_deg,
        year.completed_years,
        MunthaDegree::default(),
    )?;
    Ok(OfficeBearers {
        muntha: found.lord,
        janma_lagna: natal.attributes().lord,
        varsha_lagna: annual.attributes().lord,
        tri_rashi: tri_rashi_lord(annual, year.by_day),
        dina_ratri: if year.by_day { sun } else { moon }.attributes().lord,
        by_day: year.by_day,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{Office, YearCharts, office_bearers, tri_rashi_lord};
    use teistro_core::catalogue::{Graha, Rashi};

    /// The source's Table VII-1, as printed: each lagna's Tri-Rashi lord by
    /// day and by night, Aries to Pisces.
    const PRINTED: [(Graha, Graha); 12] = [
        (Graha::Sun, Graha::Jupiter),
        (Graha::Venus, Graha::Moon),
        (Graha::Saturn, Graha::Mercury),
        (Graha::Venus, Graha::Mars),
        (Graha::Jupiter, Graha::Sun),
        (Graha::Moon, Graha::Venus),
        (Graha::Mercury, Graha::Saturn),
        (Graha::Mars, Graha::Venus),
        (Graha::Saturn, Graha::Saturn),
        (Graha::Mars, Graha::Mars),
        (Graha::Jupiter, Graha::Jupiter),
        (Graha::Moon, Graha::Moon),
    ];

    /// The rule reproduces every printed cell, and so every printed cell
    /// checks the rule: a misprint and a wrong rule both land here.
    #[test]
    fn the_rule_reproduces_the_printed_tri_rashi_table() {
        let mut cells = 0;
        for (sign, (day, night)) in Rashi::ALL.into_iter().zip(PRINTED) {
            assert_eq!(tri_rashi_lord(sign, true), day, "{sign:?} by day");
            assert_eq!(tri_rashi_lord(sign, false), night, "{sign:?} by night");
            cells += 2;
        }
        assert_eq!(cells, 24);
    }

    /// The fallback in `tri_rashi_lord` is dead because of this, and the
    /// test is what keeps it dead: every sign carries an element with a
    /// triplicity, three signs to each.
    #[test]
    fn every_sign_has_a_triplicity_and_each_has_three_signs() {
        use super::triplicity;
        let mut per_element = std::collections::BTreeMap::new();
        for sign in Rashi::ALL {
            let element = sign.attributes().element;
            assert!(triplicity(element).is_some(), "{sign:?}");
            *per_element.entry(element.id()).or_insert(0) += 1;
        }
        assert_eq!(
            per_element.values().copied().collect::<Vec<_>>(),
            [3, 3, 3, 3]
        );
    }

    fn worked() -> YearCharts {
        // K.S. Charak, *A Textbook of Varshaphala*, Chart III-1 and its
        // annual chart: Leo 14°36′ at birth; the forty-first year's chart
        // at Bombay, Scorpio 9°26′ rising at 13:17:29 IST, the Sun in Leo
        // 3°50′ and the Moon in Taurus 9°40′.
        YearCharts {
            natal_lagna_deg: 4.0 * 30.0 + 14.6,
            completed_years: 40,
            annual_lagna_deg: 7.0 * 30.0 + 9.0 + 26.0 / 60.0,
            annual_sun_deg: 4.0 * 30.0 + 3.0 + 50.0 / 60.0,
            annual_moon_deg: 30.0 + 9.0 + 40.0 / 60.0,
            by_day: true,
        }
    }

    /// The source's own five for its worked year, every one of them.
    #[test]
    fn the_sources_worked_year_is_reproduced() {
        let year = office_bearers(&worked()).unwrap();
        assert_eq!(
            Office::ALL.map(|office| year.holder(office)),
            [
                Graha::Jupiter,
                Graha::Sun,
                Graha::Mars,
                Graha::Mars,
                Graha::Sun
            ],
        );
        assert_eq!(
            year.portfolios(Graha::Sun),
            [Office::JanmaLagna, Office::DinaRatri]
        );
        assert_eq!(
            year.portfolios(Graha::Mars),
            [Office::VarshaLagna, Office::TriRashi]
        );
        assert!(year.portfolios(Graha::Venus).is_empty());
        assert_eq!(year.claimants(), [Graha::Jupiter, Graha::Sun, Graha::Mars]);
    }

    /// By night the last two follow the Moon and the night column, and
    /// the first three do not move.
    #[test]
    fn night_moves_only_the_two_that_depend_on_it() {
        let day = office_bearers(&worked()).unwrap();
        let night = office_bearers(&YearCharts {
            by_day: false,
            ..worked()
        })
        .unwrap();
        assert_eq!(
            (night.muntha, night.janma_lagna, night.varsha_lagna),
            (day.muntha, day.janma_lagna, day.varsha_lagna)
        );
        // Scorpio by night is Venus; the Moon in Taurus is Venus's too.
        assert_eq!(
            (night.tri_rashi, night.dina_ratri),
            (Graha::Venus, Graha::Venus)
        );
        assert!(!night.by_day);
    }

    /// Every claimant holds at least one portfolio, the portfolios add up
    /// to five, and no planet is a claimant twice.
    #[test]
    fn portfolios_partition_the_five() {
        for by_day in [true, false] {
            for annual in 0..12_u16 {
                let year = office_bearers(&YearCharts {
                    annual_lagna_deg: f64::from(annual) * 30.0 + 1.0,
                    by_day,
                    ..worked()
                })
                .unwrap();
                let claimants = year.claimants();
                let held: usize = claimants
                    .iter()
                    .map(|graha| year.portfolios(*graha).len())
                    .sum();
                assert_eq!(held, 5);
                assert!(
                    claimants
                        .iter()
                        .all(|graha| !year.portfolios(*graha).is_empty())
                );
                assert!((1..=5).contains(&claimants.len()));
            }
        }
    }

    #[test]
    fn a_longitude_that_is_not_a_number_is_refused_by_its_field() {
        let why = office_bearers(&YearCharts {
            annual_moon_deg: f64::NAN,
            ..worked()
        })
        .expect_err("refused");
        assert_eq!(why.field(), Some("annual_moon_deg"));
    }
}
