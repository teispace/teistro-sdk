//! The four moving limbs of a day, from three crossing searches.
//!
//! A tithi is twelve degrees of the Moon's elongation from the Sun, a
//! karana six, a nakshatra a twenty-seventh of the Moon's own circle and
//! a yoga a twenty-seventh of the Moon's plus the Sun's. `astro::events`
//! already searches a lattice of boundaries over a window, so a limb list
//! is that search, cut into spans, each classified by the integer path
//! (ADR-0016).
//!
//! Three things make it three searches and not four:
//!
//! 1. **A karana is half a tithi**, so the six-degree lattice's crossings
//!    contain the twelve-degree lattice's: one search answers both.
//! 2. **The window is widened** by the longest a member can run, so the
//!    first and last spans carry their own bounds and not the window's,
//!    which is what the corpus loses (`03-design/panchanga-day-conventions.md`
//!    §2).
//! 3. **One source serves all four.** The provider is asked for tropical
//!    longitudes and this module shifts them by the ayanamsha *at each
//!    instant*, so the elongation (where the shift cancels), the Moon's
//!    longitude and the Moon-plus-Sun sum are all read from one place. A
//!    fixed shift would be up to two seconds of time out over a search
//!    window, which is wider than the tolerance the corpus declares.
//!
//! None of the four quantities can go backwards — the slowest elongation
//! is about eleven degrees a day — so the spans are in order and the
//! kernel says so rather than hoping.

use serde::{Deserialize, Serialize};
use teistro_astro::ayanamsha::Basis;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::events::{Longitudes, Search};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Karana, Masa, Nakshatra, Rashi, Tithi, Yoga};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::AyanamshaChoice;
use teistro_port_ephemeris::{Body, Lattice, Quantity};

use crate::span::Span;

/// The longest a member of any limb can run, days.
///
/// The slowest tithi is about twenty-six hours (twelve degrees at the
/// elongation's slowest, near eleven degrees a day) and the slowest
/// nakshatra transit about twenty-eight (a twenty-seventh of the circle
/// at the Moon's slowest). A day and a half covers both with room, and it
/// is what the search window is widened by so that the first and last
/// spans carry their own bounds.
pub const LONGEST_SPAN_DAYS: f64 = 1.5;

/// The four moving limbs of one day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Limbs {
    /// The tithis that touch the day.
    pub tithi: Vec<Span<Tithi>>,
    /// The nakshatras the Moon was in.
    pub nakshatra: Vec<Span<Nakshatra>>,
    /// The nitya yogas.
    pub yoga: Vec<Span<Yoga>>,
    /// The karanas; half-tithis, so there are three or four of them.
    pub karana: Vec<Span<Karana>>,
}

/// Which limb a list belongs to: the lattice it is found on and how a
/// span between two boundaries is named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Limb {
    Tithi,
    Nakshatra,
    Yoga,
    Karana,
}

impl Limb {
    /// The quantity whose crossings bound this limb's members.
    const fn quantity(self) -> Quantity {
        match self {
            Limb::Tithi | Limb::Karana => Quantity::ELONGATION,
            Limb::Nakshatra => Quantity::Longitude(Body::Moon),
            Limb::Yoga => Quantity::MOON_PLUS_SUN,
        }
    }

    /// How many members the quantity's circle holds.
    const fn divisions(self) -> u32 {
        match self {
            Limb::Tithi => 30,
            Limb::Nakshatra | Limb::Yoga => 27,
            // Sixty half-tithis make a lunar month; which karana each is
            // comes from `karana_of`.
            Limb::Karana => 60,
        }
    }

    /// The lattice its boundaries lie on.
    fn lattice(self) -> Lattice {
        match self {
            Limb::Tithi => Lattice::TITHIS,
            Limb::Nakshatra => Lattice::NAKSHATRAS,
            Limb::Yoga => Lattice::YOGAS,
            Limb::Karana => Lattice::KARANAS,
        }
    }
}

/// The karana that a half-tithi of the lunar month is.
///
/// Sixty of them make a month and they are not a cycle of eleven:
/// Kimstughna opens the month, Shakuni, Chatushpada and Naga close it,
/// and the seven movable karanas repeat through everything between. The
/// corpus's own successor relation holds over all 109 consecutive pairs
/// it records (`03-design/panchanga-day-conventions.md` §2).
#[must_use]
pub fn karana_of(half_tithi: u32) -> Karana {
    match half_tithi % 60 {
        0 => Karana::Kimstughna,
        57 => Karana::Shakuni,
        58 => Karana::Chatushpada,
        59 => Karana::Naga,
        // The seven movable karanas, Bava first, from the second half of
        // the first tithi.
        other => {
            Karana::from_id(u16::try_from((other - 1) % 7).unwrap_or(0)).unwrap_or(Karana::Bava)
        }
    }
}

