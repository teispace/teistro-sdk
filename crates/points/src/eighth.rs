//! The upagrahas the day divides: Gulika and Mandi, which are the
//! ascendant at the two ends of Saturn's eighth of the arc, and Kala,
//! Mrityu, Yamaghantaka and Ardhaprahara, which are the ascendant where
//! the Sun's, Mars's, Jupiter's and Mercury's eighths begin.
//!
//! The arc a birth falls in — the daylight, or the night that follows
//! it — is cut into eight. Which eighth is Saturn's the catalogue
//! carries, measured against the recorded panchanga on all 55 days
//! (`Kaala::GulikaKaala`); a night birth walks that table five weekdays
//! on, the same walk the choghadiya take.
//!
//! **Gulika begins the portion and Mandi ends it.** The research page
//! marks both halves of that "verify" — where inside the portion, and
//! whether the two names differ at all — and the falsification pass
//! settled it by trying all twenty-four candidate instants against every
//! recorded value rather than proposing one: two readings of one
//! portion, start against end and not start against middle, on all 71
//! fixtures (`03-design/points-measured.md` §6).
//!
//! **The other four are the same rule with another lord.** BPHS ch. 3
//! vv. 66 to 69 (the numbering of the sanskritdocuments Devanagari
//! e-text; the 1923 Subodhini print numbers them 67 to 70) give the
//! eighths' lords in weekday order from the vara's own lord by day and
//! from the fifth lord by night, leave the eighth eighth unlorded
//! (निरीशः), and name five portions together: Saturn's गुलिक, the Sun's
//! काल, Mars's मृत्यु, Jupiter's यमघण्टक and Mercury's (सौम्य)
//! अर्धप्रहरक. The corpus records none of the four, so where inside its
//! portion each is read is crux C405, and the default is where the SDK's
//! own Gulika stands — the start ([`Moment`]).
//!
//! What this needs and a foundation does not carry is the **ascendant at
//! an instant that is not the birth**, which is why [`Ascendant`] is a
//! trait rather than a dependency: the ascendant is the sidereal time
//! and the latitude, so no ephemeris is involved and a test implements
//! it in four lines.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Vara};
//! use teistro_points::eighth::lords_eighth;
//!
//! // A Sunday's daylight begins with the Sun's own eighth, which is
//! // Kala's, and Saturn's is the seventh.
//! assert_eq!(lords_eighth(Graha::Sun, Vara::Ravivara, true), Some(1));
//! assert_eq!(lords_eighth(Graha::Saturn, Vara::Ravivara, true), Some(7));
//! // Its night begins with the fifth lord, Jupiter, whose eighth is
//! // Yamaghantaka's.
//! assert_eq!(lords_eighth(Graha::Jupiter, Vara::Ravivara, false), Some(1));
//! // The nodes lord no weekday, so no eighth is theirs.
//! assert_eq!(lords_eighth(Graha::Rahu, Vara::Ravivara, true), None);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Kaala, Point, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use crate::derived::Derived;

/// How many parts an arc is cut into.
pub const EIGHTHS: u16 = 8;

/// How many weekdays on the night's own sequence begins: the fifth lord
/// from the day's, which is four steps (वारेशात् पञ्चमादितः, BPHS ch. 3
/// v. 67).
pub const NIGHT_WALK: u8 = 4;

/// The weekdays a sequence of lords walks before it repeats.
const WEEK: u8 = 7;

/// The upagrahas BPHS reads off a lord's eighth of the arc, each with
/// that lord, in the order ch. 3 vv. 68 to 69 name them: शन्यंशो गुलिकः,
/// रव्यंशः कालसञ्ज्ञकः, भौमांशो मृत्युः, गुर्वंशो यमघण्टकः,
/// सौम्यांशोऽर्धप्रहरकः.
///
/// The Moon's and Venus's eighths carry no named point, and the eighth
/// eighth has no lord at all.
pub const PORTION_LORDS: [(Point, Graha); 5] = [
    (Point::Gulika, Graha::Saturn),
    (Point::Kala, Graha::Sun),
    (Point::Mrityu, Graha::Mars),
    (Point::Yamaghantaka, Graha::Jupiter),
    (Point::Ardhaprahara, Graha::Mercury),
];

