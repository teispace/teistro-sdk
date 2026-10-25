//! The assembly: every derived point of a founded chart.
//!
//! Each family computes on its own from plain longitudes, so a caller
//! with a Sun and nothing else can have the chain. This is the
//! convenience over them, and the only part that needs a whole chart.

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Point, PointFamily, Rashi, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;

use crate::derived::Derived;
use crate::eighth::{self, Ascendant};
use crate::lagna;
use crate::solar;

/// Every derived point the SDK can compute for one founded chart.
///
/// ```
/// use teistro_chart::{day::DayPart, sample};
/// use teistro_core::interval::Interval;
/// use teistro_points::Points;
///
/// let sample_parts = sample::Sample::new();
/// let founder = sample_parts.founder();
/// let foundation = sample::foundation()?;
/// let (from, to) = foundation.day.part_bounds();
/// let points = Points::of(
///     &foundation,
///     foundation.day.day.vara,
///     Interval::new(from, to)?,
///     foundation.day.part == DayPart::Daylight,
///     &|at| founder.ascendant_at(at, &foundation.place, &foundation.zodiac),
/// )?;
/// // The thirteen a chart's longitudes decide, and six the day divides.
/// assert_eq!(points.all().len(), 19);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Points {
    found: Vec<Derived>,
}

impl Points {
    /// The points a chart's own longitudes decide, which is everything
    /// but the six the day divides.
    ///
    /// Use this when there is no ascendant to be had — a chart founded
    /// over a provider that cannot answer another instant, or a caller
    /// that does not want Gulika. [`Points::of`] adds the other six:
    /// Gulika and Mandi, and Kala, Mrityu, Yamaghantaka and Ardhaprahara.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// // No Gulika without an ascendant to read it from.
    /// assert_eq!(points.all().len(), 13);
    /// assert!(points.at(teistro_core::catalogue::Point::Gulika).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a chart whose Sun, Moon or lagna is not a
    /// finite longitude, and `OUT_OF_RANGE` for one whose elapsed time
    /// since sunrise is not a time.
    pub fn from_longitudes(foundation: &ChartFoundation) -> Result<Points, Error> {
        let sun = longitude(foundation, Graha::Sun)?;
        let moon = longitude(foundation, Graha::Moon)?;
        let lagna_deg = foundation.lagna_deg;
        let hours = foundation.timing.ishtakaal.to_hours();
        let mut found = Vec::with_capacity(19);
        found.extend(solar::chain(sun)?);
        found.push(lagna::bhava(sun, hours)?);
        found.push(lagna::hora(sun, hours)?);
        found.push(lagna::ghati(sun, hours)?);
        found.push(lagna::pranapada(sun, hours)?);
        found.push(lagna::sree(lagna_deg, moon)?);
        found.push(lagna::indu(lagna_deg, moon)?);
        found.push(lagna::yogi(sun, moon)?);
        found.push(lagna::avayogi(sun, moon)?);
        Ok(Points { found })
    }

    /// Every derived point of a founded chart, the six the day divides
    /// included.
    ///
    /// Kala, Mrityu, Yamaghantaka and Ardhaprahara are read where their
    /// lords' eighths begin, the default of crux C405;
    /// [`eighth::lorded_at`] reads them at another moment.
    ///
    /// ```
    /// use teistro_chart::{day::DayPart, sample};
    /// use teistro_core::interval::Interval;
    /// use teistro_points::Points;
    ///
    /// let sample_parts = sample::Sample::new();
    /// let founder = sample_parts.founder();
    /// let foundation = sample::foundation()?;
    /// let (from, to) = foundation.day.part_bounds();
    /// let points = Points::of(
    ///     &foundation,
    ///     foundation.day.day.vara,
    ///     Interval::new(from, to)?,
    ///     foundation.day.part == DayPart::Daylight,
    ///     &|at| founder.ascendant_at(at, &foundation.place, &foundation.zodiac),
    /// )?;
    /// use teistro_core::catalogue::Point;
    ///
    /// assert!(points.at(Point::Gulika).is_some() && points.at(Point::Mandi).is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Points::from_longitudes`], plus `UNSUPPORTED` for a day with
    /// no arc to divide — a polar day or night — and whatever the
    /// [`Ascendant`] returns, unchanged.
    pub fn of(
        foundation: &ChartFoundation,
        vara: Vara,
        arc: Interval,
        is_day: bool,
        ascendant: &dyn Ascendant,
    ) -> Result<Points, Error> {
        let mut points = Points::from_longitudes(foundation)?;
        points
            .found
            .extend(eighth::day_division(arc, vara, is_day, ascendant)?);
        points
            .found
            .extend(eighth::lorded(arc, vara, is_day, ascendant)?);
        Ok(points)
    }

