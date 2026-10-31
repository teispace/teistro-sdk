//! The day in sentences (`03-design/day-in-sentences.md`, SR-13): what an
//! almanac's answer says of one day, as short declarative sentences a
//! reader can be read aloud, each a message of `sdk.day` whose slots are
//! the answer's own values.
//!
//! The composer decides nothing. It says the members the day's spans name
//! and the instants they end at, read on the clock the day was asked on
//! and rounded to the minute a printed day prints. A fact the day has not
//! got is not said, and a fact a polar day holds only somewhere else is
//! named in [`DayPlan::withheld`], so absence is reported as absence
//! (`03-design/panchanga-day.md` §15).

use teistro_calendar::lunisolar::MonthKind;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::PolarDayPolicy;
use teistro_core::time::LocalClock;
use teistro_intl::messages::sdk::day as messages;
use teistro_intl::{ClockTime, Params, TypedMessage};
use teistro_panchanga::{Panchanga, Span};
use teistro_time::{DayState, PolarKind};

/// A sentence before it is said: a message and its slots.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Said {
    pub(crate) key: String,
    pub(crate) params: Params,
}

/// The sentences of a day, in order. Not `teistro_interpret::Plan`, which
/// the `panchanga` profile leaves out with the rest of the chart family
/// (`03-design/day-in-sentences.md` §3).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Plan {
    pub(crate) items: Vec<Said>,
}

impl Plan {
    fn say<M: TypedMessage>(&mut self, message: &M) {
        self.items.push(Said {
            key: String::from(M::KEY),
            params: message.params(),
        });
    }
}

/// Every message the day's composer can emit. The answer's refusal reads
/// it: a locale that does not carry all of these has no templates for a
/// day.
pub(crate) const DAY_KEYS: [&str; 18] = [
    <messages::Vara as TypedMessage>::KEY,
    <messages::Tithi as TypedMessage>::KEY,
    <messages::Nakshatra as TypedMessage>::KEY,
    <messages::Yoga as TypedMessage>::KEY,
    <messages::Karana as TypedMessage>::KEY,
    <messages::Sun as TypedMessage>::KEY,
    <messages::SunUp as TypedMessage>::KEY,
    <messages::SunDown as TypedMessage>::KEY,
    <messages::SunSign as TypedMessage>::KEY,
    <messages::Sankranti as TypedMessage>::KEY,
    <messages::MoonSign as TypedMessage>::KEY,
    <messages::Moonrise as TypedMessage>::KEY,
    <messages::Moonset as TypedMessage>::KEY,
    <messages::Month as TypedMessage>::KEY,
    <messages::Ritu as TypedMessage>::KEY,
    <messages::Ayana as TypedMessage>::KEY,
    <messages::Kaala as TypedMessage>::KEY,
    <messages::Abhijit as TypedMessage>::KEY,
];

/// `ends` when the fact ends on the civil day it opened on.
const SAME_DAY: &str = "sameDay";
/// `ends` when it ends after the next midnight, inside the day.
const NEXT_DAY: &str = "nextDay";
/// `ends` when it held from the day's opening past its close.
const ALL_DAY: &str = "allDay";
/// `ends` when it began in the day and held past its close.
const REST: &str = "rest";

/// What a polar day under `NEAREST_EVENT` does not say. Its window stands
/// at the nearest real sunrise, which may be weeks off, so these facts are
/// true of that stretch and not of the date asked.
pub(crate) const WITHHELD: [&str; 9] = [
    "tithi",
    "nakshatra",
    "yoga",
    "karana",
    "sun",
    "moon",
    "month",
    "ayana",
    "kaala",
];

/// A day's sentences as a plan, and the facts it did not say.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DayPlan {
    /// The sentences, in the order a printed day runs.
    pub(crate) plan: Plan,
    /// The facts a polar day holds elsewhere and so does not say here,
    /// from [`WITHHELD`]; empty on any other day.
    pub(crate) withheld: Vec<String>,
}

/// An instant as the day's clock shows it.
struct Reading<'a> {
    clock: &'a dyn LocalClock,
    window: Interval,
    /// The civil day the window opened on.
    opening: i64,
}