/// The four of [`PORTION_LORDS`] that are not Gulika — which
/// [`day_division`] already reads, with Mandi — in the same order.
pub const BESIDE_GULIKA: [Point; 4] = [
    Point::Kala,
    Point::Mrityu,
    Point::Yamaghantaka,
    Point::Ardhaprahara,
];

/// Something that can say where the ecliptic rises at an instant.
///
/// The day-division points are read off the ascendant inside a
/// portion, which is not the birth, so this module cannot take those
/// from a founded chart. It takes them from whatever the caller has:
/// `chart` implements this over `astro::houses`, and a test implements
/// it over a straight line.
pub trait Ascendant {
    /// The ascendant at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged. This module
    /// does not wrap it.
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;
}

impl<F> Ascendant for F
where
    F: Fn(JulianDay<Utc>) -> Result<f64, Error>,
{
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        self(at)
    }
}

/// A lord's eighth of an arc: which part it is, and when it runs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Portion {
    /// Which eighth, counted from one as an almanac prints it.
    pub eighth: u8,
    /// When it runs.
    pub at: Interval,
}

/// Where inside its lord's portion an upagraha is read.
///
/// BPHS ch. 3 v. 70 reads Gulika as the lagna at its portion's time
/// (गुलिकेष्टवशाल्लग्नं) without saying which instant of the portion that
/// is. The SDK's Gulika is the start, as the corpus measured; the 1923
/// Subodhini print's worked example multiplies an eighth by the part's
/// ordinal, which is the end, and Sareen's translation of Uttara
/// Kalamrita (Kanda I, section 1, v. 8) reads Gulika at the end too. So
/// the instant is the caller's to change, and the start, which keeps the
/// five portions one rule with the shipped Gulika, is the default (crux
/// C405).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Moment {
    /// Where the portion begins, as the SDK's Gulika is read.
    #[default]
    Start,
    /// Halfway through the portion.
    Middle,
    /// Where the portion ends, as the SDK's Mandi is read.
    End,
}

impl Moment {
    /// The instant this reading takes from a portion.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_points::eighth::Moment;
    ///
    /// let portion = Interval::literal(2_460_000.25, 2_460_000.3125);
    /// let at = |moment: Moment| moment.of(portion).expect("finite").get();
    /// assert!((at(Moment::Start) - 2_460_000.25).abs() < 1e-9);
    /// assert!((at(Moment::Middle) - 2_460_000.281_25).abs() < 1e-9);
    /// assert!((at(Moment::End) - 2_460_000.3125).abs() < 1e-9);
    /// ```
    ///
    /// # Errors
    ///
    /// None in practice: the middle is a finite fraction of a finite
    /// interval, and the `Result` is `Interval::at_fraction`'s.
    pub fn of(self, portion: Interval) -> Result<JulianDay<Utc>, Error> {
        match self {
            Moment::Start => Ok(portion.from),
            Moment::Middle => portion.at_fraction(0.5),
            Moment::End => Ok(portion.to),
        }
    }
}

/// The weekday whose lord takes the first eighth of an arc: the vara's
/// own by day, the fifth lord from it by night.
fn first_weekday(vara: Vara, is_day: bool) -> u8 {
    let weekday = vara.attributes().weekday;
    if is_day {
        weekday
    } else {
        (weekday + NIGHT_WALK) % WEEK
    }
}

/// Which eighth of its arc is a graha's, on a chart of this vara and
/// half of the day, counted from one, or `None` for a graha that lords
/// no weekday.
///
/// The seven weekday lords take the first seven eighths in weekday
/// order, beginning with the vara's own lord by day and the fifth from
/// it by night (BPHS ch. 3 vv. 66 to 67); the eighth eighth has none.
/// For Saturn this is the catalogue's measured table, which
/// [`saturns_eighth`] reads, and a test holds the two to each other on
/// all fourteen halves of the week.
#[must_use]
pub fn lords_eighth(lord: Graha, vara: Vara, is_day: bool) -> Option<u8> {
    let own = Vara::ALL
        .into_iter()
        .find(|day| day.attributes().lord == lord)?
        .attributes()
        .weekday;
    Some((own + WEEK - first_weekday(vara, is_day)) % WEEK + 1)
}

