//! The **Varshesha**, the lord of the year: which of the five
//! office-bearers holds it (`03-design/varshesha.md`, crux C106).
//!
//! The rule is a chain, not a maximum, and every step of it is the
//! source's:
//!
//! 1. the office-bearers **under five units** are too weak to hold the
//!    year at all, and if every one of them is, the Muntha's lord takes it;
//! 2. otherwise only those that **aspect the annual lagna** may hold it —
//!    an aspect being the Tajika one, which the neutral houses 2, 6, 8 and
//!    12 do not give, and a friendly aspect counting no differently from
//!    an inimical one, as the source says outright;
//! 3. of those, the **strongest** by Panchavargiya bala;
//! 4. on a tie of strength, the one holding the **most portfolios**;
//! 5. on a tie of that too, the **Muntha's lord** — or the Dina-Ratri
//!    Pati, which others give;
//! 6. and if **none** aspects the lagna, the Muntha's lord — or the annual
//!    lagna's lord, which some authorities give instead and which is a
//!    named reading here rather than a silence.
//!
//! And the **Moon** is passed over: "mild in nature and therefore unable to
//! govern unless extraordinarily strong", it is not made lord of the year
//! even when it is the strongest office-bearer aspecting the lagna, and the
//! next one down takes it instead. Where there is nobody to step down to,
//! the planet in **Ithasala** with the Moon takes it, the strongest of
//! several, and failing any the **lord of the Moon's sign** — Charak's rule
//! for that case, and the *Tajika Nilakanthi*'s for every case, which is
//! [`MoonMayRule::Ithasala`].
//!
//! Every answer says **which step decided it**, because a year lord chosen
//! by the fallback is a different statement about the year from one chosen
//! on strength, and a reader cannot tell them apart from the planet alone.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::bala::{AnnualSky, Bala, Panchavargiya, SEVEN, sign_of_longitude};
use crate::drishti::{Drishti, DrishtiRules, between_with_rules};
use crate::office::{Office, OfficeBearers};

/// The strength below which an office-bearer cannot hold the year.
///
/// The source's own figure: "when the strength of the office-bearers is
/// less than 5 units in the Panchavargiya chart, the Muntha lord is to be
/// taken as the Varshesha".
pub const WEAK_BELOW: Bala = Bala::new(5, 0, 0);

/// What to do when no office-bearer aspects the annual lagna (crux C106).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NoneAspects {
    /// The Muntha's lord takes the year: the source's own rule.
    #[default]
    MunthaLord,
    /// The annual lagna's lord takes it, which "some authorities" give.
    AnnualLagnaLord,
    /// The strongest of the five takes it, aspect or none: the *Tajika
    /// Nilakanthi*'s Varshatantra v. 11.
    Strongest,
}

/// Who takes the year when the office-bearers tie outright (crux C106).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Tied {
    /// The Muntha's lord: the source's own rule.
    #[default]
    MunthaLord,
    /// The Dina-Ratri Pati, which "still others" give.
    DinaRatriPati,
}

/// Whether the Moon may hold the year, and who takes it when it may not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonMayRule {
    /// It is passed over and the next one down that aspects takes the
    /// year; where nobody is next, its **Ithasala successor** does.
    /// Charak's two steps, and the default.
    #[default]
    PassedOver,
    /// Its Ithasala successor takes the year at once, with no step down:
    /// the *Tajika Nilakanthi*'s own view (Varshatantra v. 12), which
    /// Charak gives as "some authorities".
    Ithasala,
    /// It holds the year like any other office-bearer, for the
    /// "extraordinarily strong and well-positioned" Moon the source
    /// allows and leaves to the reader.
    LikeAnyOther,
}

/// Which planets may succeed the Moon through an Ithasala.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonPartner {
    /// Any of the seven, the default: Charak's "the planet with which
    /// this Moon establishes an Ithasala", and the *Nilakanthi*'s verse.
    #[default]
    AnyPlanet,
    /// Only an office-bearer, as one of the *Nilakanthi*'s two
    /// commentaries reads its verse.
    OfficeBearer,
}

/// The readings the year lord's chain parts on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct VarsheshaRules {
    /// Who takes the year when nobody aspects the lagna.
    pub none_aspects: NoneAspects,
    /// Who takes it when the office-bearers tie outright.
    pub tied: Tied,
    /// Whether the Moon may hold it.
    pub moon: MoonMayRule,
    /// Who may succeed the Moon through an Ithasala.
    pub moon_partner: MoonPartner,
    /// How the Ithasala the Moon's successor needs is read: the same
    /// rules the yogas take.
    pub drishti: DrishtiRules,
}

