//! The time a sankalpa recites (`03-design/sankalpa.md`): the year, the
//! half of the year, the season and the month as the day's sunrise holds
//! them, and the fortnight, the tithi, the nakshatra, the yoga, the
//! karana and the three bodies' signs at the instant of the rite.
//!
//! The parts are facts the almanac already reckons. What a sankalpa adds
//! is **which instant** reads each, and the answer says it: a reader of
//! a sankalpa recites the day's samvatsara, ayana, ritu and masa
//! (*udaya-vyapini*, standing at sunrise) and the rest as they stand when
//! the rite is begun. Every convention a tradition differs on is named
//! in the answer beside the part it decided, and none is chosen here:
//! the month and the season are the profile's (`calendars.lunar_month`,
//! `panchanga.ritu`), the year's name its `calendars.samvatsara`, and the
//! half of the year the request's own [`AyanaReckoning`].
//!
//! The words are not here. The place formula and the Sanskrit
//! inflections are a template in a locale pack, and the parts are what
//! fills it.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{
    Ayana, Karana, Kind, Nakshatra, Paksha, Rashi, Ritu, Tithi, Vara, Yoga, write_in_full,
};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::RituReckoning;

use crate::month::LunarMonth;
use crate::year::LunarYear;

/// How the half of the year is reckoned.
///
/// Almanacs differ: most Indian ones and Nepal's turn the ayana at
/// Makara and Karka sankranti, the sidereal Sun entering Capricorn and
/// Cancer; others, Drik Panchang's "Drik" option among them, at the
/// solstices, the tropical Sun's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AyanaReckoning {
    /// Uttarayana from Makara sankranti, the sidereal Sun's entry into
    /// Capricorn, to Karka sankranti: the almanac's own `sun.ayana`.
    #[default]
    Sidereal,
    /// Uttarayana from the winter solstice to the summer solstice, the
    /// tropical Sun's.
    Tropical,
}

/// The time a sankalpa recites.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Sankalpa {
    /// The instant of the rite.
    pub at: JulianDay<Utc>,
    /// The civil day it falls in: the one whose sunrise it follows and
    /// whose next sunrise it precedes, so a rite before dawn is the day
    /// before's.
    pub date: CalendarDate,
    /// That day's weekday, which runs from its sunrise.
    pub vara: Vara,
    /// That day's sunrise.
    pub sunrise: JulianDay<Utc>,
    /// The next day's sunrise, where the day ends.
    pub next_sunrise: JulianDay<Utc>,
    /// The lunar year the day's sunrise falls in, named by the sixty-year
    /// cycle under `calendars.samvatsara`, with its Vikrama and Shaka
    /// numbers.
    pub year: LunarYear,
    /// The half of the year at the day's sunrise.
    pub ayana: Ayana,
    /// How [`Sankalpa::ayana`] was reckoned: the request's.
    pub ayana_reckoning: AyanaReckoning,
    /// The season at the day's sunrise.
    pub ritu: Ritu,
    /// How [`Sankalpa::ritu`] was reckoned: `panchanga.ritu`.
    pub ritu_reckoning: RituReckoning,
    /// The lunar month at the day's sunrise, under both conventions and
    /// the profile's, and whether it is adhika.
    pub month: LunarMonth,
    /// The fortnight of the tithi at the rite.
    pub paksha: Paksha,
    /// The tithi at the rite.
    pub tithi: Tithi,
    /// The nakshatra at the rite.
    pub nakshatra: Nakshatra,
    /// The yoga at the rite.
    pub yoga: Yoga,
    /// The karana at the rite.
    pub karana: Karana,
    /// The Sun's sidereal sign at the rite.
    pub sun: Rashi,
    /// The Moon's sidereal sign at the rite.
    pub moon: Rashi,
    /// Jupiter's sidereal sign at the rite, the *devaguru* a sankalpa
    /// may name.
    pub jupiter: Rashi,
}

impl Sankalpa {
    /// Where a sankalpa names catalogue members, and of which kind: a
    /// dotted path through it. What the boundary writes in full; the
    /// SDK's tests hold it to serde, both ways.
    pub const MEMBERS: [(&'static str, Kind); 20] = [
        ("date.calendar", Kind::Calendar),
        ("date.era.era", Kind::Era),
        ("vara", Kind::Vara),
        ("year.samvatsara", Kind::Samvatsara),
        ("year.lupta", Kind::Samvatsara),
        ("year.jovian.member", Kind::Samvatsara),
        ("ayana", Kind::Ayana),
        ("ritu", Kind::Ritu),
        ("month.month", Kind::Masa),
        ("month.amanta", Kind::Masa),
        ("month.purnimanta", Kind::Masa),
        ("month.paksha", Kind::Paksha),
        ("paksha", Kind::Paksha),
        ("tithi", Kind::Tithi),
        ("nakshatra", Kind::Nakshatra),
        ("yoga", Kind::Yoga),
        ("karana", Kind::Karana),
        ("sun", Kind::Rashi),
        ("moon", Kind::Rashi),
        ("jupiter", Kind::Rashi),
    ];

    /// The sankalpa as JSON, every catalogue member written as its full
    /// key: what the boundary carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if it does not serialise, which a value this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Error::internal(format!("the sankalpa did not serialise: {error}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// The member of a limb's spans that holds an instant: the span it falls
/// in, its start inclusive.
pub(crate) fn at<T: Copy>(spans: &[crate::span::Span<T>], instant: JulianDay<Utc>) -> Option<T> {
    spans
        .iter()
        .find(|span| span.whole.from.get() <= instant.get() && instant.get() < span.whole.to.get())
        .map(|span| span.member)
}
