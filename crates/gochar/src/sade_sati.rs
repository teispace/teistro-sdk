//! Sade Sati and Saturn's smaller spells from the natal Moon
//! (`03-design/sade-sati.md`), read off Saturn's crossings of a 30°
//! lattice.
//!
//! No classical verse names the seven and a half years (C89, C147): the
//! definition is practice's, and every choice in it is a knob. What this
//! module adds to the practice is exactness. A retrograde re-entry is
//! kept in its period by **which circuit of the zodiac** it belongs to
//! (C148), not by a gap in days, so no constant decides it.
//!
//! ```
//! use teistro_core::quantity::{JulianDay, Utc};
//! use teistro_gochar::hits::Motion;
//! use teistro_gochar::sade_sati::{Crossing, Outcome, Phase, Reckoning, Searched, periods};
//!
//! let at = |jd| JulianDay::<Utc>::literal(jd);
//! // The natal Moon in Aries. Saturn walks from Capricorn through Pisces
//! // (the 12th), Aries and Taurus to Cancer, a sign every 900 days: the
//! // search reaches two houses past the Sade Sati either side.
//! let crossings: Vec<Crossing> = [300.0, 330.0, 0.0, 30.0, 60.0, 90.0]
//!     .iter()
//!     .zip(0_u32..)
//!     .map(|(line, k)| Crossing::new(at(f64::from(k) * 900.0), *line, Motion::Direct))
//!     .collect();
//! let searched = Searched::new(&crossings, Reckoning::Sign.origin_deg(10.0));
//! let Outcome::Found(found) = periods(&searched, (at(2_000.0), at(2_000.0)), &[4, 8])? else {
//!     unreachable!("the search reaches two houses either side");
//! };
//! let sade_sati = &found.sade_sati[0];
//! assert_eq!(sade_sati.begins(), Some(at(900.0)));
//! assert_eq!(sade_sati.ends(), Some(at(3_600.0)));
//! assert_eq!(sade_sati.phase_at(at(2_000.0)), Some(Phase::Peak));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::Reference;
use crate::hits::{Motion, entered};

/// The houses a zodiac is divided into.
const HOUSES: i64 = 12;

/// What a Sade Sati's houses are reckoned in (C147).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Reckoning {
    /// Whole signs from the reference's sign: the reading every
    /// calculator surveyed prints.
    #[default]
    Sign,
    /// Arcs of 30° with the reference's degree in the middle of the first:
    /// the Sade Sati from 45° before the natal Moon to 45° past it.
    Degree,
}

impl Reckoning {
    /// Every reckoning, for a crossing to hold each to a code of its own.
    pub const ALL: &'static [Reckoning] = &[Reckoning::Sign, Reckoning::Degree];

    /// Where the first house begins for a reference at `reference_deg`, a
    /// sidereal longitude: its sign's start, or 15° before it.
    ///
    /// ```
    /// use teistro_gochar::sade_sati::Reckoning;
    ///
    /// assert_eq!(Reckoning::Sign.origin_deg(33.0), 30.0);
    /// assert_eq!(Reckoning::Degree.origin_deg(33.0), 18.0);
    /// assert_eq!(Reckoning::Degree.origin_deg(5.0), 350.0);
    /// ```
    #[must_use]
    pub fn origin_deg(self, reference_deg: f64) -> f64 {
        let at = reference_deg.rem_euclid(360.0);
        match self {
            Reckoning::Sign => (at / 30.0).floor() * 30.0,
            Reckoning::Degree => (at - 15.0).rem_euclid(360.0),
        }
    }
}

/// A Sade Sati's three phases, one a house (C147).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    /// Saturn in the 12th from the reference.
    Rising,
    /// Saturn in the 1st.
    Peak,
    /// Saturn in the 2nd.
    Setting,
}

impl Phase {
    /// The three, in the order Saturn passes them.
    pub const ALL: [Phase; 3] = [Phase::Rising, Phase::Peak, Phase::Setting];

    /// The house from the reference, 1 to 12.
    #[must_use]
    pub const fn house(self) -> u8 {
        match self {
            Phase::Rising => 12,
            Phase::Peak => 1,
            Phase::Setting => 2,
        }
    }

    /// The phase a house is, if it is one of the three.
    #[must_use]
    pub const fn of(house: u8) -> Option<Phase> {
        match house {
            12 => Some(Phase::Rising),
            1 => Some(Phase::Peak),
            2 => Some(Phase::Setting),
            _ => None,
        }
    }
}