/// A source of longitudes in the chart's own zodiac.
///
/// The provider is asked for tropical positions and shifted here, because
/// a chart holds one ayanamsha and the provider's may not be it
/// (`03-design/chart-foundation.md` §4). The shift is evaluated at every
/// instant rather than once for the window: the ayanamsha moves about
/// 0.00014 degrees a day, which over a four-day search is two seconds of
/// the Moon's time — wider than the tolerance the corpus declares.
struct Sidereal<'a, S: Longitudes + ?Sized> {
    tropical: &'a S,
    ayanamsha: Option<AyanamshaChoice>,
    basis: Basis,
    precession: PrecessionModel,
    delta_t: DeltaTModel,
}

impl<S: Longitudes + ?Sized> Sidereal<'_, S> {
    /// The ayanamsha at an instant, degrees, and its rate, degrees a day.
    fn offset(&self, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        let Some(choice) = self.ayanamsha else {
            return Ok((0.0, 0.0));
        };
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let value = teistro_astro::ayanamsha::value_deg(
            &choice,
            tt,
            self.basis,
            self.precession,
            self.delta_t,
        )?;
        // A day on either side gives the rate by a central difference;
        // the ayanamsha is a smooth function of the instant and the rate
        // only corrects a speed nothing classifies with.
        let step = 1.0;
        let ahead = teistro_astro::ayanamsha::value_deg(
            &choice,
            JulianDay::literal(tt.get() + step),
            self.basis,
            self.precession,
            self.delta_t,
        )?;
        let behind = teistro_astro::ayanamsha::value_deg(
            &choice,
            JulianDay::literal(tt.get() - step),
            self.basis,
            self.precession,
            self.delta_t,
        )?;
        Ok((value, (ahead - behind) / (2.0 * step)))
    }
}

impl<S: Longitudes + ?Sized> Longitudes for Sidereal<'_, S> {
    fn longitude_and_speed(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<(f64, f64), Error> {
        let (longitude, speed) = self.tropical.longitude_and_speed(body, ut1)?;
        let (offset, rate) = self.offset(ut1)?;
        Ok(((longitude - offset).rem_euclid(360.0), speed - rate))
    }

    fn longitude_and_speed_pair(
        &self,
        bodies: [Body; 2],
        ut1: JulianDay<Ut1>,
    ) -> Result<[(f64, f64); 2], Error> {
        let pair = self.tropical.longitude_and_speed_pair(bodies, ut1)?;
        let (offset, rate) = self.offset(ut1)?;
        Ok(pair.map(|(longitude, speed)| ((longitude - offset).rem_euclid(360.0), speed - rate)))
    }

    fn describe(&self) -> String {
        match self.ayanamsha {
            Some(choice) => format!("{} shifted by {choice:?}", self.tropical.describe()),
            None => self.tropical.describe(),
        }
    }
}

/// Everything the kernel needs beyond a source of longitudes.
#[derive(Clone, Copy, Debug)]
pub struct Zodiac {
    /// The ayanamsha the day's limbs are measured from, or `None` for a
    /// tropical reading.
    pub ayanamsha: Option<AyanamshaChoice>,
    /// Whether that ayanamsha carries the nutation.
    pub basis: Basis,
    /// The precession model behind it.
    pub precession: PrecessionModel,
    /// The Delta T model.
    pub delta_t: DeltaTModel,
}