impl<'a> Reading<'a> {
    fn of(day: &Panchanga, clock: &'a dyn LocalClock) -> Reading<'a> {
        let mut reading = Reading {
            clock,
            window: day.window,
            opening: 0,
        };
        reading.opening = reading.minutes(day.window.from).div_euclid(MINUTES_A_DAY);
        reading
    }

    /// Whole minutes on the clock since its civil epoch, to the nearest,
    /// so the day a rounded time belongs to is the time's own.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a Julian day's minutes are some 3.5e9, well inside an i64"
    )]
    fn minutes(&self, at: JulianDay<Utc>) -> i64 {
        ((self.clock.local_jd(at) + 0.5) * MINUTES_A_DAY_F).round() as i64
    }

    /// The time an instant shows.
    fn time(&self, at: JulianDay<Utc>) -> ClockTime {
        let minute = self.minutes(at).rem_euclid(MINUTES_A_DAY);
        ClockTime::new(
            u8::try_from(minute / 60).unwrap_or(0),
            u8::try_from(minute % 60).unwrap_or(0),
            0,
        )
    }

    /// Whether an instant falls on the civil day the window opened on or
    /// a later one, as a `when` slot.
    fn when(&self, at: JulianDay<Utc>) -> String {
        let later = self.minutes(at).div_euclid(MINUTES_A_DAY) > self.opening;
        String::from(if later { NEXT_DAY } else { SAME_DAY })
    }

    /// How a span's end reads in the day, and the time it ends at. The
    /// time is the answer's in every case, though the two that run past
    /// the close do not print it.
    fn ends<T>(&self, span: &Span<T>) -> (String, ClockTime) {
        let until = self.time(span.whole.to);
        let ends = if span.whole.to.get() < self.window.to.get() {
            self.when(span.whole.to)
        } else if span.whole.from.get() <= self.window.from.get() {
            String::from(ALL_DAY)
        } else {
            String::from(REST)
        };
        (ends, until)
    }
}

/// The minutes of a civil day.
const MINUTES_A_DAY: i64 = 1440;
/// The same, to scale a Julian day by.
const MINUTES_A_DAY_F: f64 = 1440.0;

/// The day's facts as sentences: its vara and five-limb reckoning, the
/// Sun's and the Moon's day, the lunar month and season, the censured
/// parts of the daylight and Abhijit, each read from `day` on `clock`.
///
/// A limb that changes in the day is said once for each member, so a
/// karana is said three or four times. A polar day says the Sun neither
/// rises nor sets rather than the bounds a policy stood in. Under
/// `NEAREST_EVENT` those bounds are elsewhere, so it also says only the
/// vara and the season, the two facts read from the civil date, and names
/// the rest in [`DayPlan::withheld`].
///
/// Festivals are not said: a day's answer does not reckon them
/// (`03-design/day-in-sentences.md` §5).
#[must_use]
pub(crate) fn day_sentences(day: &Panchanga, clock: &dyn LocalClock) -> DayPlan {
    let reading = Reading::of(day, clock);
    let mut plan = Plan::default();
    plan.say(&messages::Vara { vara: day.day.vara });
    let polar = match day.day.state {
        DayState::Polar { kind, policy } => Some((kind, policy)),
        DayState::Normal => None,
    };
    if matches!(polar, Some((_, PolarDayPolicy::NearestEvent))) {
        say_polar(&mut plan, polar);
        plan.say(&messages::Ritu { ritu: day.sun.ritu });
        return DayPlan {
            plan,
            withheld: WITHHELD.iter().map(|fact| String::from(*fact)).collect(),
        };
    }
    limbs(&mut plan, day, &reading);
    if polar.is_some() {
        say_polar(&mut plan, polar);
    } else {
        plan.say(&messages::Sun {
            rise: reading.time(day.day.sunrise),
            set: reading.time(day.day.sunset),
        });
    }
    sun_and_moon(&mut plan, day, &reading);
    plan.say(&messages::Month {
        masa: day.month.month,
        kind: String::from(if day.month.kind == MonthKind::Adhika {
            "adhika"
        } else {
            "nija"
        }),
    });
    plan.say(&messages::Ritu { ritu: day.sun.ritu });
    plan.say(&messages::Ayana {
        ayana: day.sun.ayana,
    });
    for part in &day.kaalas {
        plan.say(&messages::Kaala {
            kaala: part.kaala,
            from: reading.time(part.at.from),
            to: reading.time(part.at.to),
        });
    }
    if let Some(abhijit) = day.muhurtas.abhijit {
        plan.say(&messages::Abhijit {
            from: reading.time(abhijit.from),
            to: reading.time(abhijit.to),
        });
    }
    DayPlan {
        plan,
        withheld: Vec::new(),
    }
}