/// The smaller spells counted unless asked otherwise (C149): the 4th from
/// the reference (Kantaka Shani, the small Panoti) and the 8th (Ashtama
/// Shani).
pub const DEFAULT_SPELLS: [u8; 2] = [4, 8];

/// One of Saturn's crossings of the lattice: when, over which line, and
/// which way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crossing {
    /// When.
    pub instant: JulianDay<Utc>,
    /// The line crossed, a sidereal longitude.
    pub boundary_deg: f64,
    /// Which way Saturn was moving.
    pub motion: Motion,
}

impl Crossing {
    /// A crossing of `boundary_deg` at `instant`, moving `motion`.
    #[must_use]
    pub const fn new(instant: JulianDay<Utc>, boundary_deg: f64, motion: Motion) -> Crossing {
        Crossing {
            instant,
            boundary_deg,
            motion,
        }
    }
}

/// One unbroken stay of Saturn in a house, half-open: from the instant it
/// entered to the instant it left. A bound is absent when it lies past what
/// the ephemeris covers.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Visit {
    /// When Saturn entered, a UTC Julian day; absent before the coverage.
    pub from: Option<JulianDay<Utc>>,
    /// When it left; absent after the coverage.
    pub to: Option<JulianDay<Utc>>,
}

impl Visit {
    /// Whether Saturn was in the house at `instant`.
    #[must_use]
    pub fn contains(&self, instant: JulianDay<Utc>) -> bool {
        self.from.is_none_or(|from| from.get() <= instant.get())
            && self.to.is_none_or(|to| instant.get() < to.get())
    }

    /// Whether the stay reaches into the window `[from, to]`.
    fn meets(&self, (from, to): (f64, f64)) -> bool {
        self.from.is_none_or(|start| start.get() <= to)
            && self.to.is_none_or(|end| end.get() > from)
    }
}

/// Saturn's stays in one house from the reference during one circuit of
/// the zodiac: one visit, or several when it turned retrograde across the
/// house's edge and came back (C148).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Spell {
    /// The house from the reference, 1 to 12.
    pub house: u8,
    /// Every stay, in time order.
    pub visits: Vec<Visit>,
}

impl Spell {
    /// When Saturn first entered the house; absent before the coverage.
    #[must_use]
    pub fn begins(&self) -> Option<JulianDay<Utc>> {
        self.visits.first().and_then(|visit| visit.from)
    }

    /// When it last left; absent after the coverage.
    #[must_use]
    pub fn ends(&self) -> Option<JulianDay<Utc>> {
        self.visits.last().and_then(|visit| visit.to)
    }

    /// Whether Saturn stood in the house at `instant`: a day between two
    /// visits, when it had stepped back out, is not.
    #[must_use]
    pub fn contains(&self, instant: JulianDay<Utc>) -> bool {
        self.visits.iter().any(|visit| visit.contains(instant))
    }

    /// The Sade Sati phase this house is, if it is one.
    #[must_use]
    pub const fn phase(&self) -> Option<Phase> {
        Phase::of(self.house)
    }
}

/// One Sade Sati: Saturn's spells in the 12th, the 1st and the 2nd from
/// the reference, on one circuit of the zodiac.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SadeSati {
    /// Rising, peak and setting, in that order; a phase past the coverage
    /// is missing.
    pub phases: Vec<Spell>,
}

impl SadeSati {
    /// When Saturn first entered the 12th.
    #[must_use]
    pub fn begins(&self) -> Option<JulianDay<Utc>> {
        self.phases.first().and_then(Spell::begins)
    }

    /// When it last left the 2nd.
    #[must_use]
    pub fn ends(&self) -> Option<JulianDay<Utc>> {
        self.phases.last().and_then(Spell::ends)
    }

    /// The phase Saturn stood in at `instant`, if it stood in one.
    #[must_use]
    pub fn phase_at(&self, instant: JulianDay<Utc>) -> Option<Phase> {
        self.phases
            .iter()
            .find(|spell| spell.contains(instant))
            .and_then(Spell::phase)
    }

    /// One phase's spell.
    #[must_use]
    pub fn phase(&self, phase: Phase) -> Option<&Spell> {
        self.phases
            .iter()
            .find(|spell| spell.house == phase.house())
    }
}

