//! `sdk.almanac`: a panchanga — the five limbs of a day and its periods,
//! for a day or a run of them.

use teistro_astro::precession::PrecessionModel;
use teistro_calendar::CalendarDate;
use teistro_calendar::solar::drik::DrikSun;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::settings::AyanamshaChoice;
use teistro_core::time::UtcOffset;
use teistro_panchanga::almanac::{Almanac, Panchanga};

use crate::area::system_of;
use crate::context::Context;
use crate::ephemeris::no_ephemeris;

/// `sdk.almanac`: what a Nepali or Indian almanac prints — each day's
/// tithi, nakshatra, yoga, karana and vara, and the periods that divide
/// it.
///
/// **A run of days is the shape, and one day is the run of one.** A
/// chart is consulted once; a panchanga every morning. Consecutive days
/// share a boundary — day n's next sunrise is day n+1's sunrise — so a
/// week asked for together costs much less than seven days asked for
/// separately, which is why `of` takes a range.
#[derive(Clone, Copy, Debug)]
pub struct AlmanacArea<'a> {
    context: &'a Context,
}

impl<'a> AlmanacArea<'a> {
    pub(crate) fn of_context(context: &'a Context) -> AlmanacArea<'a> {
        AlmanacArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Every day from one date to another, inclusive, at a place — in
    /// **one crossing**.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a calendar the SDK does not ship, a
    /// range the wrong way round, or the first day the almanac refuses.
    pub fn of(
        self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<Panchanga>>, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let resolved = self.context.resolved();
        let settings = &resolved.settings;
        let calendar = system_of(from.calendar)?;
        let ayanamsha = match settings.frame.ayanamsha {
            AyanamshaChoice::Catalogued { id } => id,
            AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
        };
        let model = DrikSun::new(
            provider,
            ayanamsha,
            settings.day.sunrise,
            settings.provider.overrides,
            self.context.delta_t(),
        );
        // `Almanac::between` seals, as the founder does: the hash of
        // the value is filled where both are known, which is the join.
        Almanac::new(
            provider,
            resolved,
            &model,
            calendar,
            &offset,
            PrecessionModel::default(),
            self.context.delta_t(),
        )
        .between(from, to, place)
    }

    /// One day: the run of one, unwrapped.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`].
    pub fn day(
        self,
        date: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Panchanga>, Error> {
        let Envelope { value, provenance } = self.of(date, date, place, offset)?;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a range of one day answered no panchanga, which cannot happen",
            ));
        };
        // Re-sealed, as `chart().found` is and for the same reason: the
        // hash belongs to the value this envelope holds, and a day is
        // not a range of one.
        Ok(Envelope::sealing(one, provenance))
    }
}