/// The lord whose eighth an upagraha is, or `None` for a point that is
/// not one of [`PORTION_LORDS`].
///
/// ```
/// use teistro_core::catalogue::{Graha, Point};
/// use teistro_points::eighth::lord_of;
///
/// assert_eq!(lord_of(Point::Yamaghantaka), Some(Graha::Jupiter));
/// assert_eq!(lord_of(Point::Mandi), None, "Mandi is Saturn's portion read at its end");
/// ```
#[must_use]
pub fn lord_of(point: Point) -> Option<Graha> {
    PORTION_LORDS
        .into_iter()
        .find(|(named, _)| *named == point)
        .map(|(_, lord)| lord)
}

/// Which eighth of its arc is Saturn's, on a chart of this vara and
/// half of the day, counted from one.
///
/// The daylight takes the catalogue's table as it stands; the night
/// takes it walked [`NIGHT_WALK`] weekdays on.
#[must_use]
pub fn saturns_eighth(vara: Vara, is_day: bool) -> Option<u8> {
    Kaala::GulikaKaala
        .attributes()
        .eighth_by_vara
        .get(usize::from(first_weekday(vara, is_day)))
        .copied()
}

/// An arc with a length, or the refusal a polar day or night gets: an
/// eighth of nothing is not an instant, and an empty portion would answer
/// "Gulika is at sunrise" for a place where the Sun did not rise.
fn with_length(arc: Interval, is_day: bool) -> Result<Interval, Error> {
    if arc.is_empty() {
        return Err(Error::unsupported(format!(
            "the {} arc has no length, so it has no eighths; a polar day or night \
             has no Gulika and no other upagraha the day divides",
            if is_day { "daylight" } else { "night" }
        ))
        .with_field("points.arc"));
    }
    Ok(arc)
}

/// The `eighth`th part of an arc, counted from one.
fn cut(arc: Interval, eighth: u8) -> Result<Portion, Error> {
    let at = arc.part(u16::from(eighth.saturating_sub(1)), EIGHTHS)?;
    Ok(Portion { eighth, at })
}

/// Saturn's portion of an arc.
///
/// # Errors
///
/// `UNSUPPORTED` for an arc with no length, which is what a polar day or
/// a polar night gives: an eighth of nothing is not an instant, and an
/// empty portion would answer "Gulika is at sunrise" for a place where
/// the Sun did not rise.
pub fn saturns(arc: Interval, vara: Vara, is_day: bool) -> Result<Portion, Error> {
    let arc = with_length(arc, is_day)?;
    let eighth = saturns_eighth(vara, is_day).ok_or_else(|| {
        Error::unsupported(format!("no eighth is Saturn's on {}", vara.key()))
            .with_field("points.vara")
    })?;
    cut(arc, eighth)
}

/// A graha's portion of an arc.
///
/// ```
/// use teistro_core::catalogue::{Graha, Vara};
/// use teistro_core::interval::Interval;
/// use teistro_points::eighth::lords;
///
/// // Mars takes a Sunday's third eighth: the Sun, the Moon, Mars.
/// let day = Interval::literal(2_460_000.25, 2_460_000.75);
/// let portion = lords(day, Graha::Mars, Vara::Ravivara, true).expect("a day");
/// assert_eq!(portion.eighth, 3);
/// assert!((portion.at.from.get() - 2_460_000.375).abs() < 1e-9);
/// ```
///
/// # Errors
///
/// `UNSUPPORTED` for an arc with no length, as [`saturns`], and for a
/// graha that lords no weekday — a node or an outer planet — naming
/// `points.lord`.
pub fn lords(arc: Interval, lord: Graha, vara: Vara, is_day: bool) -> Result<Portion, Error> {
    let arc = with_length(arc, is_day)?;
    let eighth = lords_eighth(lord, vara, is_day).ok_or_else(|| {
        Error::unsupported(format!(
            "{} lords no weekday, so no eighth of the day is its; only the seven \
             from SUN to SATURN do",
            lord.key()
        ))
        .with_field("points.lord")
    })?;
    cut(arc, eighth)
}