/// Every period of a window: each Sade Sati and each smaller spell asked
/// for whose span, from its first entry to its last exit, reaches into it;
/// whole.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Periods {
    /// The Sade Satis, in time order.
    pub sade_sati: Vec<SadeSati>,
    /// The smaller spells, in time order.
    pub spells: Vec<Spell>,
}

/// A natal chart's periods over a window, with what they were counted
/// from and how they were reckoned, so a report says which reading it is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Report {
    /// The natal point the houses are counted from, and its sign.
    pub reference: Reference,
    /// Whole signs or the reference's degree (C147).
    pub reckoning: Reckoning,
    /// The Sade Satis reaching into the window, in time order.
    pub sade_sati: Vec<SadeSati>,
    /// The smaller spells asked for reaching into it, in time order.
    pub spells: Vec<Spell>,
}

impl Report {
    /// The periods found, labelled.
    #[must_use]
    pub fn new(reference: Reference, reckoning: Reckoning, periods: Periods) -> Report {
        Report {
            reference,
            reckoning,
            sade_sati: periods.sade_sati,
            spells: periods.spells,
        }
    }

    /// The Sade Sati phase Saturn stood in at `instant`, if any.
    #[must_use]
    pub fn phase_at(&self, instant: JulianDay<Utc>) -> Option<Phase> {
        self.sade_sati
            .iter()
            .find_map(|sade_sati| sade_sati.phase_at(instant))
    }

    /// The smaller spell's house Saturn stood in at `instant`, if any.
    #[must_use]
    pub fn spell_at(&self, instant: JulianDay<Utc>) -> Option<u8> {
        self.spells
            .iter()
            .find(|spell| spell.contains(instant))
            .map(|spell| spell.house)
    }
}

/// What a search found: Saturn's crossings in time order, where the houses
/// begin, and whether each end of the search is the ephemeris's own.
#[derive(Clone, Copy, Debug)]
pub struct Searched<'c> {
    crossings: &'c [Crossing],
    origin_deg: f64,
    covered_before: bool,
    covered_after: bool,
}

impl<'c> Searched<'c> {
    /// `crossings` of the lattice whose first house begins at `origin_deg`
    /// ([`Reckoning::origin_deg`]), from a search that could still widen
    /// either way.
    #[must_use]
    pub const fn new(crossings: &'c [Crossing], origin_deg: f64) -> Searched<'c> {
        Searched {
            crossings,
            origin_deg,
            covered_before: false,
            covered_after: false,
        }
    }

    /// The same search, which began (`before`) or ended (`after`) at the
    /// edge of the ephemeris's coverage and so cannot widen that way.
    #[must_use]
    pub const fn at_coverage(mut self, before: bool, after: bool) -> Searched<'c> {
        self.covered_before = before;
        self.covered_after = after;
        self
    }
}

/// What [`periods`] makes of a search.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The answer, which no wider search could change.
    Found(Periods),
    /// The search must reach further before (`before`) or after (`after`)
    /// for a period's bound to be certain.
    Widen {
        /// Earlier.
        before: bool,
        /// Later.
        after: bool,
    },
}

/// A period being gathered: Sade Sati `k` (the circuit) or house `h`'s
/// spell on circuit `k`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    SadeSati(i64),
    Spell(u8, i64),
}

impl Key {
    /// The period an unwound house belongs to, if any: a Sade Sati's
    /// three houses, `12k + 11` to `12k + 13` counted from 0, are circuit
    /// `k + 1`'s.
    fn of(unwound: i64, houses: &[u8]) -> Option<Key> {
        let shifted = unwound + 1;
        if shifted.rem_euclid(HOUSES) < 3 {
            return Some(Key::SadeSati(shifted.div_euclid(HOUSES)));
        }
        houses.iter().find_map(|house| {
            let index = i64::from(*house) - 1;
            (unwound.rem_euclid(HOUSES) == index)
                .then(|| Key::Spell(*house, (unwound - index).div_euclid(HOUSES)))
        })
    }

    /// The first and last unwound house of the period.
    fn houses(self) -> (i64, i64) {
        match self {
            Key::SadeSati(circuit) => (HOUSES * circuit - 1, HOUSES * circuit + 1),
            Key::Spell(house, circuit) => {
                let at = HOUSES * circuit + i64::from(house) - 1;
                (at, at)
            }
        }
    }
}

