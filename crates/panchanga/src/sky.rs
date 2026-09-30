//! What the Moon and the Sun did in the day: rise and set, the sign each
//! stood in, and when either left it.
//!
//! The Moon's rise and set are where the recording engine keeps two
//! windows in one section. Every other field of its `panchanga_day` is
//! bounded by sunrise; the moonrise and moonset are the first at or after
//! the local civil midnight, so 24 of the 108 recorded events fall
//! outside the day the limbs occupy
//! (`03-design/panchanga-day-conventions.md` §9). That is a defect and
//! not a convention — a reader cannot tell which window a row belongs to
//! — so the SDK uses the day's own window and `panchanga.moon_events`
//! carries the engine's reading for the profile that reproduces it.
//!
//! Under the day's window there may be **none, one or two** of each: a
//! moonrise and the next are about 24 h 50 m apart, so a 24-hour window
//! sometimes holds two and sometimes none. The lists say so; a single
//! nullable field would not.

use serde::{Deserialize, Serialize};
use teistro_calendar::solar::{MonthStartRule, SolarModel, rule};
use teistro_calendar::{BikramSambat, CalendarSystem, FixedDay};
use teistro_core::catalogue::{Ayana, Masa, Rashi, Ritu};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::SolarMonthStart;
use teistro_core::time::LocalClock;

use crate::span::Span;

/// The first sidereal sign of uttarayana, Capricorn: the Sun's northward
/// half runs from Makara sankranti to the end of Gemini.
const FIRST_UTTARAYANA_SIGN: u16 = 9;
/// The last, Gemini.
const LAST_UTTARAYANA_SIGN: u16 = 2;

/// What the Moon did in the day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MoonDay {
    /// Every moonrise inside the window, in order.
    pub rises: Vec<JulianDay<Utc>>,
    /// Every moonset inside the window.
    pub sets: Vec<JulianDay<Utc>>,
    /// The signs the Moon stood in, with when it entered and left each.
    pub signs: Vec<Span<Rashi>>,
    /// Which window the rises and sets were found in, for the stamp.
    pub window: Interval,
}

impl MoonDay {
    /// The sign the Moon was in when the day opened.
    #[must_use]
    pub fn sign(&self) -> Option<Rashi> {
        self.signs.first().map(|span| span.member)
    }

    /// When the Moon changed sign inside the day, if it did.
    #[must_use]
    pub fn changed_sign_at(&self) -> Option<JulianDay<Utc>> {
        self.signs.get(1).map(|span| span.whole.from)
    }
}

/// What the Sun did in the day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SunDay {
    /// The signs the Sun stood in; two only on a sankranti day.
    pub signs: Vec<Span<Rashi>>,
    /// Which half of the year the day falls in.
    pub ayana: Ayana,
    /// Which season the day falls in, under `panchanga.ritu`.
    pub ritu: Ritu,
    /// The sankranti inside the day, when the Sun entered a new sign.
    pub sankranti: Option<JulianDay<Utc>>,
}

impl SunDay {
    /// The sign the Sun was in when the day opened.
    #[must_use]
    pub fn sign(&self) -> Option<Rashi> {
        self.signs.first().map(|span| span.member)
    }
}

/// The half of the year a sidereal sign falls in.
///
/// Uttarayana runs while the sidereal Sun is in Capricorn to Gemini —
/// from Makara sankranti — which the corpus bears out on all 55 recorded
/// days (`03-design/panchanga-day-conventions.md` §9).
#[must_use]
pub fn ayana_of(sign: Rashi) -> Ayana {
    let index = sign.id();
    if index >= FIRST_UTTARAYANA_SIGN || index <= LAST_UTTARAYANA_SIGN {
        Ayana::Uttarayana
    } else {
        Ayana::Dakshinayana
    }
}

/// The season of the solar month a sign names: two signs each from
/// Capricorn (Surya Siddhanta XIV.10), read off the catalogue's months,
/// which pair each sign with its season.
#[must_use]
pub fn ritu_of(sign: Rashi) -> Ritu {
    Masa::ALL
        .iter()
        .map(|masa| masa.attributes())
        .find(|month| month.solar_sign == sign)
        .map_or(Ritu::Vasanta, |month| month.ritu)
}

/// The Sun's day from the signs it stood in and the season the almanac
/// reckoned under `panchanga.ritu`.
#[must_use]
pub fn sun_day(signs: Vec<Span<Rashi>>, ritu: Ritu) -> SunDay {
    let ayana = signs
        .first()
        .map_or(Ayana::Uttarayana, |span| ayana_of(span.member));
    let sankranti = signs.get(1).map(|span| span.whole.from);
    SunDay {
        signs,
        ayana,
        ritu,
        sankranti,
    }
}