/// Gulika and Mandi: the ascendant where Saturn's portion begins and
/// where it ends.
///
/// # Errors
///
/// As [`saturns`], and whatever the [`Ascendant`] returns.
pub fn day_division(
    arc: Interval,
    vara: Vara,
    is_day: bool,
    ascendant: &dyn Ascendant,
) -> Result<[Derived; 2], Error> {
    let portion = saturns(arc, vara, is_day)?;
    Ok([
        Derived::at(Point::Gulika, ascendant.ascendant_deg(portion.at.from)?)?,
        Derived::at(Point::Mandi, ascendant.ascendant_deg(portion.at.to)?)?,
    ])
}

/// One upagraha of [`PORTION_LORDS`]: the ascendant at a moment of its
/// lord's eighth.
///
/// ```
/// use teistro_core::catalogue::{Point, Vara};
/// use teistro_core::error::Error;
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::{JulianDay, Utc};
/// use teistro_points::eighth::{Moment, upagraha};
///
/// // An ascendant that turns once a day, which is enough to tell one
/// // instant from another.
/// let rising = |at: JulianDay<Utc>| -> Result<f64, Error> {
///     Ok((at.get() * 360.0).rem_euclid(360.0))
/// };
/// let day = Interval::literal(2_460_000.25, 2_460_000.75);
/// // Mrityu on a Sunday is Mars's third eighth, which begins a quarter
/// // of the way through the day: the turning ascendant's 135°.
/// let mrityu = upagraha(Point::Mrityu, day, Vara::Ravivara, true, Moment::Start, &rising)
///     .expect("a day with eighths");
/// assert!((mrityu.longitude_deg - 135.0).abs() < 1e-9);
/// ```
///
/// # Errors
///
/// `UNSUPPORTED` for a point that is not read off a lord's eighth,
/// naming `points.point`; as [`lords`]; and whatever the [`Ascendant`]
/// returns, unchanged.
pub fn upagraha(
    point: Point,
    arc: Interval,
    vara: Vara,
    is_day: bool,
    moment: Moment,
    ascendant: &dyn Ascendant,
) -> Result<Derived, Error> {
    let lord = lord_of(point).ok_or_else(|| {
        Error::unsupported(format!(
            "{} is not read off a lord's eighth of the day; GULIKA, KALA, MRITYU, \
             YAMAGHANTAKA and ARDHAPRAHARA are",
            point.key()
        ))
        .with_field("points.point")
    })?;
    let portion = lords(arc, lord, vara, is_day)?;
    Derived::at(point, ascendant.ascendant_deg(moment.of(portion.at)?)?)
}

/// Kala, Mrityu, Yamaghantaka and Ardhaprahara, each the ascendant where
/// its lord's eighth begins — the default of crux C405.
///
/// # Errors
///
/// As [`lorded_at`].
pub fn lorded(
    arc: Interval,
    vara: Vara,
    is_day: bool,
    ascendant: &dyn Ascendant,
) -> Result<[Derived; 4], Error> {
    lorded_at(arc, vara, is_day, Moment::Start, ascendant)
}

