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
use teistro_core::house::House;
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
use teistro_tajika::{
    AnnualSky, Between, DrishtiRules, Muntha, MunthaDegree, Natal, OfficeBearers, Panchavargiya,
    Pravesha, Reading, Varshesha, VarsheshaRules, YearCharts, YearYogas,
};
use teistro_vargas::chart::{Axis, chart as varga_chart};

use crate::area::system_of;
use crate::context::Context;
use crate::ephemeris::no_ephemeris;
use crate::reading::{ChartRequest, Sections};
use crate::rule_request::{Longevity, Present, RuleSet, RulesReading};
use crate::rules_bridge::RuleInputs;
use teistro_rules::longevity::{AyurdayaRules, ThreePairsRules};

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

    /// Readings asked to answer a set of rules as well: each document with
    /// the sections the rules read added to the request's own, and what the
    /// chart answers by rule (`03-design/rules-at-the-boundary.md`).
    ///
    /// A birth that cannot have an input only a rule named — the special
    /// lagnas of a day with no sunrise — is read without it rather than
    /// refused, and its reading says so in `unreadable`; a section the request
    /// asked for itself is refused as [`ChartArea::readings`] refuses it.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::readings`], and whatever the rule inputs refuse.
    pub fn readings_with_rules<'r>(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
        set: &'r RuleSet,
    ) -> Result<Envelope<Vec<(Document, RulesReading<'r>)>>, Error> {
        let asked = request.clone().rule_inputs(set.rules(), true);
        let (documents, provenance, unreadable) = match self.readings(instants, &asked) {
            Ok(read) => (read.value, read.provenance, false),
            // Only the points a rule named, never ones the caller asked for.
            Err(error)
                if !request.asks_points()
                    && error
                        .field()
                        .is_some_and(|field| field.starts_with("points")) =>
            {
                let without = request.clone().rule_inputs(set.rules(), false);
                let read = self.readings(instants, &without)?;
                (read.value, read.provenance, true)
            }
            Err(error) => return Err(error),
        };
        let readings = set.readings().readings();
        let mut answered = Vec::with_capacity(documents.len());
        for document in documents {
            let inputs = RuleInputs::of(&document)?;
            let evaluator = inputs.evaluator(readings).with_rules(set.rules());
            let present = set
                .rules()
                .iter()
                .filter_map(|rule| {
                    let result = evaluator.evaluate(rule);
                    result.present.then_some(Present { rule, result })
                })
                .collect();
            let houses = set.houses().then(|| evaluator.house_readings(set.rules()));
            let longevity = set.longevity().then(|| Longevity {
                three_pairs: evaluator.three_pairs(ThreePairsRules::VERSE),
                ayurdaya: evaluator.ayurdaya(AyurdayaRules::default()),
                marakas: evaluator.marakas(),
            });
            let reading = RulesReading {
                present,
                houses,
                longevity,
                unreadable: if unreadable {
                    vec!["points"]
                } else {
                    Vec::new()
                },
            };
            answered.push((document, reading));
        }
        Ok(Envelope::sealing(answered, provenance))
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
            // Which kernel runs a consumer's system is the definition's to
            // say, never guessed from the fields it carries.
            if let Some(rashi) = definition.rashi() {
                let dasha = self.rashi_dasha_of_row(
                    foundation,
                    &rashi.row(),
                    rules,
                    RashiRules::of(settings),
                )?;
                return Ok(DashaReading::of_registered_rashi(&dasha, rashi, rules));
            }
            let Some(udu) = definition.udu() else {
                return Err(Error::internal("a definition names one of the two kernels"));
            };
            let moon_span = match rules.balance {
                Balance::Temporal => Some(self.moon_span(foundation)?),
                _ => None,
            };
            let dasha = Dasha::new(&udu.row(), &Self::birth_of(foundation, moon_span)?, rules)?;
            return Ok(DashaReading::of_registered(&dasha, udu, moon_span));
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
                    .map(|(_, definition)| definition.key().to_owned()),
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
        self.rashi_dasha_of_row(foundation, row, rules, rashi)
    }

    /// The sign-based dasha of a founded chart under a row, shipped or a
    /// consumer's.
    ///
    /// The chart a rashi dasha reads is the same either way — the lagna's
    /// sign, the grahas' signs and dignities under the settings, the navamsa
    /// lagna and the arudha lagna — so it is assembled once here and the
    /// row is the only thing that differs.
    fn rashi_dasha_of_row(
        self,
        foundation: &ChartFoundation,
        row: &teistro_dasha::RashiRow,
        rules: DashaRules,
        rashi: RashiRules,
    ) -> Result<RashiDasha, Error> {
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

    /// The annual charts' instants: the Sun's returns to where it stood at
    /// birth, `1` opening the first year of life
    /// (`03-design/annual-chart.md`).
    ///
    /// Read in the chart's **own** zodiac, on the ayanamsha basis the
    /// settings name — not a frame's sidereal reading, which applies the
    /// mean ayanamsha where a founded chart applies the nutated one and
    /// lands about two degrees of lagna away
    /// (`03-design/annual-chart-measured.md`).
    ///
    /// Fewer instants than asked for is the answer rather than an error:
    /// an ephemeris that ends before a birth's fortieth year has said so.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Document, Ephemeris};
    /// # use teistro::tajika::Reading;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let document: Document = todo!();
    /// let years = sdk.chart().praveshas(&document, Reading::Sidereal, 40)?;
    /// let thirtieth = years.iter().find(|one| one.year == 30);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// No ephemeris; a `through` outside one to two hundred, named
    /// `through`; whatever the provider refuses while searching.
    pub fn praveshas(
        self,
        document: &Document,
        reading: Reading,
        through: u16,
    ) -> Result<Vec<Pravesha>, Error> {
        // The argument is refused before anything is looked up, so a
        // nonsense year is reported as one whichever reading was asked
        // for and whether or not a provider is attached.
        let asked = teistro_tajika::years(through)?;
        let foundation = &document.foundation;
        let natal = Self::natal_of(foundation)?;
        if reading == Reading::Mean {
            return teistro_tajika::mean_praveshas(&natal, asked);
        }
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
        let zodiac = LimbZodiac::of(
            foundation.zodiac.ayanamsha,
            settings.frame.ayanamsha_basis,
            PrecessionModel::default(),
            self.context.delta_t(),
        );
        // Asked for more years than the provider covers, answer the ones
        // it does. Without this the search runs off the end and the
        // provider's own `OutOfRange` comes back naming an instant the
        // caller never mentioned — true, and useless to act on. Fewer
        // instants than asked for is the answer this method documents,
        // and none of them is the answer when the coverage reaches none;
        // the refusal for a nonsense year already happened above, before
        // capping could turn it into an empty list.
        let through = asked.min(Self::years_covered(&completion, foundation.instant.get()));
        if through == 0 {
            return Ok(Vec::new());
        }
        teistro_tajika::praveshas(&longitudes, zodiac, &natal, reading, through)
    }

    /// How many whole returns the provider's coverage holds after an
    /// instant.
    ///
    /// A year short rather than a year long: the last return inside the
    /// coverage is the last one that can be *searched for*, and the search
    /// samples a step past where it expects to find it.
    fn years_covered<P: teistro_port_ephemeris::EphemerisProvider + ?Sized>(
        completion: &Completion<'_, P>,
        birth: f64,
    ) -> u16 {
        let covered = completion.capabilities().jd_range.1;
        let years = (covered - birth - teistro_tajika::STEP_DAYS * 2.0)
            / teistro_tajika::SIDEREAL_YEAR_DAYS;
        if years <= 0.0 {
            return 0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a positive count of years is clamped to u16 below"
        )]
        let years = years.floor().min(f64::from(u16::MAX)) as u16;
        years
    }

    /// One annual chart, founded: the document of the year a return opens.
    ///
    /// The place and the zone are the birth's, which is the default the
    /// schools differ on rather than a rule
    /// (`03-design/annual-chart.md`); a consumer casting for a residence
    /// asks for the instant with [`ChartArea::praveshas`] and founds the
    /// chart themselves, which is the same call this makes.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::praveshas`]; a year the ephemeris does not reach is
    /// refused by `year` with the years it does reach.
    pub fn annual(
        self,
        document: &Document,
        reading: Reading,
        year: u16,
        request: &ChartRequest,
    ) -> Result<Envelope<Document>, Error> {
        let found = self.praveshas(document, reading, year)?;
        let at = found
            .iter()
            .find(|one| one.year == year)
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "the ephemeris does not reach year {year} of this birth"
                ))
                .with_field("year")
                .with_hint(format!(
                    "it reaches {}",
                    found.last().map_or_else(
                        || String::from("none of its years"),
                        |one| format!("year {}", one.year)
                    )
                ))
            })?
            .at;
        self.reading(at, request)
    }

    /// The **Muntha** at one of a birth's returns: the lagna's sign
    /// progressed one sign for each completed year, and that sign's lord.
    ///
    /// `completed_years` is exactly a [`Pravesha::year`], which counts
    /// returns — pass that field in unchanged. Zero is the birth, where
    /// the Muntha sits on the lagna.
    ///
    /// **It needs no ephemeris.** The whole progression is the founded
    /// chart's own lagna and a count of years, so a context with no
    /// provider attached answers it, exactly as [`Reading::Mean`] does
    /// for the returns themselves.
    ///
    /// # Errors
    ///
    /// A `completed_years` past two hundred, named `completed_years`; a
    /// lagna that is not a number, named `lagna`.
    pub fn muntha(
        self,
        document: &Document,
        completed_years: u16,
        degree: MunthaDegree,
    ) -> Result<Muntha, Error> {
        teistro_tajika::muntha(document.foundation.lagna_deg, completed_years, degree)
    }

    /// The annual chart's five **office-bearers**, read from the birth
    /// chart and an annual chart **you founded** — for the birthplace or
    /// for a residence, which is your choice and not the SDK's.
    ///
    /// `completed_years` is the [`Pravesha::year`] the annual chart was
    /// founded for. Three of the five depend on the annual chart's place
    /// (its lagna, and whether the return fell between sunrise and sunset
    /// there), which is why this takes the chart rather than an instant.
    ///
    /// It needs **no ephemeris**: both charts are already founded.
    ///
    /// # Errors
    ///
    /// A `completed_years` past two hundred, named `completed_years`; an
    /// annual chart that places no Sun or no Moon.
    pub fn office_bearers(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
    ) -> Result<OfficeBearers, Error> {
        let year = &annual.foundation;
        let at = |graha: Graha| {
            year.graha(graha)
                .map(|placed| placed.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {graha:?}")))
        };
        teistro_tajika::office_bearers(&YearCharts {
            natal_lagna_deg: natal.foundation.lagna_deg,
            completed_years,
            annual_lagna_deg: year.lagna_deg,
            annual_sun_deg: at(Graha::Sun)?,
            annual_moon_deg: at(Graha::Moon)?,
            by_day: year.day.part.is_daylight(),
        })
    }

    /// The **Panchavargiya bala** of the seven, read from an annual chart
    /// you founded: the five-fold strength the lord of the year is chosen
    /// by.
    ///
    /// Exact, in sub-sub units — the source's own unit, and its worked
    /// chart's last figure — so nothing here rounds and two answers can be
    /// compared as integers.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven.
    pub fn panchavargiya(self, annual: &Document) -> Result<[Panchavargiya; 7], Error> {
        teistro_tajika::panchavargiya(&Self::sky_of(annual)?)
    }

    /// The **Varshesha**, the lord of the year, from a birth chart and an
    /// annual chart you founded.
    ///
    /// One call for the whole chain: the five office-bearers, their
    /// five-fold strengths, and the rule that picks among them — the
    /// strongest that **aspects the annual lagna**, with the source's
    /// fallbacks each a named step. The answer carries every claimant and
    /// which step decided it, because a year lord chosen on strength and
    /// one chosen by a fallback are different statements about the year.
    ///
    /// It needs **no ephemeris**: both charts are already founded.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::office_bearers`] and
    /// [`ChartArea::panchavargiya`]; an annual chart whose lagna is not a
    /// number.
    pub fn varshesha(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        rules: VarsheshaRules,
    ) -> Result<Varshesha, Error> {
        let bearers = self.office_bearers(natal, annual, completed_years)?;
        let strengths = self.panchavargiya(annual)?;
        teistro_tajika::varshesha(&bearers, &strengths, annual.foundation.lagna_deg, rules)
    }

    /// The **Tajika aspects** of an annual chart you founded: every pair
    /// of the seven, with the orb that governs it and whether the two are
    /// coming together or drawing apart.
    ///
    /// Twenty-one pairs. Each carries the sign aspect (the kendras and
    /// the 3, 5, 9, 11 houses; the rest is no aspect at all), the mean of
    /// the two deeptamshas, how far apart they stand **within their
    /// signs** — which is how this tradition counts behind from ahead —
    /// and the **Ithasala** — in one of its three kinds — or **Ishrafa**
    /// that makes, if any.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// [`ChartArea::drishtis_with_rules`] takes the readings the source
    /// leaves open; this is that under [`DrishtiRules::default`].
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven.
    pub fn drishtis(self, annual: &Document) -> Result<Vec<Between>, Error> {
        self.drishtis_with_rules(annual, DrishtiRules::default())
    }

    /// The Tajika aspects under stated readings of the source.
    ///
    /// The source's chapter and its Table X-3 place a pair less than a
    /// degree past differently, and 934 of the 29 166 aspecting pairs
    /// over the corpus's recorded years fall there. [`crate::SubDegree`]
    /// names
    /// the three readings; the default is the table's.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, DrishtiRules, SubDegree};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let rules = DrishtiRules { sub_degree: SubDegree::Ishrafa };
    /// let pairs = sdk.chart().drishtis_with_rules(&annual, rules)?;
    /// let contested = pairs.iter().filter(|pair| pair.disputed()).count();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::drishtis`].
    pub fn drishtis_with_rules(
        self,
        annual: &Document,
        rules: DrishtiRules,
    ) -> Result<Vec<Between>, Error> {
        teistro_tajika::drishtis_with_rules(&Self::sky_of(annual)?, rules)
    }

    /// The **sixteen Tajika yogas** of an annual chart, for one matter.
    ///
    /// Fourteen of the sixteen are judgements about a **pair** — the
    /// *lagnesha*, the lord of the annual lagna, and the *karyesha*, the
    /// lord of the house the matter asked about belongs to — so this
    /// takes the house and not only the chart. "Is the marriage promised
    /// this year?" is `House::try_new(7)`; "what yogas does this year
    /// have?" is not a question these sixteen answer.
    ///
    /// Four are built today. The other twelve are **listed** in
    /// [`YearYogas::unanswered`] at every call, because "Kamboola did not
    /// hold" and "this build cannot tell you about Kamboola" are
    /// different statements; [`YearYogas::holds`] answers `None` for
    /// them rather than `false`.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, House};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let marriage = sdk.chart().tajika_yogas(&annual, House::try_new(7)?)?;
    /// let promised = marriage.holds(teistro::YearYoga::Ithasala);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven, or whose
    /// lagna is not a number.
    pub fn tajika_yogas(self, annual: &Document, house: House) -> Result<YearYogas, Error> {
        self.tajika_yogas_with_rules(annual, house, DrishtiRules::default())
    }

    /// The sixteen Tajika yogas for one matter, under stated readings.
    ///
    /// The readings are the aspects' own ([`crate::SubDegree`]), because
    /// every one of the fourteen pair yogas is built on what the pair is
    /// doing and inherits whatever the caller reads that as.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::tajika_yogas`].
    pub fn tajika_yogas_with_rules(
        self,
        annual: &Document,
        house: House,
        rules: DrishtiRules,
    ) -> Result<YearYogas, Error> {
        teistro_tajika::year_yogas_with_rules(
            annual.foundation.lagna_deg,
            house,
            &Self::sky_of(annual)?,
            rules,
        )
    }

    /// Where the seven stand in a founded chart, which both the strengths
    /// and the aspects read.
    fn sky_of(annual: &Document) -> Result<AnnualSky, Error> {
        let year = &annual.foundation;
        let at = |graha: Graha| {
            year.graha(graha)
                .map(|placed| placed.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {graha:?}")))
        };
        Ok(AnnualSky {
            sun_deg: at(Graha::Sun)?,
            moon_deg: at(Graha::Moon)?,
            mars_deg: at(Graha::Mars)?,
            mercury_deg: at(Graha::Mercury)?,
            jupiter_deg: at(Graha::Jupiter)?,
            venus_deg: at(Graha::Venus)?,
            saturn_deg: at(Graha::Saturn)?,
        })
    }

    /// The Sun where it stood at birth, in both zodiacs, as a return needs
    /// it.
    ///
    /// Both are read off the founded chart rather than one being rebuilt
    /// from the other through the ayanamsha: the chart already answered
    /// that question and a second answer to it is a second thing to get
    /// wrong.
    fn natal_of(foundation: &ChartFoundation) -> Result<Natal, Error> {
        let sun = foundation
            .graha(Graha::Sun)
            .ok_or_else(|| Error::internal("a founded chart places the Sun"))?;
        Ok(Natal {
            instant: foundation.instant,
            sidereal_sun_deg: sun.longitude_deg,
            tropical_sun_deg: sun.tropical_deg,
        })
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
        let zodiac = LimbZodiac::of(
            foundation.zodiac.ayanamsha,
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
            // carries, whatever this context has registered — and from the
            // kernel that definition names, so a sign-based one is not run
            // through the nakshatra-seeded kernel and refused for having no
            // lords.
            if let Some(rashi) = definition.rashi() {
                let rules = reading.rashi.unwrap_or(RashiRules::RECORDING_ENGINE);
                return self
                    .rashi_dasha_of_row(&document.foundation, &rashi.row(), reading.rules, rules)
                    .map(DashaCursor::Rashi);
            }
            let Some(udu) = definition.udu() else {
                return Err(Error::internal("a definition names one of the two kernels"));
            };
            return Dasha::new(
                &udu.row(),
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
