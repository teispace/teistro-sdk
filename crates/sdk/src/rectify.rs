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
use teistro_rectification::baseline::{self, BaselineAnswer, BaselineRequest};
use teistro_rectification::{
    Answer, Circumstance, CircumstanceRules, Conception, ConceptionRules, ConceptionSky, Day,
    Facts, Rules, Sky, Svarodaya, SvarodayaRun, Window, circumstance, conception, narrow,
    svarodaya, svarodaya_runs,
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

    /// What *Brihat Jataka* ch. V says a candidate birth time shows of the
    /// birth (`03-design/rectification.md`, step 5): the father away
    /// (V.1–2), the child's presentation (V.17), the lamp's oil and wick
    /// (V.18) and the women attending (V.22), each weighed against what
    /// the family remembers. A fact not given is read and not weighed, and
    /// none of them bars.
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{JulianDay, Place};
    /// # use teistro::rectification::{CircumstanceRules, Facts, Presentation};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let place: Place = todo!();
    /// let facts = Facts { father_present: Some(false), presentation: Some(Presentation::Head), ..Facts::default() };
    /// let at = JulianDay::literal(2_460_000.3);
    /// let read = sdk.chart().circumstance(at, &place, UtcOffset::UTC, &facts, &CircumstanceRules::default())?.value;
    /// println!("{} of {} facts agree; the father away: {}", read.agreeing(), read.weights.len(), read.father.away);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, and whatever founding a chart at the
    /// candidate refuses.
    pub fn circumstance(
        self,
        at: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
        facts: &Facts,
        rules: &CircumstanceRules,
    ) -> Result<Envelope<Circumstance>, Error> {
        let input = CircumstanceInput {
            at,
            place: *place,
            utc_offset_seconds: offset.seconds(),
            facts,
            rules,
        };
        self.over_sky(at, place, offset, &input, |sky| {
            circumstance(sky, at, facts, *rules)
        })
    }

    /// The Shiva Svarodaya's nadi and tattva at an instant
    /// (`03-design/rectification.md`, step 7, X12): the nadi the tithi at
    /// sunrise starts (v. 62), its turn of two and a half ghatis (v. 63),
    /// and the tattva flowing in it (vv. 71, 72, 154, 197), with the sex
    /// v. 60 gives the nadi.
    ///
    /// **An application the text does not make**: no verse reads a nadi or
    /// a tattva at a birth, so this is a report, never a bar.
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{JulianDay, Place};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let place: Place = todo!();
    /// let read = sdk.chart().svarodaya(JulianDay::literal(2_460_000.3), &place, UtcOffset::UTC)?.value;
    /// println!("{:?} nadi, {:?}, turn {} of the day", read.run.nadi, read.run.tattva, read.run.turn);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, and whatever founding a chart at the
    /// instant or reckoning its day refuses.
    pub fn svarodaya(
        self,
        at: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Svarodaya>, Error> {
        let input = SvarodayaInput {
            window: None,
            at: Some(at),
            place: *place,
            utc_offset_seconds: offset.seconds(),
        };
        self.over_sky(at, place, offset, &input, |sky| svarodaya(sky, at))
    }

    /// Every run of the Shiva Svarodaya's nadi and tattva inside a birth
    /// window, clipped to it, as [`ChartArea::svarodaya`] reads each:
    /// what a consumer sets beside [`ChartArea::rectify`]'s runs.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::svarodaya`], at every sunrise the window reaches.
    pub fn svarodaya_runs(
        self,
        window: Window,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<SvarodayaRun>>, Error> {
        let input = SvarodayaInput {
            window: Some(window),
            at: None,
            place: *place,
            utc_offset_seconds: offset.seconds(),
        };
        let middle = JulianDay::literal(window.from.get() + window.days() / 2.0);
        self.over_sky(middle, place, offset, &input, |sky| {
            svarodaya_runs(sky, window)
        })
    }

    /// The baseline engine's rectification, reproduced
    /// (`03-design/rectification.md`, step 6): a posterior over a grid of
    /// candidate times, from a prior on the reported time and the tattva
    /// of the child's sex, and from how well each candidate's Vimshottari
    /// periods fit dated life events.
    ///
    /// **Rank 2, and unsourced**: no text gives either stage. It is never
    /// combined with [`ChartArea::rectify`]'s verses, which remove times
    /// where this only ranks them, so a consumer reads the two side by
    /// side. The sky is this context's, so a context under the baseline's
    /// profile reproduces the baseline's charts.
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{JulianDay, Place};
    /// # use teistro::rectification::{BaselineRequest, EventKind, LifeEvent};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let place: Place = todo!();
    /// let mut request = BaselineRequest::around(JulianDay::literal(2_451_779.1354), 60.0);
    /// request.events.push(LifeEvent::on(EventKind::Marriage, JulianDay::literal(2_459_000.5)));
    /// let answer = sdk.chart().rectify_baseline(&place, UtcOffset::literal(5, 45, 0), &request)?.value;
    /// for interval in &answer.intervals {
    ///     println!("{} to {}", interval.from.get(), interval.to.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`BaselineRequest::check`] refuses, a context with no
    /// ephemeris, and whatever founding a chart at a candidate refuses.
    pub fn rectify_baseline(
        self,
        place: &Place,
        offset: UtcOffset,
        request: &BaselineRequest,
    ) -> Result<Envelope<BaselineAnswer>, Error> {
        request.check()?;
        let minutes = offset.seconds().div_euclid(60);
        let input = BaselineInput {
            place: *place,
            utc_offset_seconds: offset.seconds(),
            request,
        };
        self.over_sky(request.reported, place, offset, &input, |sky| {
            baseline::rectify(sky, request, minutes)
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

/// What a baseline rectification was asked, which its input hash seals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BaselineInput<'r> {
    place: Place,
    utc_offset_seconds: i32,
    request: &'r BaselineRequest,
}

/// What a Svarodaya reading was asked, which its input hash seals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SvarodayaInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<Window>,
    #[serde(skip_serializing_if = "Option::is_none")]
    at: Option<JulianDay<Utc>>,
    place: Place,
    utc_offset_seconds: i32,
}

/// What a circumstance report was asked, which its input hash seals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CircumstanceInput<'r> {
    at: JulianDay<Utc>,
    place: Place,
    utc_offset_seconds: i32,
    facts: &'r Facts,
    rules: &'r CircumstanceRules,
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

    fn grahas_deg(&self, grahas: &[Graha], at: JulianDay<Utc>) -> Result<Vec<f64>, Error> {
        let read = self
            .founder
            .longitudes_in(at, &self.place, &self.zodiac, grahas)?;
        if read.len() == grahas.len() {
            Ok(read)
        } else {
            Err(Error::internal("grahas asked, and not as many answered"))
        }
    }
}

/// Thirty ghatis of twenty-four minutes: what proportional reckoning
/// gives the daylight and the night, whatever their length.
const HALF_HOURS: f64 = 12.0;
