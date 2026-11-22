//! The sighted Hijri months at a place (`docs/03-design/hijri-sighted.md`
//! §3): each month begins on the day after the evening its crescent is
//! seen, and a month whose crescent is not seen on the evening of its 29th
//! completes thirty days.
//!
//! The rule is the one the hadith states: "fast when you see it and
//! break the fast when you see it, and if it is hidden from you, complete
//! the count of thirty" (Sahih al-Bukhari, Kitab al-Sawm, nos. 1906 and
//! 1909; Sahih Muslim, Kitab al-Siyam, no. 1081). A month is so 29 or 30
//! days, and which it is turns on one evening, the 29th's, at the place.
//!
//! A chain needs a first month. The reckoning opens [`DEPTH`] months
//! before the month a span begins in, at the day after the first evening
//! from four days before the tabular month's first on which the crescent
//! is seen, and runs the 29th's rule from there. A first month a day off
//! is drawn back to the sky within a month or two, since the 29th's rule
//! lengthens a month begun early and shortens one begun late; the
//! measured page holds the depth to the months it reaches
//! (`03-design/hijri-sighted-measured.md`).

use std::collections::BTreeMap;

use teistro_astro::crescent::{Crescent, Sighting};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::sky::ApparentPositions;
use teistro_calendar::FixedDay;
use teistro_calendar::hijri::{fixed_from_hijri, hijri_from_fixed};
use teistro_calendar::hijri_sighted::{SightedMonth, SightedMonths};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};

/// How many months before a span's first the chain of months opens.
pub const DEPTH: u32 = 12;

/// How many evenings from four days before a tabular month's first the
/// first month's crescent is looked for.
const FIRST_LOOK: i64 = 6;

/// The sighted months covering every day from `first` to `last` at
/// `place`, the crescent judged by `sighting` over `sky`.
///
/// ```
/// use teistro_astro::crescent::Sighting;
/// use teistro_astro::{Completion, DeltaTModel};
/// use teistro_calendar::gregorian::fixed_from_gregorian;
/// use teistro_core::quantity::Place;
/// use teistro_core::settings::{CrescentCriterion, CrescentZone, OverridePolicy};
/// use teistro_panchanga::hijri_sighted::sighted_months;
/// use teistro_port_ephemeris::TestProvider;
///
/// let provider = TestProvider::new();
/// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// let sighting = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::A)?;
/// let day = fixed_from_gregorian(2025, 3, 31);
/// let months = sighted_months(&sky, kathmandu, DeltaTModel::TableThenModel, sighting, (day, day))?;
/// // One day falls in one month.
/// assert_eq!(months.months().len(), 1);
/// // A span that runs backward is refused.
/// assert!(sighted_months(&sky, kathmandu, DeltaTModel::TableThenModel, sighting, (day, day.plus_days(-1))).is_err());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A span that does not run forward (`INVALID_ARG`), a place where the
/// Sun does not set (`LIMIT`), or the source's refusal.
pub fn sighted_months(
    sky: &dyn ApparentPositions,
    place: Place,
    delta_t: DeltaTModel,
    sighting: Sighting,
    (first, last): (FixedDay, FixedDay),
) -> Result<SightedMonths, Error> {
    if last.get() < first.get() {
        return Err(Error::invalid_arg(format!(
            "a span of days runs forward, not from fixed day {} to {}",
            first.get(),
            last.get()
        )));
    }
    let mut reckoner = Evenings {
        crescent: Crescent::new(sky, place, delta_t),
        sighting,
        longitude: place.longitude.get(),
        seen: BTreeMap::new(),
    };
    let (year, month, _) = hijri_from_fixed(first);
    let (mut year, mut month) = back(year, month, DEPTH);
    let table = fixed_from_hijri(year, month, 1);
    let mut start = table;
    let mut seen = false;
    for offset in 0..FIRST_LOOK {
        let evening = table.plus_days(-4 + offset);
        if reckoner.seen(evening)? {
            start = evening.plus_days(1);
            seen = true;
            break;
        }
    }
    let mut months = Vec::new();
    loop {
        let twenty_ninth = start.plus_days(28);
        let next_seen = reckoner.seen(twenty_ninth)?;
        let next = start.plus_days(if next_seen { 29 } else { 30 });
        if next.get() > first.get() {
            months.push(SightedMonth {
                year,
                month,
                first: start,
                seen,
            });
        }
        if next.get() > last.get() {
            return SightedMonths::new(months, next);
        }
        (year, month) = forward(year, month);
        (start, seen) = (next, next_seen);
    }
}

/// The month `count` months before `year`-`month`.
fn back(year: i32, month: u8, count: u32) -> (i32, u8) {
    let index = i64::from(year) * 12 + i64::from(month) - 1 - i64::from(count);
    let (year, month) = (index.div_euclid(12), index.rem_euclid(12) + 1);
    (
        i32::try_from(year).unwrap_or(i32::MIN),
        u8::try_from(month).unwrap_or(1),
    )
}

/// The month after `year`-`month`.
const fn forward(year: i32, month: u8) -> (i32, u8) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

/// The evenings read so far, each read once.
struct Evenings<'a> {
    crescent: Crescent<'a>,
    sighting: Sighting,
    longitude: f64,
    seen: BTreeMap<i64, bool>,
}

impl Evenings<'_> {
    /// Whether the crescent is seen on the evening of civil day `day`:
    /// after the first sunset following the day's local mean noon.
    fn seen(&mut self, day: FixedDay) -> Result<bool, Error> {
        if let Some(seen) = self.seen.get(&day.get()) {
            return Ok(*seen);
        }
        let midnight = day
            .jd_at_midnight()
            .map_err(|why| Error::invalid_arg(why.to_string()))?;
        let noon = JulianDay::<Ut1>::try_new(midnight.get() + 0.5 - self.longitude / 360.0)
            .map_err(|why| Error::invalid_arg(why.to_string()))?;
        let seen = self.sighting.seen(&self.crescent.evening(noon)?);
        self.seen.insert(day.get(), seen);
        Ok(seen)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn months_count_back_and_forward_across_years() {
        assert_eq!(back(1446, 10, 12), (1445, 10));
        assert_eq!(back(1446, 1, 1), (1445, 12));
        assert_eq!(forward(1445, 12), (1446, 1));
        assert_eq!(forward(1446, 3), (1446, 4));
    }
}
