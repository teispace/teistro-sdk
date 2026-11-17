//! The season: the stretches of the year in which a rite is not held at
//! all (`03-design/muhurta.md` §4.1).
//!
//! A blackout is not a defect weighed against a good day. While Venus is
//! combust no school marries, however good the rest of the day, and a
//! score with enough favourable clauses would hand back that day anyway.
//! So the season is kept apart from the clauses: each blackout a kind and
//! the interval it holds over, found once for a range, and an activity's
//! rules say which kinds it heeds.
//!
//! The intervals are **instants**, not whole days: Pitru paksha begins
//! when the Purnima begins, not at a midnight. A day-level gate reads
//! "any blackout overlaps the day".
//!
//! Everything but asta and the eclipses is the Sun and the Moon: the lunar
//! months from the new moons, their names from the Sun's sign at the
//! opening new moon, their kinds from the sankrantis inside them (the
//! calendar's own rule, `calendar-indian-lunisolar.md` §2), and the tithis
//! that bound Chaturmas and Pitru paksha. Asta is the heliacal events of
//! Jupiter and Venus (`astro::visibility`), paired last-seen to
//! first-seen. The eclipse blackouts read the eclipses the place sees
//! (`muhurta.md` §4.1.1): the eclipse's star for six months after it, and
//! the vedha before it.
//!
//! The Sun's and the Moon's blackouts and an eclipse's vedha are the
//! almanac's (`teistro_panchanga::season` and `teistro_panchanga::vedha`),
//! re-exported here so the search and its callers read one rule.

use teistro_astro::events::{Longitudes, Search, value_of};
use teistro_astro::visibility::{Heliacal, HeliacalEvent, Visibility};
use teistro_calendar::lunisolar::SYNODIC_DAYS;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_panchanga::limb::{Sidereal, Zodiac};
use teistro_port_ephemeris::{Body, EphemerisProvider, Lattice, Quantity};

pub use teistro_panchanga::season::{
    Blackout, BlackoutKind, Month, SAMSARPA_REACH_MONTHS, blackouts, clip_and_order, months,
};
pub use teistro_panchanga::vedha::{
    SeenEclipse, SeenKind, VEDHA_DAYS_AROUND, eclipse_vedha, vedha_over_days, vedha_praharas,
};

/// The asta windows of one body from its heliacal events: each stretch
/// from a last sighting to the next first one, clipped to the range.
///
/// `start` is the body's state on the range's first day; when it was
/// unseen then, it was unseen before the range began too, so the first
/// stretch opens at the range's start. A last sighting with no first
/// after it holds to the range's end.
///
/// Each end is the instant the sighting was read at — the dawn or the
/// dusk the criterion looks at — so a window runs from the last dawn or
/// dusk the body was seen at to the first it is seen at again.
///
/// ```
/// # use teistro_astro::visibility::{HeliacalEvent, HeliacalKind, Motion, Side, Visibility};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_port_ephemeris::Body;
/// #
/// # let state = |at: f64, visible: bool| Visibility {
/// #     body: Body::Venus,
/// #     day_start: JulianDay::literal(at.floor() - 0.5),
/// #     instant: JulianDay::literal(at),
/// #     side: Side::East,
/// #     motion: Motion::Direct,
/// #     measure_deg: 0.0,
/// #     threshold_deg: 0.0,
/// #     visible,
/// #     evaluations: 0,
/// # };
/// # let event = |kind, at| HeliacalEvent {
/// #     kind,
/// #     day: state(at, kind == HeliacalKind::MorningFirst || kind == HeliacalKind::EveningFirst),
/// # };
/// # let events = [
/// #     event(HeliacalKind::EveningFirst, 110.2),
/// #     event(HeliacalKind::EveningLast, 150.8),
/// #     event(HeliacalKind::MorningFirst, 160.2),
/// #     event(HeliacalKind::MorningLast, 250.3),
/// # ];
/// # let range = Interval::literal(100.0, 300.0);
/// use teistro_muhurta::season::{BlackoutKind, asta};
///
/// // Unseen at the start: the first stretch opens at the range's start, and
/// // the last runs to its end.
/// let found = asta(BlackoutKind::ShukraAsta, &state(100.2, false), &events, range);
/// let spans: Vec<(f64, f64)> = found.iter().map(|b| (b.at.from.get(), b.at.to.get())).collect();
/// assert_eq!(spans, [(100.0, 110.2), (150.8, 160.2), (250.3, 300.0)]);
/// // Seen at the start, the first appearance closes nothing.
/// assert_eq!(asta(BlackoutKind::ShukraAsta, &state(100.2, true), &events, range).len(), 2);
/// ```
#[must_use]
pub fn asta(
    kind: BlackoutKind,
    start: &Visibility,
    events: &[HeliacalEvent],
    range: Interval,
) -> Vec<Blackout> {
    let mut found = Vec::new();
    let mut hidden_since = (!start.visible).then_some(range.from.get());
    for event in events {
        let read = event.day.instant.get();
        if event.kind.appears() {
            if let Some(from) = hidden_since.take() {
                if let Ok(found_one) = Blackout::between(kind, from, read) {
                    found.push(found_one);
                }
            }
        } else if hidden_since.is_none() {
            hidden_since = Some(read);
        }
    }
    if let Some(from) = hidden_since {
        if let Ok(found_one) = Blackout::between(kind, from, range.to.get().max(from)) {
            found.push(found_one);
        }
    }
    clip_and_order(found, range)
}

