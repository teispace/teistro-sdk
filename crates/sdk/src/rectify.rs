//! Rectification through the façade: a birth window narrowed by the
//! verses that test a birth time, over the sky of the chart the window
//! would found (`03-design/rectification.md`).
//!
//! The kernel reads no ephemeris; this is the [`Sky`] it asks. Every
//! instant is read as a chart founded at it would read it: the lagna by
//! [`Founder::ascendant_at`] in the zodiac of the chart founded at the
//! window's middle, the Sun and the Moon by [`Founder::longitudes_in`] in
//! that zodiac too, the
//! day by [`Founder::day_at`] and the ishtakaal by the settings' ghati
//! reckoning, so a judged instant never reckons a second day (X9).

use std::cell::Cell;

use serde::Serialize;
use teistro_chart::foundation::Founder;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{ChartKind, Graha};
use teistro_core::envelope::{Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::EphemerisProvider;
use teistro_rectification::{Answer, Day, Rules, Sky, Window, narrow};
use teistro_time::ghati::Reckoning;

use crate::area::ChartArea;

impl ChartArea<'_> {
    /// The parts of a birth window that the purifier of BPHS ch. 2
    /// vv. 67–78 leaves standing: a human lagna in a trine of the
    /// pranapada, of Gulika or of the Moon (`03-design/rectification.md`).
    ///
    /// The window is cut wherever a clause changes and each run is judged
    /// once, so the answer is a set of intervals with the clauses that
    /// held in each, never a single minute. The sky is this context's: the
    /// chart's own lagna, zodiac and day at every instant judged.
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{JulianDay, Place};
    /// # use teistro::rectification::{Rules, Window};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let place: Place = todo!();
    /// // "Between half past nine and ten": a half-hour window.
    /// let window = Window::between(JulianDay::literal(2_460_000.0), JulianDay::literal(2_460_000.0 + 0.5 / 24.0))?;
    /// let answer = sdk.chart().rectify(window, &place, UtcOffset::UTC, &Rules::default())?.value;
    /// for run in &answer.intervals {
    ///     let held: Vec<_> = run.verdict.held().map(|clause| clause.purifier).collect();
    ///     println!("{} to {}: {held:?}", run.from.get(), run.to.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`Rules::check`] refuses (a seed step outside a second to an
    /// hour, no purifier allowed), a context with no ephemeris, and
    /// whatever founding a chart at an instant of the window refuses: a
    /// polar day under `UNDEFINED`, an instant the provider cannot place.
    pub fn rectify(
        self,
        window: Window,
        place: &Place,
        offset: UtcOffset,
        rules: &Rules,
    ) -> Result<Envelope<Answer>, Error> {
        rules.check()?;
        let reckoning: Reckoning = self
            .context()
            .resolved()
            .settings
            .day
            .ghati_reckoning
            .try_into()?;
        self.founding(offset, |founder| {
            let middle = JulianDay::try_new(f64::midpoint(window.from.get(), window.to.get()))?;
            let chart = founder.found_one(middle, place, ChartKind::Natal)?;
            let sky = ChartSky {
                founder,
                place: *place,
                zodiac: chart.value.zodiac,
                reckoning,
                luminaries: Cell::new(None),
            };
            let answer = narrow(window, &sky, rules)?;
            let mut provenance = chart.provenance;
            provenance.input_hash = content_hash(&RectifyInput {
                window,
                place: *place,
                utc_offset_seconds: offset.seconds(),
                rules,
            });
            Ok(Envelope::sealing(answer, provenance))
        })
    }
}

/// What a rectification was asked, which its input hash seals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RectifyInput<'r> {
    window: Window,
    place: Place,
    utc_offset_seconds: i32,
    rules: &'r Rules,
}

/// The sky a chart founded at each instant would read.
struct ChartSky<'a, 'f> {
    founder: &'a Founder<'f, dyn EphemerisProvider + 'f>,
    place: Place,
    /// The zodiac of the chart founded at the window's middle: a chart is
    /// measured in one ayanamsha throughout, as `ascendant_at` says.
    zodiac: ChartZodiac,
    /// How the chart counts its ghatis, which the pranapada counts in.
    reckoning: Reckoning,
    /// The Sun and the Moon of the last instant asked, by its bits: the
    /// purifier asks the Sun and then the Moon of one instant, which is one
    /// request.
    luminaries: Cell<Option<(u64, [f64; 2])>>,
}

impl ChartSky<'_, '_> {
    /// The Sun and the Moon at an instant, read once for both.
    fn luminaries(&self, at: JulianDay<Utc>) -> Result<[f64; 2], Error> {
        let bits = at.get().to_bits();
        if let Some((held, both)) = self.luminaries.get()
            && held == bits
        {
            return Ok(both);
        }
        let read = self.founder.longitudes_in(
            at,
            &self.place,
            &self.zodiac,
            &[Graha::Sun, Graha::Moon],
        )?;
        let [sun, moon] = read[..] else {
            return Err(Error::internal("two grahas asked, and not two answered"));
        };
        self.luminaries.set(Some((bits, [sun, moon])));
        Ok([sun, moon])
    }
}

impl Sky for ChartSky<'_, '_> {
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        self.founder.ascendant_at(at, &self.place, &self.zodiac)
    }

    fn sun_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(self.luminaries(at)?[0])
    }

    fn moon_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(self.luminaries(at)?[1])
    }

    fn day(&self, at: JulianDay<Utc>) -> Result<Day, Error> {
        let day = self.founder.day_at(at, &self.place)?.day;
        Ok(Day {
            sunrise: day.sunrise,
            sunset: day.sunset,
            next_sunrise: day.next_sunrise,
            vara: day.vara,
        })
    }

    /// The ishtakaal as `teistro_time::ghati::ghati_pala` counts it, read
    /// continuously rather than snapped to the vipala, so an edge is
    /// pinned below the 0.4 seconds a vipala lasts.
    fn ishtakaal_hours(&self, at: JulianDay<Utc>, day: &Day) -> Result<f64, Error> {
        let elapsed = at.get() - day.sunrise.get();
        Ok(match self.reckoning {
            Reckoning::Civil => elapsed * 24.0,
            Reckoning::Proportional if at.get() < day.sunset.get() => {
                HALF_HOURS * elapsed / (day.sunset.get() - day.sunrise.get())
            }
            Reckoning::Proportional => {
                HALF_HOURS
                    + HALF_HOURS * (at.get() - day.sunset.get())
                        / (day.next_sunrise.get() - day.sunset.get())
            }
        })
    }
}

/// Thirty ghatis of twenty-four minutes: what proportional reckoning
/// gives the daylight and the night, whatever their length.
const HALF_HOURS: f64 = 12.0;