    /// Every point, in the order the families computed them.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// assert_eq!(points.all()[0].point, teistro_core::catalogue::Point::Dhuma);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn all(&self) -> &[Derived] {
        &self.found
    }

    /// One point, if this chart has it.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// use teistro_core::catalogue::{Graha, Point};
    /// use teistro_points::lagna;
    ///
    /// let sun = foundation.graha(Graha::Sun).map_or(0.0, |sun| sun.longitude_deg);
    /// assert_eq!(points.at(Point::Dhuma), Some(&teistro_points::solar::chain(sun)?[0]));
    /// assert!(lagna::sign_of(sun).is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn at(&self, point: Point) -> Option<&Derived> {
        self.found.iter().find(|found| found.point == point)
    }

    /// The points of one family.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// use teistro_core::catalogue::PointFamily;
    ///
    /// assert_eq!(points.family(PointFamily::UpagrahaSolar).count(), 5);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn family(&self, family: PointFamily) -> impl Iterator<Item = &Derived> {
        self.found
            .iter()
            .filter(move |found| found.point.attributes().family == family)
    }

    /// The points standing in a sign.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// use teistro_core::catalogue::Rashi;
    ///
    /// // Every point is in one sign or another.
    /// let counted: usize = Rashi::ALL.into_iter().map(|sign| points.in_sign(sign).count()).sum();
    /// assert_eq!(counted, points.all().len());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn in_sign(&self, sign: Rashi) -> impl Iterator<Item = &Derived> {
        self.found.iter().filter(move |found| found.sign == sign)
    }

    /// Which bhava of a founded chart a point falls in, counting
    /// inclusively from the lagna's own sign.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_points::Points;
    ///
    /// let foundation = sample::foundation()?;
    /// let points = Points::from_longitudes(&foundation)?;
    /// use teistro_core::catalogue::Point;
    ///
    /// let house = points.house_of(Point::Yogi, &foundation);
    /// assert!(house.is_some_and(|house| (1..=12).contains(&house)));
    /// // A point the chart does not carry has no house.
    /// assert_eq!(points.house_of(Point::Gulika, &foundation), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn house_of(&self, point: Point, foundation: &ChartFoundation) -> Option<u8> {
        let found = self.at(point)?;
        let lagna = Derived::at(Point::Lagna, foundation.lagna_deg).ok()?;
        let (here, there) = (lagna.sign as u16, found.sign as u16);
        u8::try_from((there + 12 - here) % 12 + 1).ok()
    }
}

/// A graha's longitude in a founded chart.
fn longitude(foundation: &ChartFoundation, graha: Graha) -> Result<f64, Error> {
    foundation
        .graha(graha)
        .map(|position| position.longitude_deg)
        .ok_or_else(|| {
            Error::invalid_arg(format!(
                "a derived point needs {}, which this chart does not place",
                graha.key()
            ))
            .with_field("points.foundation")
        })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and two longitudes computed the same way are equal or the code is wrong"
    )]

    use super::Points;
    use teistro_core::catalogue::{Point, PointFamily};

    #[test]
    fn a_family_holds_what_the_catalogue_says_it_does() {
        // Nothing is assembled here — this is the catalogue's own claim,
        // which `Points::family` reads and this test pins.
        assert_eq!(Point::Dhuma.attributes().family, PointFamily::UpagrahaSolar);
        assert_eq!(Point::Gulika.attributes().family, PointFamily::UpagrahaDay);
        // The four other portions' points sit in Gulika's family, and the
        // Indu lagna among the special lagnas.
        for point in crate::eighth::BESIDE_GULIKA {
            assert_eq!(point.attributes().family, PointFamily::UpagrahaDay);
        }
        assert_eq!(
            Point::InduLagna.attributes().family,
            PointFamily::SpecialLagna
        );
        assert_eq!(
            Point::HoraLagna.attributes().family,
            PointFamily::SpecialLagna
        );
        // The catalogue files the two Yogi points under the sphutas,
        // which is where a reader of the catalogue will look for them
        // even though this crate computes them beside the lagnas.
        assert_eq!(Point::Yogi.attributes().family, PointFamily::Sphuta);
        assert_eq!(Point::Avayogi.attributes().family, PointFamily::Sphuta);
    }

    #[test]
    fn an_empty_set_answers_nothing_rather_than_guessing() {
        let none = Points { found: Vec::new() };
        assert_eq!(none.all(), []);
        assert_eq!(none.at(Point::Dhuma), None);
        assert_eq!(none.family(PointFamily::UpagrahaSolar).count(), 0);
    }
}