/// The days either side of its own civil day that any named rule moves a
/// sankranti: the following day, or the day before for a Karka sankranti
/// before dawn. One further than a sankranti can be placed, so a sankranti
/// this far from the day is settled without asking the rule.
const RULE_REACH_DAYS: i64 = 2;

/// The sidereal solar month a civil day belongs to, under
/// `panchanga.solar_month_start`.
///
/// Under a rule, the last sign whose sankranti the rule places on the day
/// or before it. The spans are the Sun's over the day's window, each
/// carrying its whole sign; the sankrantis they bound are the only ones a
/// rule can place on this day, since none moves one by more than a day.
/// `None` only for no spans at all. Under `BIKRAM_SAMBAT`, the month the
/// shipped calendar gives the day, whose sankrantis are the Surya
/// Siddhanta's and not the spans'.
///
/// # Errors
///
/// A rule's refusal (a sunrise it needs on a day without one), a day
/// outside the Bikram Sambat table, or a value this build does not know.
pub fn solar_month(
    start: SolarMonthStart,
    date: FixedDay,
    signs: &[Span<Rashi>],
    (place, clock, model): (&Place, &dyn LocalClock, &dyn SolarModel),
) -> Result<Option<Rashi>, Error> {
    let rule = match start {
        SolarMonthStart::Punyakala => MonthStartRule::Punyakala,
        SolarMonthStart::SankrantiDay => MonthStartRule::SankrantiDay,
        SolarMonthStart::FollowingDay => MonthStartRule::FollowingDay,
        SolarMonthStart::SunriseToSunrise => MonthStartRule::SunriseToSunrise,
        SolarMonthStart::BeforeSunset => MonthStartRule::BeforeSunset,
        SolarMonthStart::BeforeAparahna => MonthStartRule::BeforeAparahna,
        SolarMonthStart::BikramSambat => return bikram_sambat_month(date).map(Some),
        other => {
            return Err(Error::unsupported(format!(
                "the panchanga does not know the panchanga.solar_month_start value {other:?} yet"
            ))
            .with_field("panchanga.solar_month_start"));
        }
    };
    let (Some(first), Some(last)) = (signs.first(), signs.last()) else {
        return Ok(None);
    };
    let placed = |sign: Rashi, sankranti: JulianDay<Utc>| -> Result<bool, Error> {
        let near = date.days_until(rule::local_day(clock, sankranti).0);
        if near <= -RULE_REACH_DAYS {
            return Ok(true);
        }
        if near >= RULE_REACH_DAYS {
            return Ok(false);
        }
        let index = u8::try_from(sign.id()).unwrap_or(0);
        Ok(rule.month_start(index, sankranti, clock, model, place)? <= date)
    };
    let mut month = previous(first.member);
    for span in signs {
        if placed(span.member, span.whole.from)? {
            month = span.member;
        }
    }
    let after = next(last.member);
    if placed(after, last.whole.to)? {
        month = after;
    }
    Ok(Some(month))
}

/// The sign whose month the shipped Bikram Sambat calendar gives a day:
/// Baisakh is Mesha's.
fn bikram_sambat_month(date: FixedDay) -> Result<Rashi, Error> {
    let bs = BikramSambat::shipped().date_of(date).map_err(|error| {
        error
            .with_field("panchanga.solar_month_start")
            .with_hint("the Bikram Sambat table ends; PUNYAKALA places the months by rule")
    })?;
    Rashi::from_id(u16::from(bs.month.saturating_sub(1))).ok_or_else(|| {
        Error::internal(format!(
            "the Bikram Sambat calendar gave month {} of twelve",
            bs.month
        ))
    })
}

fn next(sign: Rashi) -> Rashi {
    Rashi::from_id((sign.id() + 1) % 12).unwrap_or(sign)
}