/// One stay of Saturn's, with the house it stood in unwound.
#[derive(Clone, Copy, Debug)]
struct Stay {
    unwound: i64,
    visit: Visit,
}

/// The Sade Satis and the smaller spells in `houses` that reach into
/// `window`, whole (`03-design/sade-sati.md` §3).
///
/// Walking the crossings, the house Saturn stands in is **unwound**: each
/// pass from the 12th into the 1st adds a circuit and each pass back takes
/// one away, so `12 × circuit + house` changes by exactly one at every
/// crossing. The unwound house groups the visits: a retrograde loop keeps
/// its circuit, and two Sade Satis are a circuit apart.
///
/// Certain means Saturn stood **two houses** short of the earliest period
/// when the search began, and two past the latest when it ended: a
/// retrograde arc is far shorter than a house, so nothing outside the
/// search could have belonged to either.
///
/// # Errors
///
/// `INVALID_ARG` naming `houses` for a house outside 3 to 11 (the Sade
/// Sati's own three are always counted) or named twice; `INTERNAL` when
/// two crossings in a row are not neighbouring lines, which means the
/// search missed one.
pub fn periods(
    searched: &Searched<'_>,
    (from, to): (JulianDay<Utc>, JulianDay<Utc>),
    houses: &[u8],
) -> Result<Outcome, Error> {
    check_houses(houses)?;
    let stays = unwind(searched)?;
    let (Some(first), Some(last)) = (stays.first(), stays.last()) else {
        // Saturn stays in no house for six years, so a search without a
        // crossing is one too narrow to say anything.
        return if searched.covered_before && searched.covered_after {
            Err(Error::internal(
                "Saturn crossed no line of the lattice in the ephemeris's whole coverage",
            ))
        } else {
            Ok(Outcome::Widen {
                before: !searched.covered_before,
                after: !searched.covered_after,
            })
        };
    };
    let (first, last) = (first.unwound, last.unwound);

    // Every period's stays, in time order within each.
    let mut gathered: Vec<(Key, Vec<Stay>)> = Vec::new();
    for stay in stays {
        let Some(key) = Key::of(stay.unwound, houses) else {
            continue;
        };
        if let Some((_, known)) = gathered.iter_mut().find(|(known, _)| *known == key) {
            known.push(stay);
        } else {
            gathered.push((key, vec![stay]));
        }
    }
    // A period reaches into the window across its whole span, the days it
    // stepped back out between two visits included: a Sade Sati paused by
    // a retrograde loop has begun and not ended.
    let window = (from.get(), to.get());
    gathered.retain(|(_, stays)| {
        let span = Visit {
            from: stays.first().and_then(|stay| stay.visit.from),
            to: stays.last().and_then(|stay| stay.visit.to),
        };
        span.meets(window)
    });

    // Certain only if the search began two houses before every period it
    // reports, and ended two after.
    let earliest = gathered.iter().map(|(key, _)| key.houses().0).min();
    let latest = gathered.iter().map(|(key, _)| key.houses().1).max();
    let before = earliest.is_some_and(|house| first > house - 2) && !searched.covered_before;
    let after = latest.is_some_and(|house| last < house + 2) && !searched.covered_after;
    if before || after {
        return Ok(Outcome::Widen { before, after });
    }

    gathered.sort_by_key(|(key, _)| *key);
    let mut found = Periods::default();
    for (key, stays) in gathered {
        match key {
            Key::SadeSati(_) => found.sade_sati.push(SadeSati {
                phases: spells_of(&stays),
            }),
            Key::Spell(..) => found.spells.extend(spells_of(&stays)),
        }
    }
    found.spells.sort_by(|a, b| {
        let at = |spell: &Spell| spell.begins().map_or(f64::NEG_INFINITY, JulianDay::get);
        at(a).total_cmp(&at(b)).then(a.house.cmp(&b.house))
    });
    Ok(Outcome::Found(found))
}