/// That the Sun neither rose nor set.
fn say_polar(plan: &mut Plan, polar: Option<(PolarKind, PolarDayPolicy)>) {
    match polar {
        Some((PolarKind::Day, _)) => plan.say(&messages::SunUp),
        Some((PolarKind::Night, _)) => plan.say(&messages::SunDown),
        _ => {}
    }
}

/// The four moving limbs, each member as long as it ran.
fn limbs(plan: &mut Plan, day: &Panchanga, reading: &Reading<'_>) {
    for span in &day.limbs.tithi {
        let (ends, until) = reading.ends(span);
        plan.say(&messages::Tithi {
            tithi: span.member,
            paksha: span.member.attributes().paksha,
            ends,
            until,
        });
    }
    for span in &day.limbs.nakshatra {
        let (ends, until) = reading.ends(span);
        plan.say(&messages::Nakshatra {
            nakshatra: span.member,
            ends,
            until,
        });
    }
    for span in &day.limbs.yoga {
        let (ends, until) = reading.ends(span);
        plan.say(&messages::Yoga {
            yoga: span.member,
            ends,
            until,
        });
    }
    for span in &day.limbs.karana {
        let (ends, until) = reading.ends(span);
        plan.say(&messages::Karana {
            karana: span.member,
            ends,
            until,
        });
    }
}

