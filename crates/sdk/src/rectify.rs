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
use teistro_rectification::{
    Answer, Conception, ConceptionRules, ConceptionSky, Day, Rules, Sky, Window, conception, narrow,
};
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
        let middle = JulianDay::try_new(f64::midpoint(window.from.get(), window.to.get()))?;
        let input = RectifyInput {
            window,
            place: *place,
            utc_offset_seconds: offset.seconds(),
            rules,
        };
        self.over_sky(middle, place, offset, &input, |sky| {
            narrow(window, sky, rules)
        })
    }

    /// The reports a candidate birth time gives beside the purifier
    /// (`03-design/rectification.md`, step 4): the pranapada's house as
    /// Jha's print judges the birth, the conception BPHS ch. 3 vv. 25–29
    /// counts back to with its lagna purified "as before", and the birth
    /// Moon *Brihat Jataka* IV.21 reads from that conception, set against
    /// the candidate's own.
    ///
    /// Read at one instant, not over the window's runs: the conception
    /// moves by about a day for every degree its arcs move, so its lagna
    /// turns while the birth moves by minutes; the answer's
    /// `nisheka.count.daysPerBirthMinute` says how fast. None of the three bars.
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{JulianDay, Place};
    /// # use teistro::rectification::ConceptionRules;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let place: Place = todo!();
    /// let at = JulianDay::literal(2_460_000.3);
    /// let read = sdk.chart().conception(at, &place, UtcOffset::UTC, &ConceptionRules::default())?.value;
    /// let w = read.nisheka.count.span.written;
    /// println!("conceived {} months {} days before; its lagna pure: {}", w.months, w.days, read.nisheka.verdict.pure);
    /// println!("the Moon's sign agrees with BJ IV.21: {}", read.moon.sign_agrees);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::rectify`], at the candidate and at its conception.
    pub fn conception(
        self,
        at: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
        rules: &ConceptionRules,
    ) -> Result<Envelope<Conception>, Error> {
        rules.purifier.check()?;
        let input = ConceptionInput {
            at,
            place: *place,
            utc_offset_seconds: offset.seconds(),
            rules,
        };
        self.over_sky(at, place, offset, &input, |sky| {
            conception(sky, at, rules, |conceived| sky.refounded(conceived))
        })
    }

    /// Runs `read` over the sky of the chart founded at `founded_at`,
    /// sealing its answer under that chart's provenance and `input`'s hash.
    fn over_sky<T: Serialize, I: Serialize>(
        self,
        founded_at: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
        input: &I,
        read: impl FnOnce(&ChartSky<'_, '_>) -> Result<T, Error>,
    ) -> Result<Envelope<T>, Error> {
        let reckoning: Reckoning = self
            .context()
            .resolved()
            .settings
            .day
            .ghati_reckoning
            .try_into()?;
        self.founding(offset, |founder| {
            let chart = founder.found_one(founded_at, place, ChartKind::Natal)?;
            let sky = ChartSky {
                founder,
                place: *place,
                zodiac: chart.value.zodiac,
                reckoning,
                luminaries: Cell::new(None),
            };
            let answer = read(&sky)?;
            let mut provenance = chart.provenance;
            provenance.input_hash = content_hash(input);
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

/// What a conception report was asked, which its input hash seals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConceptionInput<'r> {
    at: JulianDay<Utc>,
    place: Place,
    utc_offset_seconds: i32,
    rules: &'r ConceptionRules,
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

impl<'a, 'f> ChartSky<'a, 'f> {
    /// The sky of the chart founded at another instant: its own zodiac, the
    /// same place and reckoning. The conception is a chart of its own.
    fn refounded(&self, at: JulianDay<Utc>) -> Result<ChartSky<'a, 'f>, Error> {
        let chart = self.founder.found_one(at, &self.place, ChartKind::Natal)?;
        Ok(ChartSky {
            founder: self.founder,
            place: self.place,
            zodiac: chart.value.zodiac,
            reckoning: self.reckoning,
            luminaries: Cell::new(None),
        })
    }

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

impl ConceptionSky for ChartSky<'_, '_> {
    fn graha_deg(&self, graha: Graha, at: JulianDay<Utc>) -> Result<f64, Error> {
        let read = self
            .founder
            .longitudes_in(at, &self.place, &self.zodiac, &[graha])?;
        read.first()
            .copied()
            .ok_or_else(|| Error::internal("one graha asked, and none answered"))
    }

    fn midheaven_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(self
            .founder
            .angles_at(at, &self.place, &self.zodiac)?
            .midheaven_deg)
    }
}

/// Thirty ghatis of twenty-four minutes: what proportional reckoning
/// gives the daylight and the night, whatever their length.
const HALF_HOURS: f64 = 12.0;