/// Guru or Shukra asta over a range: the body's state on the range's
/// first day and its heliacal events inside it, paired by [`asta`].
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
///
/// # use teistro_astro::visibility::{Criterion, Heliacal};
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_port_ephemeris::Horizon;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, DeltaTModel::TableThenModel);
/// use teistro_muhurta::season::{BlackoutKind, asta_over};
/// use teistro_port_ephemeris::Body;
///
/// // Venus meets the Sun once in October and November 2026, unseen from Kathmandu.
/// let autumn = Interval::literal(2_461_314.5, 2_461_375.5);
/// let found = asta_over(&heliacal, Body::Venus, autumn)?;
/// assert_eq!(found.len(), 1);
/// assert_eq!(found[0].kind, BlackoutKind::ShukraAsta);
/// // Asta is Jupiter's or Venus's alone.
/// assert_eq!(asta_over(&heliacal, Body::Mars, autumn).unwrap_err().field(), Some("body"));
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// A body other than Jupiter and Venus (`INVALID_ARG`), and whatever the
/// visibility reckoner refuses: a day without a dawn or a dusk, the
/// provider's own refusal.
pub fn asta_over<P: EphemerisProvider + ?Sized>(
    heliacal: &Heliacal<'_, P>,
    body: Body,
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let kind = match body {
        Body::Jupiter => BlackoutKind::GuruAsta,
        Body::Venus => BlackoutKind::ShukraAsta,
        other => {
            return Err(Error::invalid_arg(format!(
                "asta is Jupiter's or Venus's, not {other:?}'s"
            ))
            .with_field("body"));
        }
    };
    let from = JulianDay::<Ut1>::literal(range.from.get());
    let to = JulianDay::<Ut1>::literal(range.to.get());
    let start = heliacal.state(body, heliacal.day_start(from))?;
    let events = heliacal.events(body, from, to)?;
    Ok(asta(kind, &start, &events, range))
}

/// How long an eclipse's star stays barred, in synodic months: Raman's
/// six for a marriage (crux C191).
///
/// ```
/// use teistro_muhurta::season::{ECLIPSE_STAR_MONTHS, ECLIPSE_STAR_REACH_DAYS};
///
/// assert_eq!(ECLIPSE_STAR_MONTHS, 6.0);
/// assert!((ECLIPSE_STAR_REACH_DAYS - 177.18).abs() < 0.01);
/// ```
pub const ECLIPSE_STAR_MONTHS: f64 = 6.0;

/// How far before a range an eclipse can still bar a star inside it, days.
///
/// ```
/// use teistro_muhurta::season::ECLIPSE_STAR_REACH_DAYS;
///
/// // Six synodic months, under half a year.
/// assert!(ECLIPSE_STAR_REACH_DAYS < 182.6);
/// ```
pub const ECLIPSE_STAR_REACH_DAYS: f64 = ECLIPSE_STAR_MONTHS * SYNODIC_DAYS;

/// A nakshatra's width, degrees.
const NAKSHATRA_DEG: f64 = 360.0 / 27.0;

/// The stretches the Moon stands in each seen eclipse's star, from the
/// greatest eclipse to [`ECLIPSE_STAR_MONTHS`] after it, clipped to the
/// range (Raman, ch. V, Mahadosha 16; cruxes C189 to C191).
///
/// The star is the Moon's sidereal nakshatra at the greatest eclipse. The
/// passage the eclipse falls in is barred from the eclipse on, and every
/// later passage whole.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_muhurta::season::{BlackoutKind, SeenEclipse, SeenKind, eclipse_stars};
///
/// // The partial lunar eclipse of 28 August 2026: each month after, the Moon
/// // returns to its star for about a day.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 2_461_280.676,
///     seen: Interval::literal(2_461_280.6, 2_461_280.75),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// let autumn = Interval::literal(2_461_284.5, 2_461_375.5);
/// let found = eclipse_stars(&longitudes, zodiac, &[eclipse], autumn)?;
/// assert_eq!(found.len(), 3);
/// assert!(found.iter().all(|b| b.kind == BlackoutKind::EclipseStar && (0.8..1.4).contains(&b.at.days())));
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal.
pub fn eclipse_stars<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    eclipses: &[SeenEclipse],
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let moon = Quantity::Longitude(Body::Moon);
    let mut found = Vec::new();
    for eclipse in eclipses {
        let (from, to) = (eclipse.greatest, eclipse.greatest + ECLIPSE_STAR_REACH_DAYS);
        if to <= range.from.get() || from >= range.to.get() {
            continue;
        }
        let at = value_of(moon, &source, JulianDay::<Ut1>::literal(from))?;
        let star = (at.rem_euclid(360.0) / NAKSHATRA_DEG).floor();
        let crossings = |deg: f64| -> Result<Vec<f64>, Error> {
            Ok(Search::new(&source, moon, Lattice::single(deg))
                .between(
                    JulianDay::<Ut1>::literal(from),
                    JulianDay::<Ut1>::literal(to),
                )?
                .iter()
                .map(|event| event.instant.get())
                .collect())
        };
        let entries = crossings(star * NAKSHATRA_DEG)?;
        let exits = crossings(((star + 1.0) * NAKSHATRA_DEG).rem_euclid(360.0))?;
        for start in core::iter::once(from).chain(entries) {
            let end = exits
                .iter()
                .copied()
                .find(|exit| *exit > start)
                .unwrap_or(to);
            found.push(Blackout::between(BlackoutKind::EclipseStar, start, end)?);
        }
    }
    Ok(clip_and_order(found, range))
}