/// The Sun's sign and its sankranti, the Moon's signs, its rises and sets.
fn sun_and_moon(plan: &mut Plan, day: &Panchanga, reading: &Reading<'_>) {
    if let Some(sign) = day.sun.signs.first() {
        plan.say(&messages::SunSign { rashi: sign.member });
    }
    if let (Some(at), Some(entered)) = (day.sun.sankranti, day.sun.signs.get(1)) {
        plan.say(&messages::Sankranti {
            rashi: entered.member,
            at: reading.time(at),
            when: reading.when(at),
        });
    }
    for span in &day.moon.signs {
        let (ends, until) = reading.ends(span);
        plan.say(&messages::MoonSign {
            rashi: span.member,
            ends,
            until,
        });
    }
    for at in &day.moon.rises {
        plan.say(&messages::Moonrise {
            at: reading.time(*at),
            when: reading.when(*at),
        });
    }
    for at in &day.moon.sets {
        plan.say(&messages::Moonset {
            at: reading.time(*at),
            when: reading.when(*at),
        });
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_astro::delta_t::DeltaTModel;
    use teistro_astro::precession::PrecessionModel;
    use teistro_calendar::solar::drik::DrikSun;
    use teistro_calendar::{CalendarDate, Gregorian};
    use teistro_core::catalogue::{Ayanamsha, Calendar};
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro_core::settings::{
        DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise,
    };
    use teistro_core::time::UtcOffset;
    use teistro_intl::source::Tree;
    use teistro_intl::{Intl, Value, sdk_root};
    use teistro_panchanga::Almanac;
    use teistro_port_ephemeris::test_provider::TestProvider;

    use super::*;

    const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);

    /// One day of the plausible test sky, under a polar policy.
    fn day_at(place: Place, date: (i32, u8, u8), policy: PolarDayPolicy) -> Panchanga {
        let provider = TestProvider;
        let mut patch = SettingsPatch::default();
        patch.day.polar_day_policy = Some(policy);
        let resolved = Profile::shipped(DEFAULT_PROFILE)
            .unwrap()
            .resolve(&patch)
            .unwrap();
        let model = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            Sunrise::CentreNoRefraction.into(),
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        let almanac = Almanac::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &NEPAL,
            PrecessionModel::Vondrak2011,
            DeltaTModel::TableThenModel,
        );
        let date = CalendarDate::defined(Calendar::Gregorian, date.0, date.1, date.2);
        almanac.day(&date, &place).unwrap().value
    }

    fn place(latitude: f64, longitude: f64) -> Place {
        Place::new(
            Latitude::literal(latitude),
            Longitude::literal(longitude),
            Altitude::literal(0.0),
        )
    }

    fn kathmandu_day() -> Panchanga {
        day_at(
            place(27.7172, 85.324),
            (2026, 3, 18),
            PolarDayPolicy::Undefined,
        )
    }

    fn said(plan: &Plan) -> Vec<&str> {
        plan.items.iter().map(|item| item.key.as_str()).collect()
    }

    fn rendered(plan: &Plan, locale: &str) -> Vec<String> {
        let intl = Intl::from_tree(&Tree::load(&sdk_root()).unwrap()).unwrap();
        plan.items
            .iter()
            .map(|item| {
                let out = intl.render_in(locale, &item.key, &item.params);
                assert_eq!(out.warnings, Vec::<String>::new(), "{}", item.key);
                assert!(!out.is_fallback, "{}", item.key);
                out.text
            })
            .collect()
    }

    /// Every sentence of a day is a template's, in English and in Nepali,
    /// with no fallback and nothing warned, and every key it says is
    /// listed.
    #[test]
    fn a_day_is_said_in_both_languages() {
        let day = kathmandu_day();
        let out = day_sentences(&day, &NEPAL);
        assert_eq!(out.withheld, Vec::<String>::new());
        for key in said(&out.plan) {
            assert!(DAY_KEYS.contains(&key), "`{key}` is not in DAY_KEYS");
        }
        let en = rendered(&out.plan, "en-Latn");
        assert!(en.iter().all(|text| text.ends_with('.')), "{en:?}");
        let ne = rendered(&out.plan, "ne-Deva-NP");
        assert!(ne.iter().all(|text| text.ends_with('।')), "{ne:?}");
        assert_eq!(en.len(), ne.len());
        assert_eq!(said(&out.plan)[0], "sdk.day.vara");
    }

    /// Each tithi's slots are the span's member and the time its end shows
    /// on the clock, to the nearest minute.
    #[test]
    fn a_slot_is_the_answers_own_value() {
        let day = kathmandu_day();
        let out = day_sentences(&day, &NEPAL);
        let first = out
            .plan
            .items
            .iter()
            .find(|item| item.key == "sdk.day.tithi")
            .unwrap();
        let span = &day.limbs.tithi[0];
        assert_eq!(
            first.params.get("tithi"),
            Some(&Value::catalogued(span.member))
        );
        #[allow(clippy::cast_possible_truncation, reason = "minutes of a Julian day")]
        let minutes = ((span.whole.to.get() + 0.5 + 5.75 / 24.0) * 1440.0).round() as i64 % 1440;
        assert_eq!(
            first.params.get("until"),
            Some(&Value::Time(ClockTime::new(
                u8::try_from(minutes / 60).unwrap(),
                u8::try_from(minutes % 60).unwrap(),
                0
            )))
        );
        let sun = out
            .plan
            .items
            .iter()
            .find(|item| item.key == "sdk.day.sun")
            .unwrap();
        assert!(sun.params.contains_key("rise") && sun.params.contains_key("set"));
    }

    /// The four ways a limb's end reads, each from a span placed for it.
    #[test]
    fn a_span_reads_by_where_it_ends() {
        let day = kathmandu_day();
        let reading = Reading::of(&day, &NEPAL);
        let (from, to) = (day.window.from, day.window.to);
        let span = |a: f64, b: f64| Span {
            member: (),
            whole: Interval::literal(a, b),
            inside: Interval::literal(from.get(), to.get()),
        };
        let open = from.get();
        let close = to.get();
        // An end before the clock's midnight, and one after it.
        let midnight = ((open + 5.75 / 24.0 + 0.5).ceil() - 0.5 - 5.75 / 24.0).min(close - 0.01);
        assert_eq!(reading.ends(&span(open, open + 0.01)).0, SAME_DAY);
        assert_eq!(reading.ends(&span(open, midnight + 0.01)).0, NEXT_DAY);
        assert_eq!(reading.ends(&span(open - 1.0, close + 1.0)).0, ALL_DAY);
        assert_eq!(reading.ends(&span(open + 0.1, close + 1.0)).0, REST);
    }

    /// A time rounding up to midnight is the next day's midnight, in its
    /// clock face and in its day together.
    #[test]
    fn a_rounded_time_keeps_its_own_day() {
        let day = kathmandu_day();
        let reading = Reading::of(&day, &NEPAL);
        let next = ((day.window.from.get() + 5.75 / 24.0 + 0.5).ceil() - 0.5) - 5.75 / 24.0;
        let late = JulianDay::literal(next - 10.0 / 86_400.0);
        assert_eq!(reading.time(late), ClockTime::new(0, 0, 0));
        assert_eq!(reading.when(late), NEXT_DAY);
    }

    /// An adhika month says so.
    #[test]
    fn an_adhika_month_is_said() {
        let mut day = kathmandu_day();
        day.month.kind = MonthKind::Adhika;
        let out = day_sentences(&day, &NEPAL);
        let en = rendered(&out.plan, "en-Latn");
        assert!(en.iter().any(|text| text.contains("is adhika")), "{en:?}");
    }

    /// A sankranti inside the day is said with the sign entered.
    #[test]
    fn a_sankranti_is_said_with_its_sign() {
        let mut day = kathmandu_day();
        let sign = day.sun.signs[0];
        let at = JulianDay::literal(day.window.from.get() + 0.1);
        let mut second = sign;
        second.member = teistro_core::catalogue::Rashi::Aries;
        day.sun.signs = vec![sign, second];
        day.sun.sankranti = Some(at);
        let out = day_sentences(&day, &NEPAL);
        assert!(said(&out.plan).contains(&"sdk.day.sankranti"));
        let en = rendered(&out.plan, "en-Latn");
        assert!(en.iter().any(|text| text.contains("enters")), "{en:?}");
    }

    /// Above the circle under `NEAREST_EVENT` the window stands elsewhere,
    /// so the day says the Sun's state, the vara and the season, and names
    /// the rest as withheld.
    #[test]
    fn a_polar_day_says_only_what_is_true_of_its_date() {
        let tromso = place(69.6492, 18.9553);
        for (date, sentence) in [
            ((2024, 6, 21), "sdk.day.sunUp"),
            ((2024, 12, 21), "sdk.day.sunDown"),
        ] {
            let day = day_at(tromso, date, PolarDayPolicy::NearestEvent);
            let out = day_sentences(&day, &UtcOffset::literal(1, 0, 0));
            assert_eq!(said(&out.plan), ["sdk.day.vara", sentence, "sdk.day.ritu"]);
            assert_eq!(out.withheld.len(), WITHHELD.len());
            assert_eq!(rendered(&out.plan, "ne-Deva-NP").len(), 3);
        }
    }

    /// Under `CIVIL_MIDNIGHT` the window is the civil date, so the limbs
    /// are said; only the sunrise is not.
    #[test]
    fn a_civil_polar_day_says_its_limbs_and_no_sunrise() {
        let day = day_at(
            place(69.6492, 18.9553),
            (2024, 6, 21),
            PolarDayPolicy::CivilMidnight,
        );
        let out = day_sentences(&day, &UtcOffset::literal(1, 0, 0));
        let keys = said(&out.plan);
        assert!(keys.contains(&"sdk.day.tithi") && keys.contains(&"sdk.day.sunUp"));
        assert!(!keys.contains(&"sdk.day.sun"));
        assert_eq!(out.withheld, Vec::<String>::new());
    }
}
