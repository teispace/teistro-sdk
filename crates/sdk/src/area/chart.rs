//! `sdk.chart`: a chart founded from a birth record, and a batch of them
//! founded in one crossing.

use teistro_aspect::Aspects;
use teistro_astro::completion::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::day::DayPart;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Ayanamsha, ChartKind, DashaSystem, Graha, Rashi, Varga};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::Depth;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::AyanamshaChoice;
use teistro_core::settings::Balance;
use teistro_core::time::UtcOffset;
use teistro_dasha::{
    Birth, Dasha, DashaCursor, DashaReading, RashiChart, RashiDasha, Rules as DashaRules,
};
use teistro_geometry::{Layout, draw};
use teistro_houses::Houses;
use teistro_panchanga::limb::{Zodiac as LimbZodiac, nakshatra_at};
use teistro_points::Points;
use teistro_points::arudha::arudha;
use teistro_port_ephemeris::EphemerisProvider;
use teistro_serial::Document;
use teistro_state::state;
use teistro_vargas::chart::{Axis, chart as varga_chart};

use crate::area::system_of;
use crate::context::Context;
use crate::ephemeris::no_ephemeris;
use crate::reading::{ChartRequest, Sections};

/// `sdk.chart`: the foundation every reading is built on — the lagna,
/// the day's lagna, the ayanamsha applied, the day part, the grahas
/// placed.
///
/// **A batch is the shape, and one chart is the batch unwrapped.** A
/// rectification pass wants a hundred charts at one place, and founding
/// them together resolves the settings once, builds the solar model
/// once, and reckons the day each instant belongs to against the same
/// sunrise. That is the "batch in the signature, not bolted on" rule the
/// maintainer's brief asks for.
#[derive(Clone, Copy, Debug)]
pub struct ChartArea<'a> {
    context: &'a Context,
}

