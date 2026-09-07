//! The assembly: every derived point of a founded chart.
//!
//! Each family computes on its own from plain longitudes, so a caller
//! with a Sun and nothing else can have the chain. This is the
//! convenience over them, and the only part that needs a whole chart.

use serde::Serialize;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Point, PointFamily, Rashi, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;

use crate::derived::Derived;
use crate::eighth::{self, Ascendant};
use crate::lagna;
use crate::solar;

/// Every derived point the SDK can compute for one founded chart.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Points {
    found: Vec<Derived>,
}

impl Points {
    /// The points a chart's own longitudes decide, which is everything
    /// but the two the day divides.
    ///
    /// Use this when there is no ascendant to be had — a chart founded
    /// over a provider that cannot answer another instant, or a caller
    /// that does not want Gulika. [`Points::of`] adds the other two.
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
        let mut found = Vec::with_capacity(9);
        found.extend(solar::chain(sun)?);
        found.push(lagna::hora(sun, hours)?);
        found.push(lagna::ghati(sun, hours)?);
        found.push(lagna::pranapada(sun, hours)?);
        found.push(lagna::sree(lagna_deg, moon)?);
        found.push(lagna::yogi(sun, moon)?);
        found.push(lagna::avayogi(sun, moon)?);
        Ok(Points { found })
    }

    /// Every derived point of a founded chart, the two the day divides
    /// included.
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
        Ok(points)
    }

    /// Every point, in the order the families computed them.
    #[must_use]
    pub fn all(&self) -> &[Derived] {
        &self.found
    }

    /// One point, if this chart has it.
    #[must_use]
    pub fn at(&self, point: Point) -> Option<&Derived> {
        self.found.iter().find(|found| found.point == point)
    }

    /// The points of one family.
    pub fn family(&self, family: PointFamily) -> impl Iterator<Item = &Derived> {
        self.found
            .iter()
            .filter(move |found| found.point.attributes().family == family)
    }

    /// The points standing in a sign.
    pub fn in_sign(&self, sign: Rashi) -> impl Iterator<Item = &Derived> {
        self.found.iter().filter(move |found| found.sign == sign)
    }

    /// Which bhava of a founded chart a point falls in, counting
    /// inclusively from the lagna's own sign.
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
        assert!(none.all().is_empty());
        assert_eq!(none.at(Point::Dhuma), None);
        assert_eq!(none.family(PointFamily::UpagrahaSolar).count(), 0);
    }
}