/// The four limbs of a window.
///
/// # Errors
///
/// The source's own refusal (an instant or a body it cannot answer for),
/// an ayanamsha the catalogue cannot evaluate at the instant, or a
/// quantity that moved backwards, which none of these four can and which
/// is reported as `INTERNAL` rather than silently reordered.
pub fn limbs<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Limbs, Error> {
    let source = Sidereal {
        tropical,
        ayanamsha: zodiac.ayanamsha,
        basis: zodiac.basis,
        precession: zodiac.precession,
        delta_t: zodiac.delta_t,
    };
    // One search at six degrees of the elongation answers the tithis and
    // the karanas alike, a tithi being two karanas: its boundaries are
    // the karana boundaries that fall on a multiple of twelve degrees.
    let elongation = boundaries(&source, Limb::Karana, window)?;
    let tithi_edges: Vec<f64> = elongation
        .iter()
        .filter(|(_, boundary)| is_tithi_boundary(*boundary))
        .map(|(instant, _)| *instant)
        .collect();
    let karana_edges: Vec<f64> = elongation.iter().map(|(instant, _)| *instant).collect();
    Ok(Limbs {
        tithi: spans(&source, Limb::Tithi, window, &tithi_edges)?,
        karana: spans(&source, Limb::Karana, window, &karana_edges)?,
        nakshatra: spans(
            &source,
            Limb::Nakshatra,
            window,
            &crossings(&source, Limb::Nakshatra, window)?,
        )?,
        yoga: spans(
            &source,
            Limb::Yoga,
            window,
            &crossings(&source, Limb::Yoga, window)?,
        )?,
    })
}

/// Whether a six-degree boundary of the elongation is also a tithi's.
///
/// The lattice's own degrees are exact multiples of six by construction,
/// so this is an integer question asked of a double; the tolerance is a
/// millionth of a degree, which is a tenth of a second of the Moon's
/// time and far below any spacing it could confuse.
fn is_tithi_boundary(boundary_deg: f64) -> bool {
    let twelfths = boundary_deg / 12.0;
    (twelfths - twelfths.round()).abs() < 1e-6
}

/// Just the instants of a limb's crossings.
fn crossings<S: Longitudes + ?Sized>(
    source: &S,
    limb: Limb,
    window: Interval,
) -> Result<Vec<f64>, Error> {
    Ok(boundaries(source, limb, window)?
        .into_iter()
        .map(|(instant, _)| instant)
        .collect())
}

/// The crossings of a limb's lattice inside the widened window, each with
/// the boundary it reached, in order.
fn boundaries<S: Longitudes + ?Sized>(
    source: &S,
    limb: Limb,
    window: Interval,
) -> Result<Vec<(f64, f64)>, Error> {
    let from = JulianDay::<Ut1>::literal(window.from.get() - LONGEST_SPAN_DAYS);
    let to = JulianDay::<Ut1>::literal(window.to.get() + LONGEST_SPAN_DAYS);
    let events = Search::new(source, limb.quantity(), limb.lattice()).between(from, to)?;
    let found: Vec<(f64, f64)> = events
        .iter()
        .map(|event| (event.instant.get(), event.boundary_deg))
        .collect();
    // None of the four quantities can go backwards — the slowest
    // elongation is about eleven degrees a day — so an out-of-order
    // boundary is a defect in the search and not a retrograde arc.
    if found.windows(2).any(|pair| {
        pair.first()
            .zip(pair.get(1))
            .is_some_and(|(a, b)| b.0 < a.0)
    }) {
        return Err(Error::new(
            Status::Internal,
            format!("{limb:?} boundaries came back out of order"),
        ));
    }
    Ok(found)
}

/// A limb's spans over a window, from the boundaries already found.
fn spans<S, T>(
    source: &S,
    limb: Limb,
    window: Interval,
    found: &[f64],
) -> Result<Vec<Span<T>>, Error>
where
    S: Longitudes + ?Sized,
    T: Member,
{
    let mut out = Vec::new();
    let edges = edges(found, window);
    for pair in edges.windows(2) {
        let (Some(from), Some(to)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let whole = Interval::literal(*from, *to);
        if !whole.overlaps(window) {
            continue;
        }
        // The member is read in the middle of its own span, where no
        // boundary is, so a solver's last bit cannot change the answer.
        let index = classify(source, limb, whole.days().mul_add(0.5, *from))?;
        if let Some(span) = Span::new(T::of(index), whole, window) {
            out.push(span);
        }
    }
    Ok(out)
}

/// The instants that bound the spans touching a window: the boundaries
/// inside the widened search, with the widened ends closing the first and
/// last spans.
fn edges(boundaries: &[f64], window: Interval) -> Vec<f64> {
    let first = window.from.get() - LONGEST_SPAN_DAYS;
    let last = window.to.get() + LONGEST_SPAN_DAYS;
    let mut edges = Vec::with_capacity(boundaries.len() + 2);
    edges.push(first);
    edges.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > first && *at < last),
    );
    edges.push(last);
    edges
}

/// Which member of the limb the quantity names at an instant, by the
/// integer path (ADR-0016).
fn classify<S: Longitudes + ?Sized>(source: &S, limb: Limb, jd: f64) -> Result<u32, Error> {
    let ut1 = JulianDay::<Ut1>::literal(jd);
    let degrees = teistro_astro::events::value_of(limb.quantity(), source, ut1)?;
    Ok(Nas::try_from_degrees(degrees)?.division_index(limb.divisions()))
}