fn previous(sign: Rashi) -> Rashi {
    Rashi::from_id((sign.id() + 11) % 12).unwrap_or(sign)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{MoonDay, ayana_of, ritu_of, solar_month, sun_day};
    use teistro_calendar::FixedDay;
    use teistro_calendar::solar::{DayArc, DayLight, SolarModel};
    use teistro_core::catalogue::{Ayana, Rashi, Ritu};
    use teistro_core::error::Error;
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    use teistro_core::settings::SolarMonthStart;
    use teistro_core::time::UtcOffset;

    use crate::span::Span;

    fn window() -> Interval {
        Interval::literal(100.0, 101.0)
    }

    fn signs(members: &[(Rashi, f64, f64)]) -> Vec<Span<Rashi>> {
        members
            .iter()
            .filter_map(|(sign, from, to)| {
                Span::new(*sign, Interval::literal(*from, *to), window())
            })
            .collect()
    }

    /// The Surya Siddhanta's seasons (XIV.10), stated rather than read
    /// off the table the function reads: two signs each, from Capricorn.
    #[test]
    fn the_seasons_run_two_signs_each_from_capricorn() {
        let pairs = [
            (Ritu::Shishira, [Rashi::Capricorn, Rashi::Aquarius]),
            (Ritu::Vasanta, [Rashi::Pisces, Rashi::Aries]),
            (Ritu::Grishma, [Rashi::Taurus, Rashi::Gemini]),
            (Ritu::Varsha, [Rashi::Cancer, Rashi::Leo]),
            (Ritu::Sharad, [Rashi::Virgo, Rashi::Libra]),
            (Ritu::Hemanta, [Rashi::Scorpio, Rashi::Sagittarius]),
        ];
        for (ritu, signs) in pairs {
            for sign in signs {
                assert_eq!(ritu_of(sign), ritu, "{sign:?}");
            }
        }
    }

    #[test]
    fn uttarayana_runs_from_capricorn_to_gemini() {
        for sign in [
            Rashi::Capricorn,
            Rashi::Aquarius,
            Rashi::Pisces,
            Rashi::Aries,
            Rashi::Taurus,
            Rashi::Gemini,
        ] {
            assert_eq!(ayana_of(sign), Ayana::Uttarayana, "{sign:?}");
        }
        for sign in [
            Rashi::Cancer,
            Rashi::Leo,
            Rashi::Virgo,
            Rashi::Libra,
            Rashi::Scorpio,
            Rashi::Sagittarius,
        ] {
            assert_eq!(ayana_of(sign), Ayana::Dakshinayana, "{sign:?}");
        }
        // Six each, and every sign in one of them.
        assert_eq!(
            Rashi::ALL
                .iter()
                .filter(|sign| ayana_of(**sign) == Ayana::Uttarayana)
                .count(),
            6
        );
    }

    #[test]
    fn a_sankranti_is_the_second_sign_beginning() {
        let ordinary = sun_day(signs(&[(Rashi::Leo, 99.0, 130.0)]), Ritu::Varsha);
        assert_eq!(ordinary.sign(), Some(Rashi::Leo));
        assert_eq!(ordinary.ayana, Ayana::Dakshinayana);
        assert!(ordinary.sankranti.is_none());

        let turning = sun_day(
            signs(&[
                (Rashi::Sagittarius, 70.0, 100.4),
                (Rashi::Capricorn, 100.4, 130.0),
            ]),
            Ritu::Hemanta,
        );
        assert_eq!(turning.sign(), Some(Rashi::Sagittarius));
        assert_eq!(
            turning.ayana,
            Ayana::Dakshinayana,
            "the ayana is the sign the day opened in"
        );
        assert_eq!(
            turning
                .sankranti
                .map(teistro_core::quantity::JulianDay::get),
            Some(100.4)
        );
    }

    /// A Sun that rises at 06:00 and sets at 18:00 UTC every day.
    struct SixToSix;

    impl SolarModel for SixToSix {
        fn sidereal_sun_deg(&self, _jd_ut: f64) -> Result<f64, Error> {
            Ok(0.0)
        }

        fn day_light(&self, day: FixedDay, _place: &Place) -> Result<DayLight, Error> {
            let midnight = day.jd_at_midnight()?;
            Ok(DayLight::Arc(DayArc {
                sunrise: midnight.plus_days(0.25)?,
                sunset: midnight.plus_days(0.75)?,
            }))
        }

        fn describe(&self) -> String {
            String::from("six to six")
        }

        fn convention(&self) -> teistro_core::settings::SunriseConvention {
            teistro_core::settings::Sunrise::CentreNoRefraction.into()
        }
    }

    const DAY: FixedDay = FixedDay::new(739_700);

    fn at(day: FixedDay, hours: f64) -> JulianDay<Utc> {
        day.jd_at_midnight()
            .unwrap()
            .plus_days(hours / 24.0)
            .unwrap()
    }

    /// The month `DAY` belongs to when the Sun leaves `from` for the next
    /// sign `hours` after its midnight, over the sunrise-to-sunrise window.
    fn month(start: SolarMonthStart, from: Rashi, hours: f64) -> Rashi {
        let window = Interval::new(at(DAY, 6.0), at(DAY, 30.0)).unwrap();
        let sankranti = at(DAY, hours);
        let into = Rashi::from_id((from.id() + 1) % 12).unwrap();
        let spans: Vec<Span<Rashi>> = [
            (from, at(DAY, -700.0), sankranti),
            (into, sankranti, at(DAY, 700.0)),
        ]
        .into_iter()
        .filter_map(|(sign, a, b)| Span::new(sign, Interval::new(a, b).unwrap(), window))
        .collect();
        let place = Place::new(
            Latitude::literal(0.0),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        );
        let clock = UtcOffset::literal(0, 0, 0);
        solar_month(start, DAY, &spans, (&place, &clock, &SixToSix))
            .unwrap()
            .unwrap()
    }

    /// Nepal's 15 May 2026: a sankranti half an hour after sunrise begins
    /// the month that day, where the Sun's sign at sunrise says the next.
    #[test]
    fn a_daytime_sankranti_begins_the_month_on_its_own_day() {
        assert_eq!(
            month(SolarMonthStart::Punyakala, Rashi::Aries, 6.5),
            Rashi::Taurus
        );
        assert_eq!(
            month(SolarMonthStart::FollowingDay, Rashi::Aries, 6.5),
            Rashi::Aries
        );
        assert_eq!(
            month(SolarMonthStart::SunriseToSunrise, Rashi::Aries, 6.5),
            Rashi::Taurus
        );
    }

    /// The punya-kala's two exceptions, each against the civil day.
    #[test]
    fn karka_before_dawn_and_makara_after_sunset_move() {
        // Karka at 03:00 of the next civil day belongs to this one.
        assert_eq!(
            month(SolarMonthStart::Punyakala, Rashi::Gemini, 27.0),
            Rashi::Cancer
        );
        assert_eq!(
            month(SolarMonthStart::SankrantiDay, Rashi::Gemini, 27.0),
            Rashi::Gemini
        );
        // Makara at 20:00 belongs to the next day; Kumbha at 20:00 does
        // not.
        assert_eq!(
            month(SolarMonthStart::Punyakala, Rashi::Sagittarius, 20.0),
            Rashi::Sagittarius
        );
        assert_eq!(
            month(SolarMonthStart::Punyakala, Rashi::Capricorn, 20.0),
            Rashi::Aquarius
        );
    }

    /// Nepal's calendar names the month of its own table, whatever the
    /// spans say: 16 July 2026 is Asar 32, Gemini's month, though the
    /// modern Sun entered Cancer before its next dawn.
    #[test]
    fn the_bikram_sambat_row_reads_the_calendar() {
        use teistro_calendar::gregorian::fixed_from_gregorian;
        let month_of = |(y, m, d)| {
            solar_month(
                SolarMonthStart::BikramSambat,
                fixed_from_gregorian(y, m, d),
                &[],
                (
                    &Place::new(
                        Latitude::literal(27.7172),
                        Longitude::literal(85.324),
                        Altitude::literal(1400.0),
                    ),
                    &UtcOffset::literal(5, 45, 0),
                    &SixToSix,
                ),
            )
            .unwrap()
        };
        assert_eq!(month_of((2026, 7, 16)), Some(Rashi::Gemini));
        assert_eq!(month_of((2026, 7, 17)), Some(Rashi::Cancer));
        assert_eq!(month_of((2026, 4, 14)), Some(Rashi::Aries));
        // Outside the table the row refuses and names the knob.
        let place = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let error = solar_month(
            SolarMonthStart::BikramSambat,
            fixed_from_gregorian(3000, 1, 1),
            &[],
            (&place, &UtcOffset::literal(5, 45, 0), &SixToSix),
        )
        .unwrap_err();
        assert_eq!(error.field(), Some("panchanga.solar_month_start"));
    }

    /// A day far from any sankranti is its sign's, whatever the rule, and
    /// a sankranti two days off is settled without asking it.
    #[test]
    fn a_month_far_from_its_edges_is_its_sign() {
        for start in [
            SolarMonthStart::Punyakala,
            SolarMonthStart::SankrantiDay,
            SolarMonthStart::FollowingDay,
            SolarMonthStart::SunriseToSunrise,
            SolarMonthStart::BeforeSunset,
            SolarMonthStart::BeforeAparahna,
        ] {
            assert_eq!(month(start, Rashi::Leo, -48.0), Rashi::Virgo, "{start:?}");
            assert_eq!(month(start, Rashi::Leo, 60.0), Rashi::Leo, "{start:?}");
        }
    }

    #[test]
    fn the_moon_reports_what_it_did_and_no_more() {
        let moon = MoonDay {
            rises: Vec::new(),
            sets: Vec::new(),
            signs: signs(&[(Rashi::Aries, 99.5, 100.3), (Rashi::Taurus, 100.3, 102.0)]),
            window: window(),
        };
        assert_eq!(moon.sign(), Some(Rashi::Aries));
        assert_eq!(
            moon.changed_sign_at()
                .map(teistro_core::quantity::JulianDay::get),
            Some(100.3)
        );
        // A day the Moon does not change sign in has no transition, and
        // one with no events reports none rather than a placeholder.
        let still = MoonDay {
            rises: Vec::new(),
            sets: Vec::new(),
            signs: signs(&[(Rashi::Aries, 99.0, 102.0)]),
            window: window(),
        };
        assert!(still.changed_sign_at().is_none());
        assert!(still.rises.is_empty());
    }
}