/// Which step of the chain decided the year's lord.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Chosen {
    /// The strongest office-bearer that aspects the annual lagna: the
    /// ordinary answer.
    Strongest,
    /// Two or more tied on strength, and this one holds more portfolios.
    MostPortfolios,
    /// The Muntha's lord, because no office-bearer aspects the lagna.
    MunthaLordUnaspected,
    /// The Muntha's lord, because every office-bearer is under five units.
    MunthaLordAllWeak,
    /// The Muntha's lord, because the office-bearers tie on strength,
    /// aspect and portfolios alike.
    MunthaLordTied,
    /// The Dina-Ratri Pati, on that same outright tie, under the other
    /// reading of it.
    DinaRatriTied,
    /// The annual lagna's lord, because nobody aspects the lagna and the
    /// rules ask for that reading.
    AnnualLagnaLordUnaspected,
    /// The strongest of the five, because nobody aspects the lagna and the
    /// rules ask for the *Nilakanthi*'s reading.
    StrongestUnaspected,
    /// The planet in Ithasala with the Moon, the strongest of several, in
    /// the Moon's place.
    MoonsIthasala,
    /// The lord of the Moon's sign, in the Moon's place, the Moon being in
    /// Ithasala with nothing. The Moon itself where it stands in Cancer,
    /// as the *Nilakanthi*'s commentary says.
    MoonsSignLord,
}

impl Chosen {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `STRONGEST`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Chosen::Strongest => "STRONGEST",
            Chosen::MostPortfolios => "MOST_PORTFOLIOS",
            Chosen::MunthaLordUnaspected => "MUNTHA_LORD_UNASPECTED",
            Chosen::MunthaLordAllWeak => "MUNTHA_LORD_ALL_WEAK",
            Chosen::MunthaLordTied => "MUNTHA_LORD_TIED",
            Chosen::DinaRatriTied => "DINA_RATRI_TIED",
            Chosen::AnnualLagnaLordUnaspected => "ANNUAL_LAGNA_LORD_UNASPECTED",
            Chosen::StrongestUnaspected => "STRONGEST_UNASPECTED",
            Chosen::MoonsIthasala => "MOONS_ITHASALA",
            Chosen::MoonsSignLord => "MOONS_SIGN_LORD",
        }
    }

    /// Every step, in the chain's order.
    pub const ALL: [Chosen; 10] = [
        Chosen::Strongest,
        Chosen::MostPortfolios,
        Chosen::MunthaLordUnaspected,
        Chosen::MunthaLordAllWeak,
        Chosen::MunthaLordTied,
        Chosen::DinaRatriTied,
        Chosen::AnnualLagnaLordUnaspected,
        Chosen::StrongestUnaspected,
        Chosen::MoonsIthasala,
        Chosen::MoonsSignLord,
    ];

    /// Whether the year lord was chosen on its own strength rather than by
    /// a fallback.
    #[must_use]
    pub const fn on_strength(self) -> bool {
        matches!(self, Chosen::Strongest | Chosen::MostPortfolios)
    }

    /// Whether it was chosen in the Moon's place.
    #[must_use]
    pub const fn succeeds_the_moon(self) -> bool {
        matches!(self, Chosen::MoonsIthasala | Chosen::MoonsSignLord)
    }
}

/// One office-bearer's claim on the year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Claim {
    /// Whose claim it is.
    pub graha: Graha,
    /// Its five-fold strength.
    pub vishwa: Bala,
    /// Whether it aspects the annual lagna, which it must to hold the year.
    pub aspects_lagna: bool,
    /// How many of the five portfolios it holds: the tie-break.
    pub portfolios: u8,
}

/// The lord of the year, and what made it so.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Varshesha {
    /// The lord of the year.
    pub graha: Graha,
    /// Which step of the chain decided it.
    pub chosen: Chosen,
    /// Its five-fold strength.
    pub vishwa: Bala,
    /// Every claimant, strongest first, so a reader can see the decision
    /// rather than take it on trust.
    pub claims: Vec<Claim>,
    /// Whether the Moon was passed over on the way here: the chain would
    /// have given it the year, and it is "unable to govern". True even
    /// where the successor proves to be the Moon itself, in Cancer.
    pub moon_passed_over: bool,
}

/// Whether a planet in `graha_sign` gives the Tajika aspect to
/// `lagna_sign`.
///
/// The Tajika aspects are the houses 3, 5, 9 and 11 from a planet
/// (friendly) and the kendras 1, 4, 7 and 10 (inimical); the houses 2, 6,
/// 8 and 12 give **no aspect at all**. The sets are symmetric, so it does
/// not matter which of the two is counted from. The year lord's rule makes
/// no distinction between a friendly aspect and an inimical one, which the
/// source states in the same breath as the rule.
#[must_use]
pub fn aspects(graha_sign: Rashi, lagna_sign: Rashi) -> bool {
    // One definition of the aspect, in `drishti`, which the yogas read
    // too; this rule only asks whether there is one at all.
    Drishti::between_signs(graha_sign, lagna_sign).is_aspect()
}