/// The signs a body stood in over a window, as spans.
///
/// The Sun's sign gives the ayana and its change is a sankranti; the
/// Moon's is what an almanac prints beside the nakshatra. Both are the
/// same search on the twelve-sign lattice over the same shifted source
/// the limbs use, so the sign a span names is in the chart's zodiac like
/// everything else.
///
/// # Errors
///
/// As [`limbs`].
pub fn signs<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Vec<Span<Rashi>>, Error> {
    let source = Sidereal {
        tropical,
        ayanamsha: zodiac.ayanamsha,
        basis: zodiac.basis,
        precession: zodiac.precession,
        delta_t: zodiac.delta_t,
    };
    let quantity = Quantity::Longitude(body);
    let from = JulianDay::<Ut1>::literal(window.from.get() - SIGN_SEARCH_DAYS);
    let to = JulianDay::<Ut1>::literal(window.to.get() + SIGN_SEARCH_DAYS);
    let events = Search::new(&source, quantity, Lattice::SIGNS).between(from, to)?;
    let found: Vec<f64> = events.iter().map(|event| event.instant.get()).collect();
    let mut out = Vec::new();
    let edges = sign_edges(&found, window);
    for pair in edges.windows(2) {
        let (Some(start), Some(end)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let whole = Interval::literal(*start, *end);
        if !whole.overlaps(window) {
            continue;
        }
        let middle = JulianDay::<Ut1>::literal(whole.days().mul_add(0.5, *start));
        let degrees = teistro_astro::events::value_of(quantity, &source, middle)?;
        let sign = Nas::try_from_degrees(degrees)?.sign();
        if let Some(span) = Span::new(sign, whole, window) {
            out.push(span);
        }
    }
    Ok(out)
}

/// How far either side of a window a sign search reaches.
///
/// The Sun takes a month to cross a sign and the Moon two and a half
/// days, so a body's sign can begin well before the window and end well
/// after it. Forty days covers the Sun; beyond that the span's own bounds
/// are reported as the search's edge, which is honest rather than wrong,
/// and no caller of a daily almanac needs a solar sign's exact start.
const SIGN_SEARCH_DAYS: f64 = 40.0;

/// The instants bounding the sign spans that touch a window.
fn sign_edges(boundaries: &[f64], window: Interval) -> Vec<f64> {
    let first = window.from.get() - SIGN_SEARCH_DAYS;
    let last = window.to.get() + SIGN_SEARCH_DAYS;
    let mut edges = Vec::with_capacity(boundaries.len() + 2);
    edges.push(first);
    edges.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > first && *at < last),
    );
    edges.push(last);
    edges
}

/// The amanta month a window falls in.
///
/// A lunar month takes the name of the solar month its new moon fell in:
/// Chaitra when the Sun stood in Pisces at that new moon, Vaishakha in
/// Aries, and so on, which the catalogue carries as each masa's
/// `solar_sign`. So this is one crossing search — the elongation through
/// zero, backwards from the window — and one sign.
///
/// What it does **not** decide is whether the month is intercalary. When
/// two new moons fall in one solar month both take that month's name and
/// the second is adhika; naming them is right either way, and marking
/// which is the Indian lunisolar calendar's to do.
///
/// # Errors
///
/// The source's own refusal, or a window with no new moon in the
/// thirty-one days before it, which cannot happen for a real sky and is
/// reported as `NOT_CONVERGED` naming the window.
pub fn amanta_month<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Masa, Error> {
    let source = Sidereal {
        tropical,
        ayanamsha: zodiac.ayanamsha,
        basis: zodiac.basis,
        precession: zodiac.precession,
        delta_t: zodiac.delta_t,
    };
    let from = JulianDay::<Ut1>::literal(window.from.get() - SYNODIC_SEARCH_DAYS);
    let to = JulianDay::<Ut1>::literal(window.from.get());
    let events =
        Search::new(&source, Quantity::ELONGATION, Lattice::single(0.0)).between(from, to)?;
    let new_moon = events.last().map(|event| event.instant).ok_or_else(|| {
        Error::new(
            Status::NotConverged,
            format!(
                "no new moon in the {SYNODIC_SEARCH_DAYS} days before {}",
                window.from
            ),
        )
    })?;
    let sun = teistro_astro::events::value_of(Quantity::Longitude(Body::Sun), &source, new_moon)?;
    let sign = Nas::try_from_degrees(sun)?.sign();
    Masa::ALL
        .iter()
        .copied()
        .find(|masa| masa.attributes().solar_sign == sign)
        .ok_or_else(|| Error::internal(format!("no lunar month for the Sun in {sign:?}")))
}

