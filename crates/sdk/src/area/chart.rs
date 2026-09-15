//! `sdk.chart`: a chart founded from a birth record, and a batch of them
//! founded in one crossing.

use teistro_aspect::Aspects;
use teistro_astro::completion::Completion;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::day::DayPart;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Ayanamsha, ChartKind, DashaSystem, Graha, Rashi, Vara, Varga};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::key::KeyId;
use teistro_core::quantity::Depth;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::AyanamshaChoice;
use teistro_core::settings::Balance;
use teistro_core::time::UtcOffset;
use teistro_dasha::{
    Birth, Dasha, DashaCursor, DashaName, DashaReading, KalachakraDasha, KalachakraRules,
    RashiChart, RashiDasha, RashiRules, Rules as DashaRules,
};
use teistro_geometry::{Layout, draw};
use teistro_houses::Houses;
use teistro_panchanga::limb::{Zodiac as LimbZodiac, nakshatra_at};
use teistro_points::Points;
use teistro_points::arudha::arudha;
use teistro_port_ephemeris::EphemerisProvider;
use teistro_serial::Document;
use teistro_state::state;
use teistro_strength::shadbala::{SAPTAVARGAJA_VARGAS, ShadbalaGraha};
use teistro_strength::{
    AshtakavargaChart, AshtakavargaReading, AshtakavargaRules, BhavaBalaChart, BhavaBalaReading,
    BhavaBalaRules, BhavaGraha, DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading, ShadbalaChart,
    ShadbalaReading, ShadbalaRules, VaiseshikamsaChart, VaiseshikamsaReading, VimshopakaChart,
    VimshopakaReading,
};
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
        if request.sections.has(Sections::ASHTAKAVARGA) {
            document = document.with_ashtakavarga(Self::ashtakavarga_of(foundation, settings)?);
        }
        if request.sections.has(Sections::VIMSHOPAKA) {
            document = document.with_vimshopaka(Self::vimshopaka_of(foundation, settings)?);
        }
        if request.sections.has(Sections::VAISESHIKAMSA) {
            document = document.with_vaiseshikamsa(Self::vaiseshikamsa_of(foundation, settings)?);
        }
        if request.sections.has(Sections::SHADBALA) {
            document = document.with_shadbala(Self::shadbala_of(
                founder,
                foundation,
                settings,
                request.offset(),
            )?);
        }
        if request.sections.has(Sections::BHAVA_BALA) {
            let shadbala = match document.shadbala.clone() {
                Some(reading) => reading,
                None => Self::shadbala_of(founder, foundation, settings, request.offset())?,
            };
            document =
                document.with_bhava_bala(Self::bhava_bala_of(foundation, settings, &shadbala)?);
        }
        if request.sections.has(Sections::DASHA_PHALA) {
            document = document.with_dasha_phala(Self::dasha_phala_of(foundation, settings)?);
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
    fn dasha_reading(self, foundation: &ChartFoundation, id: KeyId) -> Result<DashaReading, Error> {
        let settings = self.context.settings();
        if let Some(definition) = self.context.dashas().by_id(id) {
            let rules = DashaRules::of_definition(&settings.dasha, definition);
            let moon_span = match rules.balance {
                Balance::Temporal => Some(self.moon_span(foundation)?),
                _ => None,
            };
            let dasha = Dasha::new(
                &definition.row(),
                &Self::birth_of(foundation, moon_span)?,
                rules,
            )?;
            return Ok(DashaReading::of_registered(&dasha, definition, moon_span));
        }
        let system = DashaSystem::try_from(id).map_err(|_| self.not_registered(id))?;
        let rules = DashaRules::of(&settings.dasha, system);
        let depth = settings
            .dasha
            .depth
            .get(&system)
            .copied()
            .unwrap_or(Depth::MIN);
        if teistro_dasha::rashi_row(system).is_some() {
            let dasha = self.rashi_dasha_of(foundation, system, rules, RashiRules::of(settings))?;
            return Ok(DashaReading::of_rashi(&dasha, rules, depth));
        }
        let moon_span = match rules.balance {
            Balance::Temporal => Some(self.moon_span(foundation)?),
            _ => None,
        };
        if system == DashaSystem::Kalachakra {
            let dasha = Self::kalachakra_of(foundation, KalachakraRules::of(settings), moon_span)?;
            return Ok(DashaReading::of_kalachakra(&dasha, rules, depth, moon_span));
        }
        let dasha = Self::dasha_of(foundation, system, rules, moon_span)?;
        Ok(DashaReading::of(&dasha, depth, moon_span))
    }

    /// An id that is neither a catalogued dasha system nor one this context
    /// registered, refused with the systems it can compute.
    fn not_registered(self, id: KeyId) -> Error {
        Error::invalid_arg(format!(
            "id {:#010x} is not a dasha system this context knows",
            id.bits()
        ))
        .with_detail(teistro_core::error::Detail::UnknownKey)
        .with_hint(format!(
            "the dashas built are {}",
            self.computable().join(", ")
        ))
    }

    /// Every system this context computes: the catalogued rows, then the
    /// registered ones.
    fn computable(self) -> Vec<String> {
        teistro_dasha::systems()
            .map(|system| system.key().to_owned())
            .chain(
                self.context
                    .dashas()
                    .iter()
                    .map(|(_, definition)| definition.key.clone()),
            )
            .collect()
    }

    /// The system a catalogue names and no row implements, refused with the
    /// systems this build does implement.
    fn not_built(system: DashaSystem) -> Error {
        let built: Vec<&str> = teistro_dasha::systems().map(DashaSystem::key).collect();
        Error::unsupported(format!(
            "{} is a dasha the catalogue names and this build does not compute yet",
            system.key()
        ))
        .with_hint(format!("the dashas built are {}", built.join(", ")))
    }

    /// The birth a nakshatra-seeded dasha reads: the instant, the Moon, and
    /// the Moon's span when the balance is temporal.
    fn birth_of(foundation: &ChartFoundation, moon_span: Option<Interval>) -> Result<Birth, Error> {
        let moon = foundation
            .graha(Graha::Moon)
            .ok_or_else(|| Error::internal("a founded chart places the Moon"))?;
        Ok(Birth {
            instant: foundation.instant,
            moon: Nas::try_from_degrees(moon.longitude_deg)?,
            moon_span,
        })
    }

    /// The nakshatra-seeded dasha of a founded chart, from its Moon.
    fn dasha_of(
        foundation: &ChartFoundation,
        system: DashaSystem,
        rules: DashaRules,
        moon_span: Option<Interval>,
    ) -> Result<Dasha, Error> {
        let row = teistro_dasha::row(system).ok_or_else(|| Self::not_built(system))?;
        Dasha::new(row, &Self::birth_of(foundation, moon_span)?, rules)
    }

    /// The Kalachakra of a founded chart, from its Moon.
    fn kalachakra_of(
        foundation: &ChartFoundation,
        rules: KalachakraRules,
        moon_span: Option<Interval>,
    ) -> Result<KalachakraDasha, Error> {
        KalachakraDasha::new(&Self::birth_of(foundation, moon_span)?, rules)
    }

    /// The sign-based dasha of a founded chart, from the chart a rashi dasha
    /// reads: the lagna's sign, the grahas' signs and dignities under the
    /// settings, the navamsa lagna, and the arudha lagna.
    fn rashi_dasha_of(
        self,
        foundation: &ChartFoundation,
        system: DashaSystem,
        rules: DashaRules,
        rashi: RashiRules,
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
            rashi,
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
    pub fn dasha(
        self,
        document: &Document,
        system: impl Into<DashaName>,
    ) -> Result<DashaCursor, Error> {
        let name = system.into();
        let reading = document
            .dashas
            .iter()
            .find(|reading| reading.system == name)
            .ok_or_else(|| {
                Error::invalid_arg(format!("the document carries no {name} dasha"))
                    .with_field("system")
                    .with_hint("ask for it with ChartRequest::with_dashas")
            })?;
        if let Some(definition) = &reading.definition {
            // A consumer's system rebuilds from the definition the document
            // carries, whatever this context has registered.
            return Dasha::new(
                &definition.row(),
                &Self::birth_of(&document.foundation, reading.moon_span)?,
                reading.rules,
            )
            .map(DashaCursor::Nakshatra);
        }
        let Some(system) = name.catalogued() else {
            return Err(Error::invalid_arg(format!(
                "the document's {name} dasha carries no definition to rebuild it from"
            ))
            .with_field("system"));
        };
        if teistro_dasha::rashi_row(system).is_some() {
            // A document from before the readings were recorded was computed
            // under the recording engine's.
            let rashi = reading.rashi.unwrap_or(RashiRules::RECORDING_ENGINE);
            return self
                .rashi_dasha_of(&document.foundation, system, reading.rules, rashi)
                .map(DashaCursor::Rashi);
        }
        if let Some(rules) = reading.kalachakra {
            return Self::kalachakra_of(&document.foundation, rules, reading.moon_span)
                .map(DashaCursor::Kalachakra);
        }
        Self::dasha_of(
            &document.foundation,
            system,
            reading.rules,
            reading.moon_span,
        )
        .map(DashaCursor::Nakshatra)
    }

    /// The Ashtakavarga of a founded chart under the settings' reading.
    fn ashtakavarga_of(
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
    ) -> Result<AshtakavargaReading, Error> {
        let sign = |index: u8| {
            Rashi::from_id(u16::from(index)).ok_or_else(|| Error::internal("a sign past Pisces"))
        };
        let mut signs = [Rashi::Aries; 7];
        for (slot, graha) in signs.iter_mut().zip(teistro_strength::ashtakavarga::GRAHAS) {
            let position = foundation.graha(graha).ok_or_else(|| {
                Error::internal(format!("a founded chart places {}", graha.key()))
            })?;
            *slot = sign(position.sign_index())?;
        }
        let chart = AshtakavargaChart {
            lagna: sign(foundation.lagna_sign_index())?,
            signs,
        };
        Ok(AshtakavargaReading::of(
            &chart,
            AshtakavargaRules {
                shodhana: settings.strength.shodhana,
                ekadhipatya: settings.strength.ekadhipatya,
            },
        ))
    }

    /// The seven strength grahas' signs in each of `vargas`, Sun to Saturn.
    fn varga_signs<const N: usize, const M: usize>(
        foundation: &ChartFoundation,
        vargas: [Varga; N],
        grahas: [Graha; M],
    ) -> Result<[[Rashi; M]; N], Error> {
        let mut signs = [[Rashi::Aries; M]; N];
        for (row, varga) in signs.iter_mut().zip(vargas) {
            let placed = varga_chart(foundation, Axis::of(varga))?;
            for (slot, graha) in row.iter_mut().zip(grahas) {
                *slot = placed.graha(graha).map(|at| at.sign).ok_or_else(|| {
                    Error::internal(format!("a founded chart places {}", graha.key()))
                })?;
            }
        }
        Ok(signs)
    }

    /// The Vimshopaka of a founded chart under the settings' scoring, from
    /// the seven grahas' signs in the sixteen vargas it reads.
    fn vimshopaka_of(
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
    ) -> Result<VimshopakaReading, Error> {
        let chart = VimshopakaChart {
            signs: Self::varga_signs(
                foundation,
                teistro_strength::vimshopaka::VARGAS,
                teistro_strength::ashtakavarga::GRAHAS,
            )?,
        };
        Ok(VimshopakaReading::of(&chart, settings.strength.vimshopaka))
    }

    /// The dasha phala of a founded chart: the nine grahas' places, dignities
    /// and motions from the state, and their signs in the seven vargas.
    fn dasha_phala_of(
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
    ) -> Result<DashaPhalaReading, Error> {
        let states = state(foundation, settings)?;
        let nine = teistro_strength::bhava_bala::NINE;
        let mut grahas = [DashaPhalaGraha {
            longitude: 0.0,
            house: 1,
            dignity: teistro_core::catalogue::Dignity::Neutral,
            retrograde: false,
        }; 9];
        for (slot, graha) in grahas.iter_mut().zip(nine) {
            let position = foundation
                .grahas
                .iter()
                .find(|p| p.graha == graha)
                .ok_or_else(|| {
                    Error::internal(format!("a founded chart places {}", graha.key()))
                })?;
            let at = states
                .iter()
                .find(|s| s.graha == graha)
                .ok_or_else(|| Error::internal(format!("a state for {}", graha.key())))?;
            *slot = DashaPhalaGraha {
                longitude: position.longitude_deg,
                house: at.house,
                dignity: at.dignity,
                retrograde: at.motion.retrograde,
            };
        }
        let chart = DashaPhalaChart {
            grahas,
            vargas: Self::varga_signs(
                foundation,
                teistro_strength::shadbala::SAPTAVARGAJA_VARGAS,
                nine,
            )?,
        };
        Ok(DashaPhalaReading::of(&chart, settings.dasha.shanta_sign))
    }

    /// The Vaiseshikamsa of a founded chart: its grahas' signs in the sixteen
    /// vargas, the arudha lagna, and which grahas are combust or defeated.
    fn vaiseshikamsa_of(
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
    ) -> Result<VaiseshikamsaReading, Error> {
        let states = state(foundation, settings)?;
        let sign_of = |graha: Graha| {
            states
                .iter()
                .find(|state| state.graha == graha)
                .map_or(Rashi::Aries, |state| state.sign)
        };
        let lagna = Rashi::from_id(u16::from(foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        let mut impaired = [false; 7];
        for (slot, graha) in impaired
            .iter_mut()
            .zip(teistro_strength::ashtakavarga::GRAHAS)
        {
            *slot = states
                .iter()
                .find(|s| s.graha == graha)
                .is_some_and(|s| s.is_combust() || s.lost_its_war() || s.is_shayana());
        }
        let chart = VaiseshikamsaChart {
            signs: VimshopakaChart {
                signs: Self::varga_signs(
                    foundation,
                    teistro_strength::vimshopaka::VARGAS,
                    teistro_strength::ashtakavarga::GRAHAS,
                )?,
            },
            arudha_lagna: arudha(lagna, 1, sign_of).sign,
            impaired,
        };
        Ok(VaiseshikamsaReading::of(&chart))
    }

    /// The Shadbala of a founded chart under the settings' readings.
    ///
    /// Everything but the angles, the obliquity and a sankranti is on the
    /// foundation already: the grahas, the Hindu day and its vara, and the
    /// hora. The angles come from the founder, so the Dig bala measures this
    /// chart's own midheaven; the Mesha sankranti is searched only when the
    /// settings ask for the engine's year lord.
    fn shadbala_of(
        founder: &Founder<'_, dyn EphemerisProvider + '_>,
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
        offset: UtcOffset,
    ) -> Result<ShadbalaReading, Error> {
        let rules = ShadbalaRules::of(settings)?;
        let mut grahas = [ShadbalaGraha {
            longitude: 0.0,
            tropical: 0.0,
            latitude: 0.0,
            house: 1,
        }; 7];
        for (slot, graha) in grahas
            .iter_mut()
            .zip(teistro_strength::ashtakavarga::GRAHAS)
        {
            let at = foundation.graha(graha).ok_or_else(|| {
                Error::internal(format!("a founded chart places {}", graha.key()))
            })?;
            *slot = ShadbalaGraha {
                longitude: at.longitude_deg,
                tropical: at.tropical_deg,
                latitude: at.latitude_deg,
                house: at.house.bhava,
            };
        }
        let angles =
            founder.angles_at(foundation.instant, &foundation.place, &foundation.zodiac)?;
        let day = &foundation.day.day;
        // A civil date by the request's clock: the day's sunrise always falls
        // on its own.
        let days = offset.days();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a Julian day number fits an i64 many times over"
        )]
        let civil = |jd: f64| (jd + days + 0.5).floor() as i64;
        let civil_day = civil(day.sunrise.get());
        let sankranti_lord = if rules.kaala_lords == teistro_core::settings::KaalaLords::Sankranti {
            Some(Self::sankranti_lord(founder, foundation.instant)?)
        } else {
            None
        };
        let chart = ShadbalaChart {
            grahas,
            rahu: foundation.graha(Graha::Rahu).map(|at| at.longitude_deg),
            vargas: Self::varga_signs(
                foundation,
                SAPTAVARGAJA_VARGAS,
                teistro_strength::ashtakavarga::GRAHAS,
            )?,
            instant: foundation.instant.get(),
            sunrise: day.sunrise.get(),
            sunset: day.sunset.get(),
            next_sunrise: day.next_sunrise.get(),
            after_midnight: civil(foundation.instant.get()) > civil_day,
            civil_day,
            weekday_lord: day.vara.attributes().lord,
            hora_lord: foundation.timing.hora.lord,
            sankranti_lord,
            ascendant: angles.ascendant_deg,
            midheaven: angles.midheaven_deg,
            ayanamsha: foundation.zodiac.offset_deg,
            obliquity: angles.obliquity_deg,
        };
        Ok(ShadbalaReading::of(&chart, rules))
    }

    /// The Bhava bala of a founded chart under the settings' readings, from its
    /// own bhavas and the Shadbala already read.
    fn bhava_bala_of(
        foundation: &ChartFoundation,
        settings: &teistro_core::settings::Settings,
        shadbala: &ShadbalaReading,
    ) -> Result<BhavaBalaReading, Error> {
        let mut grahas = [BhavaGraha {
            longitude: 0.0,
            house: 1,
        }; 9];
        for (slot, graha) in grahas.iter_mut().zip(teistro_strength::bhava_bala::NINE) {
            let at = foundation.graha(graha).ok_or_else(|| {
                Error::internal(format!("a founded chart places {}", graha.key()))
            })?;
            *slot = BhavaGraha {
                longitude: at.longitude_deg,
                house: at.house.bhava,
            };
        }
        let day = &foundation.day.day;
        let chart = BhavaBalaChart {
            madhya: foundation.houses.madhya,
            grahas,
            shadbala: BhavaBalaChart::strengths(shadbala),
            instant: foundation.instant.get(),
            day: (day.sunrise.get(), day.sunset.get(), day.next_sunrise.get()),
        };
        Ok(BhavaBalaReading::of(&chart, BhavaBalaRules::of(settings)))
    }

    /// The weekday lord of the last Mesha sankranti at or before an
    /// instant, the weekday taken in UT as the recording engine takes it.
    fn sankranti_lord(
        founder: &Founder<'_, dyn EphemerisProvider + '_>,
        at: JulianDay<Utc>,
    ) -> Result<Graha, Error> {
        use teistro_calendar::solar::sankranti::find_sankranti;
        // A sankranti recurs every sidereal year, so the first one after a
        // year back is at or before the instant, and at most one more is.
        const YEAR: f64 = 366.0;
        const MOST_OF_A_YEAR: f64 = 300.0;
        let model = founder.solar_model();
        let mut found = find_sankranti(model, 0, JulianDay::try_new(at.get() - YEAR)?)?;
        let next = find_sankranti(
            model,
            0,
            JulianDay::try_new(found.instant.get() + MOST_OF_A_YEAR)?,
        )?;
        if next.instant.get() <= at.get() {
            found = next;
        }
        // Julian day 0.5 began a Monday; counted from Sunday it is day 1.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "rem_euclid of 7 is 0 to 6"
        )]
        let weekday = ((found.instant.get() + 1.5).floor().rem_euclid(7.0)) as u8;
        Vara::ALL
            .into_iter()
            .find(|vara| vara.attributes().weekday == weekday)
            .map(|vara| vara.attributes().lord)
            .ok_or_else(|| Error::internal("a weekday past Saturday"))
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