/// Kala, Mrityu, Yamaghantaka and Ardhaprahara, each read at a chosen
/// moment of its lord's eighth.
///
/// # Errors
///
/// As [`upagraha`].
pub fn lorded_at(
    arc: Interval,
    vara: Vara,
    is_day: bool,
    moment: Moment,
    ascendant: &dyn Ascendant,
) -> Result<[Derived; 4], Error> {
    let read = |point: Point| upagraha(point, arc, vara, is_day, moment, ascendant);
    let [kala, mrityu, yamaghantaka, ardhaprahara] = BESIDE_GULIKA;
    Ok([
        read(kala)?,
        read(mrityu)?,
        read(yamaghantaka)?,
        read(ardhaprahara)?,
    ])
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::unnecessary_wraps,
        reason = "tests fail by panicking, index a fixed-length pair, compare longitudes computed the same way, and match the `Ascendant` trait's shape"
    )]

    use super::{
        BESIDE_GULIKA, EIGHTHS, Moment, PORTION_LORDS, day_division, lord_of, lorded, lorded_at,
        lords, lords_eighth, saturns, saturns_eighth, upagraha,
    };
    use teistro_core::catalogue::{Graha, Point, Vara};
    use teistro_core::error::Error;
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Utc};

    /// An ascendant that turns once through the day, which is enough to
    /// tell one instant from another.
    fn rising(at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok((at.get() * 360.0).rem_euclid(360.0))
    }

    /// A daylight of half a day, so each eighth is 0.0625 of a day and
    /// the turning ascendant moves 22.5° across one. Every instant here
    /// is a sum of powers of two, so the arithmetic below is exact.
    const DAY: Interval = Interval::literal(2_460_000.25, 2_460_000.75);

    #[test]
    fn saturns_eighth_is_the_catalogue_s_by_day_and_walked_by_night() {
        // The daylight table, Sunday first.
        let day: Vec<u8> = Vara::ALL
            .into_iter()
            .map(|vara| saturns_eighth(vara, true).unwrap())
            .collect();
        assert_eq!(day, vec![7, 6, 5, 4, 3, 2, 1]);
        // The night's is the same table five weekdays on.
        let night: Vec<u8> = Vara::ALL
            .into_iter()
            .map(|vara| saturns_eighth(vara, false).unwrap())
            .collect();
        assert_eq!(night, vec![3, 2, 1, 7, 6, 5, 4]);
        for (index, vara) in Vara::ALL.into_iter().enumerate() {
            assert_eq!(
                saturns_eighth(vara, false),
                Some(day[(index + 4) % 7]),
                "{vara:?}"
            );
        }
    }

    #[test]
    fn the_verse_s_rule_reproduces_the_catalogue_s_measured_saturn() {
        // The catalogue's Saturn was measured against the recorded
        // panchanga; the rule of BPHS ch. 3 vv. 66 to 67 was not. They
        // agree on all fourteen halves of the week, which is what lets
        // the rule stand for the four lords nothing recorded.
        for vara in Vara::ALL {
            for is_day in [true, false] {
                assert_eq!(
                    lords_eighth(Graha::Saturn, vara, is_day),
                    saturns_eighth(vara, is_day),
                    "{vara:?} {is_day}"
                );
            }
        }
    }

    #[test]
    fn each_lord_s_eighth_is_the_hand_counted_one() {
        // Counted by hand from the verse. By day the vara's lord takes
        // the first eighth and the rest follow in weekday order: on a
        // Sunday the Sun 1, the Moon 2, Mars 3, Mercury 4, Jupiter 5,
        // Venus 6, Saturn 7. By night the count starts at the fifth
        // lord: on a Sunday night Jupiter 1, Venus 2, Saturn 3, the Sun 4,
        // the Moon 5, Mars 6, Mercury 7. Each row is Sunday first.
        let expected: [(Graha, [u8; 7], [u8; 7]); 4] = [
            (Graha::Sun, [1, 7, 6, 5, 4, 3, 2], [4, 3, 2, 1, 7, 6, 5]),
            (Graha::Mars, [3, 2, 1, 7, 6, 5, 4], [6, 5, 4, 3, 2, 1, 7]),
            (Graha::Mercury, [4, 3, 2, 1, 7, 6, 5], [7, 6, 5, 4, 3, 2, 1]),
            (Graha::Jupiter, [5, 4, 3, 2, 1, 7, 6], [1, 7, 6, 5, 4, 3, 2]),
        ];
        for (lord, by_day, by_night) in expected {
            for (index, vara) in Vara::ALL.into_iter().enumerate() {
                assert_eq!(
                    lords_eighth(lord, vara, true),
                    Some(by_day[index]),
                    "{lord:?} {vara:?}"
                );
                assert_eq!(
                    lords_eighth(lord, vara, false),
                    Some(by_night[index]),
                    "{lord:?} {vara:?} night"
                );
            }
        }
    }

    #[test]
    fn the_seven_lords_take_seven_eighths_and_leave_the_last_unlorded() {
        // Named so as not to shadow `lords`, which this test calls below.
        let weekday_lords: Vec<Graha> = Vara::ALL
            .into_iter()
            .map(|vara| vara.attributes().lord)
            .collect();
        for vara in Vara::ALL {
            for is_day in [true, false] {
                let mut taken: Vec<u8> = weekday_lords
                    .iter()
                    .map(|&lord| lords_eighth(lord, vara, is_day).unwrap())
                    .collect();
                taken.sort_unstable();
                assert_eq!(taken, vec![1, 2, 3, 4, 5, 6, 7], "{vara:?} {is_day}");
            }
        }
        // A graha that lords no weekday has no eighth.
        for lord in [Graha::Rahu, Graha::Ketu] {
            assert_eq!(lords_eighth(lord, Vara::ALL[0], true), None);
            let error = lords(DAY, lord, Vara::ALL[0], true).expect_err("no weekday");
            assert_eq!(error.field(), Some("points.lord"));
            assert!(error.message.contains(lord.key()), "{error}");
        }
    }

    #[test]
    fn the_five_named_portions_are_the_verse_s() {
        assert_eq!(lord_of(Point::Gulika), Some(Graha::Saturn));
        assert_eq!(lord_of(Point::Kala), Some(Graha::Sun));
        assert_eq!(lord_of(Point::Mrityu), Some(Graha::Mars));
        assert_eq!(lord_of(Point::Yamaghantaka), Some(Graha::Jupiter));
        assert_eq!(lord_of(Point::Ardhaprahara), Some(Graha::Mercury));
        assert_eq!(lord_of(Point::Mandi), None);
        // Every one beside Gulika is in the table, once.
        for point in BESIDE_GULIKA {
            assert_eq!(
                PORTION_LORDS
                    .iter()
                    .filter(|(named, _)| *named == point)
                    .count(),
                1,
                "{point:?}"
            );
        }
    }

    #[test]
    fn the_eight_portions_partition_the_arc() {
        for vara in Vara::ALL {
            for is_day in [true, false] {
                let portion = saturns(DAY, vara, is_day).unwrap();
                assert!((1..=8).contains(&portion.eighth), "{vara:?}");
                // The portion is one eighth of the arc, inside it.
                assert!(
                    (portion.at.days() - DAY.days() / f64::from(EIGHTHS)).abs() < 1e-12,
                    "{vara:?}"
                );
                assert!(portion.at.from.get() >= DAY.from.get() - 1e-12);
                assert!(portion.at.to.get() <= DAY.to.get() + 1e-12);
            }
        }
    }

    #[test]
    fn gulika_begins_the_portion_and_mandi_ends_it() {
        let vara = Vara::ALL[0];
        let portion = saturns(DAY, vara, true).unwrap();
        let found = day_division(DAY, vara, true, &rising).unwrap();
        assert_eq!(found[0].point, Point::Gulika);
        assert_eq!(found[1].point, Point::Mandi);
        assert!((found[0].longitude_deg - rising(portion.at.from).unwrap()).abs() < 1e-9);
        assert!((found[1].longitude_deg - rising(portion.at.to).unwrap()).abs() < 1e-9);
        assert_ne!(found[0].longitude_deg, found[1].longitude_deg);
        // And Gulika read through the general rule at the start is the
        // same point, on every half of the week.
        for vara in Vara::ALL {
            for is_day in [true, false] {
                let general =
                    upagraha(Point::Gulika, DAY, vara, is_day, Moment::Start, &rising).unwrap();
                let shipped = day_division(DAY, vara, is_day, &rising).unwrap()[0];
                assert_eq!(general, shipped, "{vara:?} {is_day}");
            }
        }
    }

    #[test]
    fn the_four_stand_where_the_hand_count_puts_them() {
        // A Sunday's daylight from JD 2460000.25, eighths of 0.0625 day.
        // The turning ascendant is 360° times the day's fraction:
        //   Kala, the Sun's 1st eighth, starts at .25            → 90°
        //   Mrityu, Mars's 3rd, starts at .25 + 2 × .0625 = .375 → 135°
        //   Yamaghantaka, Jupiter's 5th, starts at .5            → 180°
        //   Ardhaprahara, Mercury's 4th, starts at .4375         → 157.5°
        let found = lorded(DAY, Vara::Ravivara, true, &rising).unwrap();
        let expected = [
            (Point::Kala, 90.0),
            (Point::Mrityu, 135.0),
            (Point::Yamaghantaka, 180.0),
            (Point::Ardhaprahara, 157.5),
        ];
        for (derived, (point, longitude)) in found.iter().zip(expected) {
            assert_eq!(derived.point, point);
            assert!(
                (derived.longitude_deg - longitude).abs() < 1e-9,
                "{point:?}: {} against {longitude}",
                derived.longitude_deg
            );
        }
        // A Sunday night over the same arc counts from Jupiter, so
        // Yamaghantaka takes the first eighth and Kala the fourth:
        //   Yamaghantaka at .25 → 90°; Kala at .25 + 3 × .0625 = .4375 → 157.5°.
        let night = lorded(DAY, Vara::Ravivara, false, &rising).unwrap();
        assert!((night[2].longitude_deg - 90.0).abs() < 1e-9);
        assert!((night[0].longitude_deg - 157.5).abs() < 1e-9);
    }

    #[test]
    fn the_moment_moves_the_point_through_its_portion() {
        // Kala on a Sunday by day runs .25 to .3125: 90° to 112.5°, with
        // 101.25° halfway.
        let at = |moment| {
            upagraha(Point::Kala, DAY, Vara::Ravivara, true, moment, &rising)
                .unwrap()
                .longitude_deg
        };
        assert!((at(Moment::Start) - 90.0).abs() < 1e-9);
        assert!((at(Moment::Middle) - 101.25).abs() < 1e-9);
        assert!((at(Moment::End) - 112.5).abs() < 1e-9);
        assert_eq!(Moment::default(), Moment::Start, "the default of C405");
        let start = lorded(DAY, Vara::Ravivara, true, &rising).unwrap();
        let chosen = lorded_at(DAY, Vara::Ravivara, true, Moment::Start, &rising).unwrap();
        assert_eq!(start, chosen);
    }

    #[test]
    fn a_point_the_day_does_not_divide_is_refused_by_field() {
        for point in [Point::Mandi, Point::Dhuma, Point::InduLagna] {
            let error = upagraha(point, DAY, Vara::ALL[0], true, Moment::Start, &rising)
                .expect_err("not a lord's eighth");
            assert_eq!(error.field(), Some("points.point"), "{point:?}");
            assert!(error.message.contains(point.key()), "{error}");
        }
    }

    #[test]
    fn an_arc_with_no_length_has_no_gulika() {
        let empty = Interval::literal(2_460_000.25, 2_460_000.25);
        let error = saturns(empty, Vara::ALL[0], true).expect_err("a polar day");
        assert_eq!(error.field(), Some("points.arc"));
        assert!(error.message.contains("daylight"), "{error}");
        let night = saturns(empty, Vara::ALL[0], false).expect_err("a polar night");
        assert!(night.message.contains("night"), "{night}");
        assert!(day_division(empty, Vara::ALL[0], true, &rising).is_err());
        // Nor any of the other four.
        let error = lorded(empty, Vara::ALL[0], true, &rising).expect_err("a polar day");
        assert_eq!(error.field(), Some("points.arc"));
    }

    #[test]
    fn an_ascendant_that_refuses_is_reported_unchanged() {
        let refusing = |_: JulianDay<Utc>| {
            Err(
                Error::unsupported(String::from("the provider has no positions there"))
                    .with_field("provider.instant"),
            )
        };
        let error = day_division(DAY, Vara::ALL[0], true, &refusing).expect_err("refused");
        assert_eq!(error.field(), Some("provider.instant"), "unwrapped");
        let error = lorded(DAY, Vara::ALL[0], true, &refusing).expect_err("refused");
        assert_eq!(error.field(), Some("provider.instant"), "unwrapped");
    }
}