/// How far back a new moon is looked for: a synodic month is 29.53 days,
/// so thirty-one always holds one.
const SYNODIC_SEARCH_DAYS: f64 = 31.0;

/// A limb's member, named by its index around the circle.
pub trait Member: Sized {
    /// The member an index names; an index the catalogue does not have is
    /// impossible by construction, and the first member stands in.
    fn of(index: u32) -> Self;
}

impl Member for Tithi {
    fn of(index: u32) -> Tithi {
        Tithi::from_id(u16::try_from(index % 30).unwrap_or(0)).unwrap_or(Tithi::ShuklaPratipada)
    }
}

impl Member for Nakshatra {
    fn of(index: u32) -> Nakshatra {
        Nakshatra::from_id(u16::try_from(index % 27).unwrap_or(0)).unwrap_or(Nakshatra::Ashwini)
    }
}

impl Member for Yoga {
    fn of(index: u32) -> Yoga {
        Yoga::from_id(u16::try_from(index % 27).unwrap_or(0)).unwrap_or(Yoga::Vishkambha)
    }
}

impl Member for Karana {
    fn of(index: u32) -> Karana {
        karana_of(index)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{LONGEST_SPAN_DAYS, Limb, edges, karana_of};
    use teistro_core::catalogue::Karana;
    use teistro_core::interval::Interval;

    #[test]
    fn the_month_opens_with_kimstughna_and_closes_with_naga() {
        assert_eq!(karana_of(0), Karana::Kimstughna);
        assert_eq!(karana_of(1), Karana::Bava, "the first movable one");
        assert_eq!(karana_of(7), Karana::Vishti, "and the seventh");
        assert_eq!(karana_of(8), Karana::Bava, "and round again");
        assert_eq!(karana_of(56), Karana::Vishti);
        assert_eq!(karana_of(57), Karana::Shakuni);
        assert_eq!(karana_of(58), Karana::Chatushpada);
        assert_eq!(karana_of(59), Karana::Naga);
        assert_eq!(karana_of(60), Karana::Kimstughna, "and the month turns");
    }

    #[test]
    fn the_seven_movable_karanas_repeat_eight_times() {
        let movable: Vec<Karana> = (1..57).map(karana_of).collect();
        assert_eq!(movable.len(), 56, "eight rounds of seven");
        assert!(
            movable.iter().all(|karana| karana.attributes().movable),
            "every one of them is movable"
        );
        for round in 0..8 {
            assert_eq!(movable[round * 7], Karana::Bava);
            assert_eq!(movable[round * 7 + 6], Karana::Vishti);
        }
    }

    #[test]
    fn the_fixed_karanas_are_the_ones_that_are_not_movable() {
        for fixed in [
            Karana::Kimstughna,
            Karana::Shakuni,
            Karana::Chatushpada,
            Karana::Naga,
        ] {
            assert!(!fixed.attributes().movable);
        }
    }

    #[test]
    fn the_edges_close_the_first_and_last_spans_outside_the_window() {
        let window = Interval::literal(100.0, 101.0);
        // Two boundaries inside the day and one outside the widened search.
        let found = [99.5, 100.4, 100.9, 200.0];
        let edges = edges(&found, window);
        assert_eq!(edges.first().copied(), Some(100.0 - LONGEST_SPAN_DAYS));
        assert_eq!(edges.last().copied(), Some(101.0 + LONGEST_SPAN_DAYS));
        assert!(edges.contains(&99.5), "a boundary before the day is kept");
        assert!(!edges.contains(&200.0), "one outside the search is not");
        // Every consecutive pair is a span, and they are in order.
        assert!(edges.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn each_limb_names_its_own_lattice_and_divisions() {
        assert_eq!(Limb::Tithi.divisions(), 30);
        assert_eq!(Limb::Karana.divisions(), 60);
        assert_eq!(Limb::Nakshatra.divisions(), 27);
        assert_eq!(Limb::Yoga.divisions(), 27);
        assert!((Limb::Tithi.lattice().step_deg - 12.0).abs() < f64::EPSILON);
        assert!((Limb::Karana.lattice().step_deg - 6.0).abs() < f64::EPSILON);
    }
}