/// The lord of the year.
///
/// `sky` and `strengths` describe the same annual chart: the strengths
/// rank the claimants, and the sky gives the Ithasala a Moon's successor
/// is read from.
///
/// # Errors
///
/// An annual lagna that is not a number, named `annual_lagna_deg`; a
/// strength table that does not carry an office-bearer or a planet the
/// Moon is in Ithasala with, named `strengths`; a sky that places a planet
/// at no longitude.
pub fn varshesha(
    bearers: &OfficeBearers,
    sky: &AnnualSky,
    strengths: &[Panchavargiya],
    annual_lagna_deg: f64,
    rules: VarsheshaRules,
) -> Result<Varshesha, Error> {
    if !annual_lagna_deg.is_finite() {
        return Err(
            Error::invalid_arg("the annual lagna is a longitude that is not a number")
                .with_field("annual_lagna_deg"),
        );
    }
    let lagna = sign_of_longitude(annual_lagna_deg);
    let table = Strengths(strengths);
    let mut claims = Vec::with_capacity(5);
    for graha in bearers.claimants() {
        let (vishwa, sign) = table.of(graha)?;
        claims.push(Claim {
            graha,
            vishwa,
            aspects_lagna: aspects(sign, lagna),
            portfolios: u8::try_from(bearers.portfolios(graha).len()).unwrap_or(u8::MAX),
        });
    }
    // Strongest first, and on a tie the one holding more portfolios, which
    // is the order the chain reads them in.
    claims.sort_by(|a, b| {
        b.vishwa
            .cmp(&a.vishwa)
            .then(b.portfolios.cmp(&a.portfolios))
            .then(a.graha.id().cmp(&b.graha.id()))
    });
    let (mut graha, mut chosen, mut moon_passed_over) = chain(&claims, bearers, rules);
    // Wherever the chain lands on the Moon, it is "unable to govern" and
    // its successor takes the year, unless the rules let it rule.
    if graha == Graha::Moon && rules.moon != MoonMayRule::LikeAnyOther {
        (graha, chosen) = successor(bearers, sky, &table, rules)?;
        moon_passed_over = true;
    }
    Ok(Varshesha {
        graha,
        chosen,
        vishwa: table.of(graha)?.0,
        claims,
        moon_passed_over,
    })
}

/// The strength table, looked up by planet and refused by name where it
/// does not carry one.
struct Strengths<'a>(&'a [Panchavargiya]);

impl Strengths<'_> {
    fn of(&self, graha: Graha) -> Result<(Bala, Rashi), Error> {
        self.0
            .iter()
            .find(|one| one.graha == graha)
            .map(|one| (one.vishwa, one.sign))
            .ok_or_else(|| {
                Error::invalid_arg(format!("no strength for {graha:?}")).with_field("strengths")
            })
    }
}

/// The chain over the ranked claims, before the Moon's own rule: the year
/// lord, the step that chose it, and whether the Moon stepped down for the
/// next one that aspects.
fn chain(
    claims: &[Claim],
    bearers: &OfficeBearers,
    rules: VarsheshaRules,
) -> (Graha, Chosen, bool) {
    let muntha_lord = bearers.holder(Office::Muntha);
    // 1. Every office-bearer too weak to hold the year at all.
    if claims.iter().all(|claim| claim.vishwa < WEAK_BELOW) {
        return (muntha_lord, Chosen::MunthaLordAllWeak, false);
    }
    // 2. Only those that aspect the lagna may hold it — and under the
    //    default the Moon, "unable to govern", steps aside for the next one
    //    down, where there is one.
    let mut aspecting: Vec<&Claim> = claims.iter().filter(|claim| claim.aspects_lagna).collect();
    let mut stepped_down = false;
    if rules.moon == MoonMayRule::PassedOver
        && aspecting
            .first()
            .is_some_and(|claim| claim.graha == Graha::Moon)
        && aspecting.len() > 1
    {
        aspecting.remove(0);
        stepped_down = true;
    }
    let Some(best) = aspecting.first().copied() else {
        let (graha, chosen) = match rules.none_aspects {
            NoneAspects::MunthaLord => (muntha_lord, Chosen::MunthaLordUnaspected),
            NoneAspects::AnnualLagnaLord => (
                bearers.holder(Office::VarshaLagna),
                Chosen::AnnualLagnaLordUnaspected,
            ),
            // The claims are ranked, so the first is the strongest.
            NoneAspects::Strongest => claims
                .first()
                .map_or((muntha_lord, Chosen::MunthaLordUnaspected), |claim| {
                    (claim.graha, Chosen::StrongestUnaspected)
                }),
        };
        return (graha, chosen, stepped_down);
    };
    // 3, 4 and 5. The strongest of them; on a tie of strength the most
    // portfolios; and on a tie of that too the reading of an outright tie.
    let (graha, chosen) = break_the_tie(&aspecting, best, bearers, rules);
    (graha, chosen, stepped_down)
}

/// Who takes the year in the Moon's place: the planet in Ithasala with it,
/// the strongest of several, and otherwise the lord of its sign.
///
/// "Strongest" by the same Vishwa bala the claims are ranked by. The
/// sources name no tie-break for that, so an exact tie goes to the first
/// in [`SEVEN`]'s weekday order, which keeps the answer independent of how
/// the strength table is ordered.
fn successor(
    bearers: &OfficeBearers,
    sky: &AnnualSky,
    table: &Strengths<'_>,
    rules: VarsheshaRules,
) -> Result<(Graha, Chosen), Error> {
    let claimants = bearers.claimants();
    let mut best: Option<(Bala, Graha)> = None;
    for graha in SEVEN {
        if graha == Graha::Moon
            || (rules.moon_partner == MoonPartner::OfficeBearer && !claimants.contains(&graha))
        {
            continue;
        }
        let pair = between_with_rules(Graha::Moon, graha, sky, rules.drishti)?;
        if !pair.yoga.is_some_and(crate::Yoga::is_ithasala) {
            continue;
        }
        let (vishwa, _) = table.of(graha)?;
        if best.is_none_or(|(strongest, _)| vishwa > strongest) {
            best = Some((vishwa, graha));
        }
    }
    Ok(match best {
        Some((_, graha)) => (graha, Chosen::MoonsIthasala),
        None => (
            sign_of_longitude(sky.moon_deg).attributes().lord,
            Chosen::MoonsSignLord,
        ),
    })
}