impl<'a> ChartArea<'a> {
    pub(crate) fn of(context: &'a Context) -> ChartArea<'a> {
        ChartArea { context }
    }

    /// A layout this context can draw in, shipped or registered, as its row:
    /// the value to copy, give a key of its own, change and register with
    /// [`ContextBuilder::layout`](crate::ContextBuilder::layout)
    /// (`03-design/chart-geometry.md` §7f). `key` is bare (`NORTH_INDIAN`) or
    /// full (`chart_layout.NORTH_INDIAN`).
    ///
    /// ```
    /// use teistro::{Context, Ephemeris};
    ///
    /// let base = Context::builder().ephemeris([Ephemeris::Test]).build()?;
    /// let mut kerala = base.chart().layout("SOUTH_INDIAN")?;
    /// kerala.key = String::from("ACME_KERALA");
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).layout(kerala).build()?;
    /// assert_eq!(sdk.chart().layout("chart_layout.ACME_KERALA")?.key, "ACME_KERALA");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A key no layout this context knows has, with the keys it knows as the
    /// hint.
    pub fn layout(self, key: &str) -> Result<Layout, Error> {
        let bare = key.strip_prefix("chart_layout.").unwrap_or(key);
        let layouts = self.context.layouts();
        layouts.get(bare).cloned().ok_or_else(|| {
            let known: Vec<&str> = layouts.iter().map(|layout| layout.key.as_str()).collect();
            Error::invalid_arg(format!("`{key}` is not a layout this context knows"))
                .with_field("key")
                .with_hint(format!("the layouts are {}", known.join(", ")))
        })
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Many charts at one place, founded in **one crossing**.
    ///
    /// The envelope's provenance is one stamp over the batch: the
    /// settings, the provider and the steps are the same for every chart
    /// in it, and the input hash is of the whole request.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a civil calendar the SDK does not
    /// ship, or the first instant the founder refuses.
    pub fn found_many(
        self,
        instants: &[JulianDay<Utc>],
        place: &Place,
        offset: UtcOffset,
        kind: ChartKind,
    ) -> Result<Envelope<Vec<ChartFoundation>>, Error> {
        // **Sealed here**, because this is where the value is published:
        // an envelope a consumer holds must carry the hash of its own
        // value, and `Founder::found` leaves the placeholder for its
        // caller to fill. It is not filled *in* the founder for a
        // measured reason — sealing there charged every caller a full
        // canonical serialisation for a field many discard, and the
        // instruction-count gate put `panchanga` 8.8% over its base
        // (`serial-and-the-envelope.md` §8).
        let founded = self.founding(offset, |founder| founder.found(instants, place, kind))?;
        Ok(Envelope::sealing(founded.value, founded.provenance))
    }

    /// Builds the founder this context's settings describe and hands it
    /// to `work`.
    ///
    /// A closure rather than a returned `Founder`, because a founder
    /// borrows the provider, the resolved settings **and a solar model
    /// built here** — three lifetimes, one of them a local. The closure
    /// is what lets `found_many` and `readings` share the set-up instead
    /// of keeping a copy each, which is the second copy this codebase's
    /// rule says to extract.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a civil calendar the SDK does not
    /// ship, or whatever `work` refuses.
    fn founding<T>(
        self,
        offset: UtcOffset,
        work: impl for<'f> FnOnce(&Founder<'f, dyn EphemerisProvider + 'f>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let resolved = self.context.resolved();
        let settings = &resolved.settings;
        let calendar = system_of(settings.calendars.civil_calendar)?;
        // A custom ayanamsha is a value rather than a catalogue member,
        // and the solar model wants a member; Lahiri is what the
        // boundary substitutes, so this substitutes the same.
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
        work(&Founder::new(
            provider,
            resolved,
            &model,
            calendar,
            &offset,
            PrecessionModel::default(),
            self.context.delta_t(),
        ))
    }

    /// One chart: the batch of one, unwrapped.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::found_many`].
    pub fn found(
        self,
        instant: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
        kind: ChartKind,
    ) -> Result<Envelope<ChartFoundation>, Error> {
        let many = self.found_many(&[instant], place, offset, kind)?;
        let Envelope { value, provenance } = many;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a batch of one instant founded no chart, which cannot happen",
            ));
        };
        // **Re-sealed**, because the hash is the hash of *this*
        // envelope's value. The batch's provenance claims the hash of a
        // list of one, and this envelope holds a chart -- so `found` and
        // `found_many([one])` carry different content hashes, which is
        // right: they carry different values.
        Ok(Envelope::sealing(one, provenance))
    }

    /// Many readings at one place, founded in **one crossing** and
    /// assembled from that founding.
    ///
    /// A reading is a founding plus arithmetic: six of the document's
    /// seven sections are a pure function of the foundation and the
    /// settings, so asking for all of them costs one crossing and no
    /// searches (`03-design/chart-reading.md` §2). The exception is the
    /// panchanga, which searches for sunrise, the Moon's rises and the
    /// limbs' boundaries, and is therefore the one section a caller pays
    /// a second crossing for.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::found_many`], plus whatever a section's own
    /// producer refuses — a combustion table the SDK does not ship, a
    /// divisional chart the catalogue does not describe, or a polar day
    /// with no arc for Saturn's eighth to divide.
    pub fn readings(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
    ) -> Result<Envelope<Vec<Document>>, Error> {
        let place = request.place();
        let founded = self.founding(request.offset(), |founder| {
            let founded = founder.found(instants, place, request.kind())?;
            let mut documents = Vec::with_capacity(founded.value.len());
            for foundation in &founded.value {
                documents.push(self.sections_of(founder, foundation, request)?);
            }
            Ok(Envelope::new(documents, founded.provenance))
        })?;
        Ok(Envelope::sealing(founded.value, founded.provenance))
    }

    /// One reading: the batch of one, unwrapped.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::readings`].
    pub fn reading(
        self,
        instant: JulianDay<Utc>,
        request: &ChartRequest,
    ) -> Result<Envelope<Document>, Error> {
        let many = self.readings(&[instant], request)?;
        let Envelope { value, provenance } = many;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a batch of one instant read no document, which cannot happen",
            ));
        };
        Ok(Envelope::sealing(one, provenance))
    }

    /// One foundation, with the sections the request asked for.
    ///
    /// Every section is added through `Document`'s own builder, so a
    /// section this crate learns to compute is one `with_*` call and
    /// nothing else: the document decides what it holds and this decides
    /// what to ask for.
    fn sections_of(
        self,
        founder: &Founder<'_, dyn EphemerisProvider + '_>,
        foundation: &ChartFoundation,
        request: &ChartRequest,
    ) -> Result<Document, Error> {
        let settings = self.context.settings();
        let mut document = Document::of(foundation.clone());
        for varga in request.vargas() {
            document = document.with_varga(varga_chart(foundation, Axis::of(*varga))?);
        }
        if request.sections.has(Sections::STATE) {
            document = document.with_state(state(foundation, settings)?);
        }
        if request.sections.has(Sections::ASPECTS) {
            document = document.with_aspects(Aspects::of(foundation, settings)?);
        }
        if request.sections.has(Sections::HOUSES) {
            document = document.with_houses(Houses::of(foundation)?);
        }
        if request.sections.has(Sections::POINTS) {
            document = document.with_points(Self::points_of(founder, foundation)?);
        }
        for (index, (layout, varga)) in request.drawings().iter().enumerate() {
            let at = format!("drawings[{index}]");
            let row = self.context.layouts().by_id(*layout).ok_or_else(|| {
                Error::invalid_arg(format!(
                    "{layout} is not a layout this context knows; the shipped ones are the \
                     `chart_layout` catalogue, and a consumer's own is given to the builder"
                ))
                .with_field(at.clone())
            })?;
            let drawing = draw(row, foundation, *varga).map_err(|error| error.under(&at))?;
            document = document.with_drawing(drawing);
        }
        for (index, system) in request.dashas().iter().enumerate() {
            let dasha = self
                .dasha_reading(foundation, *system)
                .map_err(|error| error.under(&format!("dashas[{index}]")))?;
            document = document.with_dasha(dasha);
        }
        if request.sections.has(Sections::PANCHANGA) {
            // The day the **chart** belongs to, which before sunrise is
            // not the day of the instant's civil date -- so it is read
            // off the foundation rather than reckoned again from the
            // instant.
            let day = self.context.almanac().day(
                &foundation.day.day.date,
                &foundation.place,
                request.offset(),
            )?;
            document = document.with_panchanga(day.value);
        }
        Ok(document)
    }

    /// One dasha of a chart under the settings' rules, its periods to the
    /// settings' depth: a nakshatra-seeded one with its balance, a
    /// sign-based one with its signs.
    fn dasha_reading(
        self,
        foundation: &ChartFoundation,
        system: DashaSystem,
    ) -> Result<DashaReading, Error> {
        let settings = self.context.settings();
        let rules = DashaRules::of(&settings.dasha, system);
        let depth = settings
            .dasha
            .depth
            .get(&system)
            .copied()
            .unwrap_or(Depth::MIN);
        if teistro_dasha::rashi_row(system).is_some() {
            let dasha = self.rashi_dasha_of(foundation, system, rules)?;
            return Ok(DashaReading::of_rashi(&dasha, rules, depth));
        }
        let moon_span = match rules.balance {
            Balance::Temporal => Some(self.moon_span(foundation)?),
            _ => None,
        };
        let dasha = Self::dasha_of(foundation, system, rules, moon_span)?;
        Ok(DashaReading::of(&dasha, depth, moon_span))
    }

    /// The system a catalogue names and no row implements, refused with the
    /// systems this build does implement.
    fn not_built(system: DashaSystem) -> Error {
        let built: Vec<&str> = teistro_dasha::ROWS
            .iter()
            .map(|row| row.system)
            .chain(teistro_dasha::RASHI_ROWS.iter().map(|row| row.system))
            .map(DashaSystem::key)
            .collect();
        Error::unsupported(format!(
            "{} is a dasha the catalogue names and this build does not compute yet",
            system.key()
        ))
        .with_hint(format!("the dashas built are {}", built.join(", ")))
    }

    /// The nakshatra-seeded dasha of a founded chart, from its Moon.
    fn dasha_of(
        foundation: &ChartFoundation,
        system: DashaSystem,
        rules: DashaRules,
        moon_span: Option<Interval>,
    ) -> Result<Dasha, Error> {
        let row = teistro_dasha::row(system).ok_or_else(|| Self::not_built(system))?;
        let moon = foundation
            .graha(Graha::Moon)
            .ok_or_else(|| Error::internal("a founded chart places the Moon"))?;
        let birth = Birth {
            instant: foundation.instant,
            moon: Nas::try_from_degrees(moon.longitude_deg)?,
            moon_span,
        };
        Dasha::new(row, &birth, rules)
    }

    /// The sign-based dasha of a founded chart, from the chart a rashi dasha
    /// reads: the lagna's sign, the grahas' signs and dignities under the
    /// settings, the navamsa lagna, and the arudha lagna.
    fn rashi_dasha_of(
        self,
        foundation: &ChartFoundation,
        system: DashaSystem,
        rules: DashaRules,
    ) -> Result<RashiDasha, Error> {
        let row = teistro_dasha::rashi_row(system).ok_or_else(|| Self::not_built(system))?;
        let states = state(foundation, self.context.settings())?;
        let grahas = [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
        ];
        let placed = |graha: Graha| {
            states
                .iter()
                .find(|state| state.graha == graha)
                .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))
        };
        let mut signs = [Rashi::Aries; 9];
        let mut dignities = [teistro_core::catalogue::Dignity::Neutral; 9];
        for ((sign, dignity), graha) in signs.iter_mut().zip(dignities.iter_mut()).zip(grahas) {
            let state = placed(graha)?;
            *sign = state.sign;
            *dignity = state.dignity;
        }
        let lagna = Rashi::from_id(u16::from(foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        let navamsa = varga_chart(foundation, Axis::of(Varga::D9))?;
        let sign_of = |graha: Graha| {
            grahas
                .iter()
                .position(|each| *each == graha)
                .and_then(|at| signs.get(at).copied())
                .unwrap_or(lagna)
        };
        let chart = RashiChart {
            lagna,
            arudha_lagna: arudha(lagna, 1, sign_of).sign,
            navamsa_lagna: navamsa.lagna.sign,
            signs,
            dignities,
        };
        RashiDasha::new(
            row,
            &chart,
            foundation.instant,
            rules.year_length,
            rules.after_cycle,
        )
    }

    /// The Moon's stay in its nakshatra around the birth, searched in the
    /// chart's own frame and zodiac: topocentric when the chart is, since a
    /// topocentric Moon can stand a degree from the geocentric one and move
    /// the nakshatra's edge by hours.
    fn moon_span(self, foundation: &ChartFoundation) -> Result<Interval, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let settings = self.context.settings();
        let completion = Completion::new(
            provider,
            settings.provider.overrides,
            self.context.delta_t(),
        );
        let frame = foundation.zodiac.request;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(foundation.place);
        }
        let zodiac = LimbZodiac::of_chart(
            &foundation.zodiac,
            settings.frame.ayanamsha_basis,
            PrecessionModel::default(),
            self.context.delta_t(),
        );
        Ok(nakshatra_at(&longitudes, foundation.instant, zodiac)?.whole)
    }

    /// The cursor behind a document's dasha: to read deeper than the
    /// document's periods, or the chain running at an instant.
    ///
    /// Rebuilt from the document alone — the rules and the Moon's span it
    /// recorded and the foundation — so a stored document gives the same
    /// periods back whatever the context's settings are now. A sign-based
    /// dasha reads the grahas' dignities under this context's settings, which
    /// are the ones that produced the document unless they have changed.
    ///
    /// # Errors
    ///
    /// A system the document carries no dasha of, named `system`.
    pub fn dasha(self, document: &Document, system: DashaSystem) -> Result<DashaCursor, Error> {
        let reading = document
            .dashas
            .iter()
            .find(|reading| reading.system == system)
            .ok_or_else(|| {
                Error::invalid_arg(format!("the document carries no {} dasha", system.key()))
                    .with_field("system")
                    .with_hint("ask for it with ChartRequest::with_dashas")
            })?;
        if teistro_dasha::rashi_row(system).is_some() {
            return self
                .rashi_dasha_of(&document.foundation, system, reading.rules)
                .map(DashaCursor::Rashi);
        }
        Self::dasha_of(
            &document.foundation,
            system,
            reading.rules,
            reading.moon_span,
        )
        .map(DashaCursor::Nakshatra)
    }

    /// The derived points, which are the one section that needs more
    /// than the foundation and the settings.
    ///
    /// Saturn's eighth divides the day's arc and asks for the ascendant
    /// at each division, and the ascendant it must ask for is **this
    /// chart's** — the same house system, the same polar policy, the
    /// same zodiac — which is what `Founder::ascendant_at` answers and
    /// why the founder is threaded this far.
    fn points_of(
        founder: &Founder<'_, dyn EphemerisProvider + '_>,
        foundation: &ChartFoundation,
    ) -> Result<Points, Error> {
        let (from, to) = foundation.day.part_bounds();
        let arc = Interval::new(from, to)?;
        let zodiac = &foundation.zodiac;
        let place = &foundation.place;
        Points::of(
            foundation,
            foundation.day.day.vara,
            arc,
            foundation.day.part == DayPart::Daylight,
            &|at| founder.ascendant_at(at, place, zodiac),
        )
    }
}
