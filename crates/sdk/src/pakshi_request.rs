//! Pancha Pakshi through the façade (`03-design/pakshi.md`): a day's birds
//! at a place from the almanac's sunrise, sunset and tithi, and a native's
//! bird from a chart's Moon.

use teistro_calendar::CalendarDate;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Paksha};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_pakshi::{Bird, BirthBird, Day, Reading, Rules, birth_bird, read_day};
use teistro_serial::Document;
use teistro_time::DayState;

use teistro_core::time::UtcOffset;

use crate::{AlmanacArea, ChartArea};

impl AlmanacArea<'_> {
    /// A native's bird read over one day at a place (`03-design/pakshi.md`):
    /// the ten yamas from the almanac's sunrise, sunset and next sunrise,
    /// each with the bird's activity and its timed sub-periods, the day's
    /// weekday that of its sunrise and its paksha the one running at its
    /// sunrise (cruxes P2, P9), and the day's death bird beside them.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::day`]; `UNSUPPORTED` on a day the Sun does not
    /// both rise and set at the place, which has no yamas.
    pub fn pakshi(
        self,
        date: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        bird: Bird,
        rules: &Rules,
    ) -> Result<Envelope<Reading>, Error> {
        let Envelope { value, provenance } = self.day(date, place, offset)?;
        let local = &value.day;
        if local.state != DayState::Normal {
            return Err(Error::unsupported(
                "the Sun does not both rise and set at this place on this day, so it has no yamas",
            )
            .with_hint(
                "Pancha Pakshi divides a day at its sunrise and sunset; read a day the Sun crosses the horizon",
            ));
        }
        let paksha = value
            .tithi_at(local.sunrise)
            .map(|span| span.member.attributes().paksha)
            .ok_or_else(|| Error::internal("no tithi runs at the day's sunrise"))?;
        let day = Day::new(
            local.sunrise.get(),
            local.sunset.get(),
            local.next_sunrise.get(),
            local.vara,
            paksha,
        )?;
        Ok(Envelope::sealing(read_day(&day, bird, rules), provenance))
    }
}

impl ChartArea<'_> {
    /// A native's bird from a chart (`03-design/pakshi.md` §2): the Moon's
    /// nakshatra, and the paksha by the Moon's elongation from the Sun at
    /// birth, under `rule` (crux P1).
    ///
    /// # Errors
    ///
    /// `INTERNAL` for a document that places no Moon or Sun, or places one
    /// at a longitude that is not finite.
    pub fn pakshi_bird(self, document: &Document, rule: BirthBird) -> Result<Bird, Error> {
        let foundation = &document.foundation;
        let longitude = |graha: Graha| {
            foundation
                .graha(graha)
                .map(|at| at.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))
        };
        let moon = longitude(Graha::Moon)?;
        let nakshatra = Nas::try_from_degrees(moon)?.nakshatra();
        let elongation = (moon - longitude(Graha::Sun)?).rem_euclid(360.0);
        let paksha = if elongation < 180.0 {
            Paksha::Shukla
        } else {
            Paksha::Krishna
        };
        Ok(birth_bird(nakshatra, paksha, rule))
    }
}