/// Who takes the year among those tied with the strongest, and why.
///
/// Its own function because the chain's last three steps read as one
/// decision — strength, then portfolios, then the reading of an outright
/// tie — and beside the two fallbacks above they obscured each other.
fn break_the_tie(
    aspecting: &[&Claim],
    best: &Claim,
    bearers: &OfficeBearers,
    rules: VarsheshaRules,
) -> (Graha, Chosen) {
    let tied: Vec<&&Claim> = aspecting
        .iter()
        .filter(|claim| claim.vishwa == best.vishwa)
        .collect();
    if tied.len() < 2 {
        return (best.graha, Chosen::Strongest);
    }
    let most = tied.iter().map(|claim| claim.portfolios).max().unwrap_or(0);
    let mut holders = tied.iter().filter(|claim| claim.portfolios == most);
    match (holders.next(), holders.next()) {
        // One of them holds more portfolios than the rest.
        (Some(winner), None) => (winner.graha, Chosen::MostPortfolios),
        // Tied on strength, aspect and portfolios alike.
        _ => match rules.tied {
            Tied::MunthaLord => (bearers.holder(Office::Muntha), Chosen::MunthaLordTied),
            Tied::DinaRatriPati => (bearers.holder(Office::DinaRatri), Chosen::DinaRatriTied),
        },
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        Chosen, MoonMayRule, MoonPartner, NoneAspects, Tied, VarsheshaRules, WEAK_BELOW, aspects,
        varshesha,
    };
    use crate::bala::{AnnualSky, Bala, Panchavargiya, panchavargiya};
    use crate::office::{OfficeBearers, YearCharts, office_bearers};
    use teistro_core::catalogue::{Graha, Rashi};

    /// The source's Example Chart, whose annual lagna is Scorpio 9°26′.
    fn worked() -> (OfficeBearers, AnnualSky, f64) {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        let sky = AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        };
        let lagna = at(7.0, 9.0, 26.0);
        let bearers = office_bearers(&YearCharts {
            natal_lagna_deg: at(4.0, 14.0, 36.0),
            completed_years: 40,
            annual_lagna_deg: lagna,
            annual_sun_deg: sky.sun_deg,
            annual_moon_deg: sky.moon_deg,
            by_day: true,
        })
        .unwrap();
        (bearers, sky, lagna)
    }

    /// The source's own worked year, and the reason it is worth having:
    /// **the strongest office-bearer does not hold the year**.
    ///
    /// Jupiter is the strongest of the three at 14:46:00, and it stands in
    /// the *second* from the Scorpio lagna — a neutral house, which gives
    /// no Tajika aspect — so the source disqualifies it in as many words.
    /// Of the remaining two the **Sun** is stronger at 14:20:15, and it is
    /// the Varshesha. Saturn is stronger than all three (16:47:45) and
    /// holds no portfolio at all, so it never enters the reckoning.
    #[test]
    fn the_sources_worked_year_is_reproduced() {
        let (bearers, sky, lagna) = worked();
        let strengths = panchavargiya(&sky).unwrap();
        let found =
            varshesha(&bearers, &sky, &strengths, lagna, VarsheshaRules::default()).unwrap();
        assert_eq!(found.graha, Graha::Sun);
        assert_eq!(found.vishwa.to_string(), "14:20:15");
        assert_eq!(found.chosen, Chosen::Strongest);
        assert!(found.chosen.on_strength());
        assert!(!found.moon_passed_over);
        // Three claimants, strongest first: Jupiter leads and is passed
        // over on the aspect, which is the whole decision in one list.
        assert_eq!(
            found
                .claims
                .iter()
                .map(|claim| (claim.graha, claim.aspects_lagna))
                .collect::<Vec<(Graha, bool)>>(),
            [
                (Graha::Jupiter, false),
                (Graha::Sun, true),
                (Graha::Mars, true)
            ]
        );
        assert_eq!(found.claims[0].vishwa.to_string(), "14:46:00");
        assert_eq!(found.claims[2].vishwa.to_string(), "14:01:00");
        // The Sun holds two portfolios, the natal lagna's and the day's.
        assert_eq!(found.claims[1].portfolios, 2);
    }

    /// The Moon, "unable to govern", steps aside for the next one down
    /// even when it is the strongest office-bearer that aspects — and
    /// holds the year under the other reading.
    #[test]
    fn the_moon_is_passed_over_unless_the_rules_allow_it() {
        let (bearers, sky, lagna) = worked();
        // A table in which the Moon is strongest and everything aspects
        // the lagna, the Moon among them.
        let strengths: Vec<crate::bala::Panchavargiya> = crate::bala::SEVEN
            .into_iter()
            .map(|graha| crate::bala::Panchavargiya {
                graha,
                sign: Rashi::Scorpio,
                griha: Bala::default(),
                uchcha: Bala::default(),
                hudda: Bala::default(),
                drekkana: Bala::default(),
                navamsha: Bala::default(),
                total: Bala::default(),
                vishwa: if graha == Graha::Moon {
                    Bala::new(18, 0, 0)
                } else if graha == Graha::Sun {
                    Bala::new(12, 0, 0)
                } else {
                    Bala::new(6, 0, 0)
                },
            })
            .collect();
        // Give the Moon a portfolio by making it the Dina-Ratri lord.
        let lunar = OfficeBearers {
            dina_ratri: Graha::Moon,
            ..bearers
        };
        let passed = varshesha(&lunar, &sky, &strengths, lagna, VarsheshaRules::default()).unwrap();
        assert_ne!(passed.graha, Graha::Moon);
        assert!(passed.moon_passed_over);
        assert_eq!(passed.graha, Graha::Sun, "the next one down that aspects");

        let allowed = varshesha(
            &lunar,
            &sky,
            &strengths,
            lagna,
            VarsheshaRules {
                moon: MoonMayRule::LikeAnyOther,
                ..VarsheshaRules::default()
            },
        )
        .unwrap();
        assert_eq!(allowed.graha, Graha::Moon);
        assert!(!allowed.moon_passed_over);
    }

    /// The source's **second** worked year (Chart VII-1), read off the
    /// page: Gemini rising at 12°53′.
    fn chart_vii_1() -> (OfficeBearers, AnnualSky, f64) {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        let sky = AnnualSky {
            sun_deg: at(7.0, 0.0, 40.0),
            moon_deg: at(2.0, 18.0, 57.0),
            mars_deg: at(6.0, 14.0, 43.0),
            mercury_deg: at(7.0, 4.0, 8.0),
            jupiter_deg: at(2.0, 16.0, 35.0),
            venus_deg: at(8.0, 17.0, 28.0),
            saturn_deg: at(8.0, 16.0, 57.0),
        };
        // "Three planets (the Moon, Mercury, and Venus) share among
        // themselves the five portfolios": the Muntha in Libra, a Cancer
        // birth, and Mercury "the lord of the lagna in the annual chart as
        // also the Tri-Rashi Pati and the Dina-Ratri Pati" -- which, with
        // the Sun in Scorpio, makes it a night return.
        let bearers = OfficeBearers {
            muntha: Graha::Venus,
            janma_lagna: Graha::Moon,
            varsha_lagna: Graha::Mercury,
            tri_rashi: Graha::Mercury,
            dina_ratri: Graha::Mercury,
            by_day: false,
        };
        (bearers, sky, at(2.0, 12.0, 53.0))
    }

    /// Chart VII-1 under Charak's default: the Moon leads and steps down,
    /// Mercury is stronger than Venus but stands in the sixth, and **Venus**
    /// holds the year, as printed.
    ///
    /// The Moon's 12:28:15 and Mercury's 10:49:15 are the book's to the
    /// sub-unit. Its Venus, 9:06:30, is not: the chain gives 7:14:00, the
    /// house part read with Jupiter opposite as an enemy where the book
    /// reads it neutral (`03-design/varshesha.md`). The answer does not
    /// turn on it.
    #[test]
    fn the_sources_second_worked_year_steps_down_to_venus() {
        let (bearers, sky, lagna) = chart_vii_1();
        let strengths = panchavargiya(&sky).unwrap();
        let found =
            varshesha(&bearers, &sky, &strengths, lagna, VarsheshaRules::default()).unwrap();
        assert_eq!(found.graha, Graha::Venus, "the source's answer");
        assert_eq!(found.chosen, Chosen::Strongest);
        assert!(found.moon_passed_over, "the Moon led and stepped aside");
        let printed = |graha: Graha| {
            found
                .claims
                .iter()
                .find(|claim| claim.graha == graha)
                .map(|claim| (claim.vishwa.to_string(), claim.aspects_lagna))
                .unwrap()
        };
        assert_eq!(printed(Graha::Moon), (String::from("12:28:15"), true));
        assert_eq!(printed(Graha::Mercury), (String::from("10:49:15"), false));
        assert_eq!(printed(Graha::Venus), (String::from("07:14:00"), true));
    }

    /// Chart VII-1 under the *Nilakanthi*'s reading, which Charak gives as
    /// "some authorities": no step down. The Moon has "just moved ahead of
    /// Venus", and past every other planet it aspects, so it forms no
    /// Ithasala, and "the next consideration falls on the lord of the Moon
    /// sign, which is Mercury".
    #[test]
    fn the_sources_second_worked_year_falls_to_the_moons_sign_lord() {
        let (bearers, sky, lagna) = chart_vii_1();
        let strengths = panchavargiya(&sky).unwrap();
        let rules = VarsheshaRules {
            moon: MoonMayRule::Ithasala,
            ..VarsheshaRules::default()
        };
        let found = varshesha(&bearers, &sky, &strengths, lagna, rules).unwrap();
        assert_eq!(found.graha, Graha::Mercury, "the source's answer");
        assert_eq!(found.chosen, Chosen::MoonsSignLord);
        assert!(found.chosen.succeeds_the_moon() && !found.chosen.on_strength());
        assert!(found.moon_passed_over);
        assert_eq!(found.vishwa.to_string(), "10:49:15");
    }

    /// Aries rising at 1°. The office-bearers are Mars (in Taurus, the
    /// second), the Sun (in Virgo, the sixth) and the Moon, in Aries at 10°
    /// and the only one of them that aspects the lagna. Jupiter at Leo 15°
    /// is five degrees ahead of the Moon in a trine: an Ithasala. Nothing
    /// else is: Venus is in Taurus, Mercury in Virgo and Saturn in Scorpio,
    /// none of them aspecting Aries.
    fn lone_moon() -> (OfficeBearers, AnnualSky) {
        let bearers = OfficeBearers {
            muntha: Graha::Mars,
            janma_lagna: Graha::Moon,
            varsha_lagna: Graha::Mars,
            tri_rashi: Graha::Sun,
            dina_ratri: Graha::Sun,
            by_day: true,
        };
        let sky = AnnualSky {
            sun_deg: 155.0,
            moon_deg: 10.0,
            mars_deg: 35.0,
            mercury_deg: 158.0,
            jupiter_deg: 135.0,
            venus_deg: 50.0,
            saturn_deg: 230.0,
        };
        (bearers, sky)
    }

    /// A strength table read off `sky`'s signs, with the strengths chosen.
    fn ranked(sky: &AnnualSky, vishwa: impl Fn(Graha) -> Bala) -> Vec<Panchavargiya> {
        crate::bala::SEVEN
            .into_iter()
            .map(|graha| Panchavargiya {
                graha,
                sign: crate::bala::sign_of_longitude(sky.longitude_of(graha)),
                griha: Bala::default(),
                uchcha: Bala::default(),
                hudda: Bala::default(),
                drekkana: Bala::default(),
                navamsha: Bala::default(),
                total: Bala::default(),
                vishwa: vishwa(graha),
            })
            .collect()
    }

    fn moon_leads(graha: Graha) -> Bala {
        match graha {
            Graha::Moon => Bala::new(12, 0, 0),
            Graha::Mars => Bala::new(8, 0, 0),
            Graha::Sun => Bala::new(7, 0, 0),
            _ => Bala::new(6, 0, 0),
        }
    }

    /// The Moon is the only office-bearer that aspects, so there is no one
    /// to step down to, and "it still does not become the year lord": the
    /// planet in Ithasala with it does, office-bearer or not. The build
    /// answered the Moon here until the successor was built.
    #[test]
    fn a_lone_moon_is_succeeded_by_its_ithasala() {
        let (bearers, sky) = lone_moon();
        let strengths = ranked(&sky, moon_leads);
        let found = varshesha(&bearers, &sky, &strengths, 1.0, VarsheshaRules::default()).unwrap();
        assert_eq!(found.graha, Graha::Jupiter);
        assert_eq!(found.chosen, Chosen::MoonsIthasala);
        assert!(found.moon_passed_over);
        assert_eq!(found.vishwa, Bala::new(6, 0, 0), "its own strength");
        assert!(!bearers.claimants().contains(&Graha::Jupiter));

        // Narrowed to the office-bearers, Jupiter cannot succeed it, no
        // office-bearer is in Ithasala with the Moon, and the lord of its
        // sign, Aries, takes the year.
        let narrowed = VarsheshaRules {
            moon_partner: MoonPartner::OfficeBearer,
            ..VarsheshaRules::default()
        };
        let found = varshesha(&bearers, &sky, &strengths, 1.0, narrowed).unwrap();
        assert_eq!(
            (found.graha, found.chosen),
            (Graha::Mars, Chosen::MoonsSignLord)
        );

        // And allowed to rule, it rules.
        let allowed = VarsheshaRules {
            moon: MoonMayRule::LikeAnyOther,
            ..VarsheshaRules::default()
        };
        let found = varshesha(&bearers, &sky, &strengths, 1.0, allowed).unwrap();
        assert_eq!(
            (found.graha, found.chosen),
            (Graha::Moon, Chosen::Strongest)
        );
        assert!(!found.moon_passed_over);
    }

    /// Several in Ithasala with the Moon: "the strongest of them becomes
    /// the year lord". Saturn at Sagittarius 14°, four degrees ahead of the
    /// Moon in the other trine, joins Jupiter, and is made the stronger.
    #[test]
    fn of_several_in_ithasala_the_strongest_succeeds() {
        let (bearers, lone) = lone_moon();
        let sky = AnnualSky {
            saturn_deg: 254.0,
            ..lone
        };
        let strengths = ranked(&sky, |graha| match graha {
            Graha::Saturn => Bala::new(9, 0, 0),
            other => moon_leads(other),
        });
        let found = varshesha(&bearers, &sky, &strengths, 1.0, VarsheshaRules::default()).unwrap();
        assert_eq!(
            (found.graha, found.chosen),
            (Graha::Saturn, Chosen::MoonsIthasala)
        );
    }

    /// A Moon in its own Cancer, in Ithasala with nothing, is the lord of
    /// its own sign, and "that very Moon is the year lord", as the
    /// *Nilakanthi*'s commentary has it. Passed over, and back.
    #[test]
    fn a_moon_in_cancer_succeeds_itself() {
        let (bearers, _) = lone_moon();
        // The Moon at Cancer 20° aspects Aries from the fourth; the Sun,
        // Mercury, Mars and Venus are all behind it in signs it aspects, so
        // each is an Ishrafa; Jupiter in Leo and Saturn in Sagittarius are
        // in the second and sixth from it, which aspect nothing.
        let sky = AnnualSky {
            sun_deg: 155.0,
            moon_deg: 110.0,
            mars_deg: 35.0,
            mercury_deg: 158.0,
            jupiter_deg: 135.0,
            venus_deg: 40.0,
            saturn_deg: 265.0,
        };
        let strengths = ranked(&sky, moon_leads);
        let found = varshesha(&bearers, &sky, &strengths, 1.0, VarsheshaRules::default()).unwrap();
        assert_eq!(
            (found.graha, found.chosen),
            (Graha::Moon, Chosen::MoonsSignLord)
        );
        assert!(found.moon_passed_over);
    }

    /// A Moon that inherits the year through a fallback is still the Moon:
    /// it holds the Muntha's portfolio, nobody aspects the lagna, and its
    /// Ithasala successor takes the year.
    #[test]
    fn a_moon_reached_by_a_fallback_is_succeeded_too() {
        let (lone, lone_sky) = lone_moon();
        let bearers = OfficeBearers {
            muntha: Graha::Moon,
            ..lone
        };
        // The Moon at Taurus 10° aspects nothing in Aries, and Jupiter at
        // Leo 15° is five degrees ahead of it from the fourth.
        let sky = AnnualSky {
            moon_deg: 40.0,
            ..lone_sky
        };
        let strengths = ranked(&sky, moon_leads);
        let found = varshesha(&bearers, &sky, &strengths, 1.0, VarsheshaRules::default()).unwrap();
        assert!(found.claims.iter().all(|claim| !claim.aspects_lagna));
        assert_eq!(
            (found.graha, found.chosen),
            (Graha::Jupiter, Chosen::MoonsIthasala)
        );
        assert!(found.moon_passed_over);
    }

    /// Nobody aspecting the lagna gives the year to the Muntha's lord
    /// (Charak), the annual lagna's lord ("some authorities") or the
    /// strongest of the five (the *Nilakanthi*'s v. 11).
    #[test]
    fn nobody_aspecting_has_three_readings() {
        let (lone, lone_sky) = lone_moon();
        let bearers = OfficeBearers {
            muntha: Graha::Sun,
            ..lone
        };
        let sky = AnnualSky {
            moon_deg: 40.0,
            ..lone_sky
        };
        let strengths = ranked(&sky, |graha| match graha {
            Graha::Mars => Bala::new(14, 0, 0),
            other => moon_leads(other),
        });
        let ask = |none_aspects| {
            varshesha(
                &bearers,
                &sky,
                &strengths,
                1.0,
                VarsheshaRules {
                    none_aspects,
                    ..VarsheshaRules::default()
                },
            )
            .unwrap()
        };
        let charak = ask(NoneAspects::MunthaLord);
        assert!(charak.claims.iter().all(|claim| !claim.aspects_lagna));
        assert_eq!(
            (charak.graha, charak.chosen),
            (Graha::Sun, Chosen::MunthaLordUnaspected)
        );
        let lagna_lord = ask(NoneAspects::AnnualLagnaLord);
        assert_eq!(
            (lagna_lord.graha, lagna_lord.chosen),
            (Graha::Mars, Chosen::AnnualLagnaLordUnaspected)
        );
        let strongest = ask(NoneAspects::Strongest);
        assert_eq!(
            (strongest.graha, strongest.chosen),
            (Graha::Mars, Chosen::StrongestUnaspected)
        );
        assert!(!strongest.chosen.on_strength(), "a fallback, strong or not");
    }

    /// The readings cross in the boundary's casing, partial records filled
    /// from the defaults, and a reading nobody wrote is refused.
    #[test]
    fn the_rules_read_camel_case_and_fill_the_rest() {
        let read: VarsheshaRules = serde_json::from_str(
            r#"{"moon": "ITHASALA", "moonPartner": "OFFICE_BEARER", "noneAspects": "STRONGEST"}"#,
        )
        .unwrap();
        assert_eq!(read.moon, MoonMayRule::Ithasala);
        assert_eq!(read.moon_partner, MoonPartner::OfficeBearer);
        assert_eq!(read.none_aspects, NoneAspects::Strongest);
        assert_eq!(read.tied, Tied::MunthaLord);
        assert_eq!(read.drishti, crate::DrishtiRules::default());
        assert!(serde_json::from_str::<VarsheshaRules>(r#"{"moonPartner": "ANY"}"#).is_err());
    }

    /// The Tajika aspect is the houses 3, 5, 9, 11 and the kendras; the
    /// neutral houses 2, 6, 8 and 12 give none. Symmetric, so counting
    /// from either end answers alike.
    #[test]
    fn the_neutral_houses_give_no_aspect_and_the_rest_do() {
        let lagna = Rashi::Aries;
        let mut aspected = 0;
        for graha in Rashi::ALL {
            let house = (lagna.id() + 12 - graha.id()) % 12 + 1;
            let expected = !matches!(house, 2 | 6 | 8 | 12);
            assert_eq!(aspects(graha, lagna), expected, "house {house}");
            assert_eq!(aspects(graha, lagna), aspects(lagna, graha), "symmetric");
            aspected += usize::from(expected);
        }
        assert_eq!(aspected, 8, "eight of the twelve houses aspect");
    }

    /// A planet that does not aspect the lagna cannot hold the year, however
    /// strong it is: the next one down that does takes it.
    #[test]
    fn the_strongest_that_does_not_aspect_is_passed_over() {
        let (bearers, sky, _) = worked();
        let strengths = panchavargiya(&sky).unwrap();
        // Cancer rising: Jupiter in Sagittarius is in the sixth from it and
        // aspects nothing; the Sun in Leo is in the second, also nothing;
        // Mars in Scorpio is in the fifth, which is a friendly aspect.
        let lagna = 3.0 * 30.0 + 10.0;
        let found =
            varshesha(&bearers, &sky, &strengths, lagna, VarsheshaRules::default()).unwrap();
        assert_eq!(found.graha, Graha::Mars);
        assert_eq!(found.chosen, Chosen::Strongest);
        let jupiter = found
            .claims
            .iter()
            .find(|claim| claim.graha == Graha::Jupiter)
            .unwrap();
        assert!(!jupiter.aspects_lagna);
        assert!(jupiter.vishwa > found.vishwa, "stronger, and passed over");
    }

    /// Every office-bearer under five units, and the Muntha's lord takes
    /// the year whatever the aspects say.
    #[test]
    fn all_of_them_weak_falls_to_the_muntha_lord() {
        let (bearers, sky, lagna) = worked();
        let feeble: Vec<crate::bala::Panchavargiya> = crate::bala::SEVEN
            .into_iter()
            .map(|graha| crate::bala::Panchavargiya {
                graha,
                sign: Rashi::Aries,
                griha: Bala::default(),
                uchcha: Bala::default(),
                hudda: Bala::default(),
                drekkana: Bala::default(),
                navamsha: Bala::default(),
                total: Bala::new(16, 0, 0),
                vishwa: Bala::new(4, 0, 0),
            })
            .collect();
        let found = varshesha(&bearers, &sky, &feeble, lagna, VarsheshaRules::default()).unwrap();
        assert_eq!(found.chosen, Chosen::MunthaLordAllWeak);
        assert_eq!(found.graha, bearers.muntha);
        assert!(found.vishwa < WEAK_BELOW);
        assert!(!found.chosen.on_strength());
    }

    /// A tie of strength among those aspecting goes to the one holding more
    /// portfolios, and a tie of that too goes to the Muntha's lord.
    #[test]
    fn a_tie_goes_to_the_portfolios_and_then_to_the_muntha_lord() {
        let (bearers, sky, lagna) = worked();
        let level = |vishwa: Bala| -> Vec<crate::bala::Panchavargiya> {
            crate::bala::SEVEN
                .into_iter()
                .map(|graha| crate::bala::Panchavargiya {
                    graha,
                    // Aries, which aspects Scorpio's lagna at the sixth?
                    // No: put every planet in the lagna's own sign, where
                    // house one is an inimical aspect and so an aspect.
                    sign: Rashi::Scorpio,
                    griha: Bala::default(),
                    uchcha: Bala::default(),
                    hudda: Bala::default(),
                    drekkana: Bala::default(),
                    navamsha: Bala::default(),
                    total: Bala::of_sub_sub(vishwa.as_sub_sub() * 4),
                    vishwa,
                })
                .collect()
        };
        // All level at ten units: the Sun and Mars hold two portfolios
        // each, Jupiter one, so the tie is not broken by portfolios alone
        // and the Muntha's lord takes it.
        let found = varshesha(
            &bearers,
            &sky,
            &level(Bala::new(10, 0, 0)),
            lagna,
            VarsheshaRules::default(),
        )
        .unwrap();
        assert_eq!(found.chosen, Chosen::MunthaLordTied);
        assert_eq!(found.graha, bearers.muntha);

        // The other reading gives that same tie to the Dina-Ratri Pati.
        let others = varshesha(
            &bearers,
            &sky,
            &level(Bala::new(10, 0, 0)),
            lagna,
            VarsheshaRules {
                tied: Tied::DinaRatriPati,
                ..VarsheshaRules::default()
            },
        )
        .unwrap();
        assert_eq!(others.chosen, Chosen::DinaRatriTied);
        assert_eq!(others.graha, bearers.dina_ratri);
    }

    #[test]
    fn a_lagna_that_is_not_a_number_is_refused_by_that_field() {
        let (bearers, sky, _) = worked();
        let strengths = panchavargiya(&sky).unwrap();
        let why = varshesha(
            &bearers,
            &sky,
            &strengths,
            f64::NAN,
            VarsheshaRules::default(),
        )
        .expect_err("refused");
        assert_eq!(why.field(), Some("annual_lagna_deg"));
    }
}