/// Saturn's stays between the crossings, each with its house unwound onto
/// circuit 0 at the first; the first stay's start and the last's end are
/// absent, because the search did not see them. None without a crossing.
fn unwind(searched: &Searched<'_>) -> Result<Vec<Stay>, Error> {
    let house_of = |crossing: &Crossing| -> i64 {
        let index = entered(
            crossing.boundary_deg - searched.origin_deg,
            crossing.motion,
            12,
        );
        i64::try_from(index).unwrap_or(0)
    };
    let step = |motion: Motion| match motion {
        Motion::Direct => 1,
        Motion::Retrograde => -1,
    };
    let Some(first) = searched.crossings.first() else {
        return Ok(Vec::new());
    };
    let mut unwound = house_of(first) - step(first.motion);
    let mut stays = Vec::with_capacity(searched.crossings.len() + 1);
    let mut since = None;
    for crossing in searched.crossings {
        let next = unwound + step(crossing.motion);
        if next.rem_euclid(HOUSES) != house_of(crossing) {
            return Err(Error::internal(format!(
                "Saturn's crossing at {} is not the neighbour of the house before it: \
                 the search missed a line",
                crossing.instant.get()
            )));
        }
        stays.push(Stay {
            unwound,
            visit: Visit {
                from: since,
                to: Some(crossing.instant),
            },
        });
        unwound = next;
        since = Some(crossing.instant);
    }
    stays.push(Stay {
        unwound,
        visit: Visit {
            from: since,
            to: None,
        },
    });
    Ok(stays)
}

/// The stays of one period grouped into a spell per unwound house, in the
/// order Saturn passes them.
fn spells_of(stays: &[Stay]) -> Vec<Spell> {
    let mut by_house: Vec<(i64, Spell)> = Vec::new();
    for stay in stays {
        if let Some((_, spell)) = by_house
            .iter_mut()
            .find(|(known, _)| *known == stay.unwound)
        {
            spell.visits.push(stay.visit);
        } else {
            let house = u8::try_from(stay.unwound.rem_euclid(HOUSES) + 1).unwrap_or(1);
            by_house.push((
                stay.unwound,
                Spell {
                    house,
                    visits: vec![stay.visit],
                },
            ));
        }
    }
    by_house.sort_by_key(|(unwound, _)| *unwound);
    by_house.into_iter().map(|(_, spell)| spell).collect()
}

/// Refuses a smaller spell's house outside 3 to 11, or one named twice.
///
/// # Errors
///
/// `INVALID_ARG` naming `houses`.
pub fn check_houses(houses: &[u8]) -> Result<(), Error> {
    for (index, house) in houses.iter().enumerate() {
        if !(3..=11).contains(house) {
            return Err(Error::invalid_arg(format!(
                "house {house} is not a smaller spell of Saturn's"
            ))
            .with_field("houses")
            .with_hint("name houses 3 to 11 from the reference; the 12th, 1st and 2nd are the Sade Sati's own"));
        }
        if houses
            .get(..index)
            .is_some_and(|before| before.contains(house))
        {
            return Err(
                Error::invalid_arg(format!("house {house} is named twice")).with_field("houses")
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they built"
    )]

    use super::*;

    const EPOCH: f64 = 2_451_545.0;

    fn at(days: f64) -> JulianDay<Utc> {
        JulianDay::<Utc>::literal(EPOCH + days)
    }

    /// Saturn's path as the lines it crosses, each `(days, line, motion)`.
    fn path(steps: &[(f64, f64, Motion)]) -> Vec<Crossing> {
        steps
            .iter()
            .map(|(days, line, motion)| Crossing::new(at(*days), *line, *motion))
            .collect()
    }

    fn found(searched: &Searched<'_>, window: (f64, f64), houses: &[u8]) -> Periods {
        match periods(searched, (at(window.0), at(window.1)), houses).unwrap() {
            Outcome::Found(found) => found,
            widen @ Outcome::Widen { .. } => panic!("{widen:?}"),
        }
    }

    use Motion::{Direct, Retrograde};

    /// Saturn direct through every line from 300° to 240°, one every 900
    /// days: Capricorn before the first, Sagittarius after the last.
    fn straight() -> Vec<Crossing> {
        path(
            &(0..14)
                .map(|k| {
                    (
                        900.0 * f64::from(k),
                        (300.0 + 30.0 * f64::from(k)).rem_euclid(360.0),
                        Direct,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn three_phases_in_order_bounded_by_their_crossings() {
        // The Moon in Aries: Pisces from 900 (line 330°), Aries from 1800,
        // Taurus from 2700, Gemini from 3600.
        let crossings = straight();
        let searched = Searched::new(&crossings, 0.0);
        let periods = found(&searched, (2_000.0, 2_000.0), &[]);
        assert_eq!(periods.sade_sati.len(), 1);
        let one = &periods.sade_sati[0];
        assert_eq!(
            one.phases
                .iter()
                .map(|spell| spell.house)
                .collect::<Vec<_>>(),
            [12, 1, 2]
        );
        assert_eq!(one.begins(), Some(at(900.0)));
        assert_eq!(one.ends(), Some(at(3_600.0)));
        assert_eq!(one.phase(Phase::Peak).unwrap().begins(), Some(at(1_800.0)));
        assert_eq!(one.phase_at(at(1_000.0)), Some(Phase::Rising));
        assert_eq!(one.phase_at(at(3_599.0)), Some(Phase::Setting));
        assert_eq!(one.phase_at(at(3_600.0)), None);
    }

    #[test]
    fn a_retrograde_re_entry_stays_in_its_period() {
        // Into Pisces at 100, back into Aquarius at 200, Pisces again at
        // 300, Aries at 1000, back into Pisces at 1100, Aries at 1200,
        // Taurus at 2000, Gemini at 2900. Lines 330, 0 and 30 are Aries's
        // 12th, 1st and 2nd. The search runs two houses beyond each end.
        let crossings = path(&[
            (-2_000.0, 270.0, Direct),
            (-1_000.0, 300.0, Direct),
            (100.0, 330.0, Direct),
            (200.0, 330.0, Retrograde),
            (300.0, 330.0, Direct),
            (1_000.0, 0.0, Direct),
            (1_100.0, 0.0, Retrograde),
            (1_200.0, 0.0, Direct),
            (2_000.0, 30.0, Direct),
            (2_900.0, 60.0, Direct),
            (3_800.0, 90.0, Direct),
        ]);
        let searched = Searched::new(&crossings, 0.0);
        let periods = found(&searched, (150.0, 150.0), &[]);
        let one = &periods.sade_sati[0];
        let rising = one.phase(Phase::Rising).unwrap();
        assert_eq!(
            rising.visits,
            [
                Visit {
                    from: Some(at(100.0)),
                    to: Some(at(200.0))
                },
                Visit {
                    from: Some(at(300.0)),
                    to: Some(at(1_000.0))
                },
                Visit {
                    from: Some(at(1_100.0)),
                    to: Some(at(1_200.0))
                },
            ]
        );
        // The day it stood back in Aquarius is in no phase, and still in
        // this Sade Sati's span.
        assert_eq!(one.phase_at(at(250.0)), None);
        assert_eq!(one.begins(), Some(at(100.0)));
        assert_eq!(one.ends(), Some(at(2_900.0)));
        // Asked on the day it stood back in Aquarius, the period is found
        // all the same: it reaches into the window.
        assert_eq!(found(&searched, (250.0, 250.0), &[]).sade_sati.len(), 1);
    }

    #[test]
    fn two_sade_satis_are_a_circuit_apart_and_never_join() {
        // Twelve houses a circuit, 900 days a house, over two circuits.
        let crossings = path(
            &(0..30)
                .map(|k| {
                    (
                        900.0 * f64::from(k),
                        (30.0 * f64::from(k)).rem_euclid(360.0),
                        Direct,
                    )
                })
                .collect::<Vec<_>>(),
        );
        let searched = Searched::new(&crossings, 0.0);
        // From after the Sade Sati Saturn begins in, to before the fourth
        // spell in the 4th.
        let periods = found(&searched, (2_000.0, 900.0 * 26.0), &[4, 8]);
        assert_eq!(periods.sade_sati.len(), 2);
        let starts: Vec<_> = periods.sade_sati.iter().map(SadeSati::begins).collect();
        // The 12th (line 330°) is entered at k = 11 and k = 23.
        assert_eq!(starts, [Some(at(9_900.0)), Some(at(20_700.0))]);
        // The 4th and 8th twice each, in time order.
        let spells: Vec<_> = periods.spells.iter().map(|spell| spell.house).collect();
        assert_eq!(spells, [4, 8, 4, 8]);
    }

    #[test]
    fn a_search_too_narrow_asks_to_widen_on_the_short_side() {
        let crossings = straight();
        // From Aries onward: Saturn is in the 12th when the search begins.
        let searched = Searched::new(&crossings[2..], 0.0);
        assert_eq!(
            periods(&searched, (at(2_000.0), at(2_000.0)), &[]).unwrap(),
            Outcome::Widen {
                before: true,
                after: false
            }
        );
        // One house before the period is not enough; two is.
        let one_back = Searched::new(&crossings[1..], 0.0);
        assert!(matches!(
            periods(&one_back, (at(2_000.0), at(2_000.0)), &[]).unwrap(),
            Outcome::Widen { before: true, .. }
        ));
        assert!(matches!(
            periods(
                &Searched::new(&crossings, 0.0),
                (at(2_000.0), at(2_000.0)),
                &[]
            )
            .unwrap(),
            Outcome::Found(_)
        ));
        // And likewise after: ending in Gemini is one house short.
        assert_eq!(
            periods(
                &Searched::new(&crossings[..5], 0.0),
                (at(2_000.0), at(2_000.0)),
                &[]
            )
            .unwrap(),
            Outcome::Widen {
                before: false,
                after: true
            }
        );
        // An empty search widens both ways.
        assert_eq!(
            periods(&Searched::new(&[], 0.0), (at(0.0), at(0.0)), &[]).unwrap(),
            Outcome::Widen {
                before: true,
                after: true
            }
        );
    }

    #[test]
    fn at_the_coverage_a_bound_is_absent_rather_than_guessed() {
        let crossings = straight();
        let searched = Searched::new(&crossings[2..], 0.0).at_coverage(true, false);
        let periods = found(&searched, (2_000.0, 2_000.0), &[]);
        let one = &periods.sade_sati[0];
        assert_eq!(one.begins(), None);
        assert_eq!(one.phase(Phase::Peak).unwrap().begins(), Some(at(1_800.0)));
        assert!(one.phases[0].contains(at(-1e6)));
    }

    #[test]
    fn a_window_with_no_period_in_it_is_empty() {
        let crossings = straight();
        let searched = Searched::new(&crossings, 0.0);
        // Scorpio, the 8th of Aries.
        let periods = found(&searched, (8_500.0, 8_500.0), &[]);
        assert_eq!(periods, Periods::default());
        assert_eq!(
            found(&searched, (8_500.0, 8_500.0), &[4]),
            Periods::default()
        );
        assert_eq!(
            found(&searched, (8_500.0, 8_500.0), &[4, 8]).spells[0].house,
            8
        );
    }

    #[test]
    fn the_degree_reckoning_moves_the_lattice_to_the_moon() {
        // A Moon at 20° Aries: the houses begin at 5°, so the rising phase
        // is 335° to 5° and the line Saturn enters it by is 335°.
        let origin = Reckoning::Degree.origin_deg(20.0);
        assert!((origin - 5.0).abs() < 1e-12);
        let crossings = path(
            &(0..8)
                .map(|k| {
                    (
                        900.0 * f64::from(k),
                        (275.0 + 30.0 * f64::from(k)).rem_euclid(360.0),
                        Direct,
                    )
                })
                .collect::<Vec<_>>(),
        );
        let searched = Searched::new(&crossings, origin);
        let one = &found(&searched, (3_000.0, 3_000.0), &[]).sade_sati[0];
        // Line 335° is crossed at k = 2.
        assert_eq!(one.begins(), Some(at(1_800.0)));
        assert_eq!(one.ends(), Some(at(4_500.0)));
    }

    #[test]
    fn a_missed_crossing_is_refused_rather_than_answered() {
        let crossings = path(&[(0.0, 0.0, Direct), (900.0, 60.0, Direct)]);
        let error = periods(&Searched::new(&crossings, 0.0), (at(0.0), at(0.0)), &[]).unwrap_err();
        assert_eq!(error.status, teistro_core::error::Status::Internal);
    }

    #[test]
    fn a_smaller_spell_is_a_house_outside_the_sade_satis() {
        for house in [0, 1, 2, 12, 13] {
            let error = check_houses(&[house]).unwrap_err();
            assert_eq!(error.field(), Some("houses"), "{house}");
        }
        assert!(check_houses(&[4, 4]).is_err());
        assert!(check_houses(&[3, 4, 7, 8, 11]).is_ok());
        assert!(check_houses(&DEFAULT_SPELLS).is_ok());
    }

    #[test]
    fn a_phase_is_its_house() {
        for phase in Phase::ALL {
            assert_eq!(Phase::of(phase.house()), Some(phase));
        }
        assert_eq!(Phase::of(4), None);
    }
}
