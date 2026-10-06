//! `sdk.chart`: a chart founded from a birth record, and a batch of them
//! founded in one crossing.

use teistro_aspect::Aspects;
use teistro_astro::completion::Completion;
use teistro_astro::events::{FrameLongitudes, StationKind};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_astro::sky::{Spherical, altitude_by_midheaven_deg};
use teistro_astro::stars::{Options as StarOptions, place_of};
use teistro_chart::day::DayPart;
use teistro_chart::foundation::{
    ChartAngles, ChartFoundation, Founder, TransitEventKind, angles_of, cusps_of, cusps_raising,
};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{
    CharaKaraka, ChartKind, DashaSystem, Graha, HouseSystem, Nakshatra, Rashi, Star, Vara, Varga,
};
use teistro_core::envelope::{Envelope, Hash};
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_core::interval::Interval;
use teistro_core::key::KeyId;
use teistro_core::quantity::Depth;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::Balance;
use teistro_core::settings::{CharaKarakas, DayLordDay, DecennialDivision, ReleasingSharedSign};
use teistro_core::time::UtcOffset;
use teistro_dasha::jaimini::{
    JaiminiReading, brahma, graha_arudhas, karakamsha, pada_lord, pada_lords,
};
use teistro_dasha::rashi::{Direction, step};
use teistro_dasha::{
    Birth, Dasha, DashaCursor, DashaName, DashaReading, DecennialDasha, FirdariaDasha,
    KalachakraDasha, KalachakraRules, ProfectionDasha, RashiChart, RashiDasha, RashiRules,
    Rules as DashaRules, Wheel, YearDasha, YearRing,
};
use teistro_geometry::{Layout, draw};
use teistro_hellenistic::{
    AccidentalSky, CHALDEAN_ORDER, ChartSky, ConsiderationRules, Considerations, Dignities,
    DignityRequest, FortitudeRequest, Fortitudes, Lot, LotFormula, LotPlace, LotPoint, LotReading,
    LotRequest, LotSky, Matter, PerfectionRequest, Sect, considerations,
};
use teistro_houses::Houses;
use teistro_houses::system::override_of;
use teistro_kp::{
    KpChart, KpNumber, KpReading, Position, RulingPlanets, RulingRules, Significators,
};
use teistro_panchanga::limb::{Zodiac as LimbZodiac, moon_between, nakshatra_at};
use teistro_points::Points;
use teistro_points::arudha::arudha_by;
use teistro_port_ephemeris::EphemerisProvider;
use teistro_serial::Document;
use teistro_state::{GrahaState, state};
use teistro_strength::shadbala::{SAPTAVARGAJA_VARGAS, ShadbalaGraha};
use teistro_strength::{
    AshtakavargaChart, AshtakavargaReading, AshtakavargaRules, BhavaBalaChart, BhavaBalaReading,
    BhavaBalaRules, BhavaGraha, DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading, ShadbalaChart,
    ShadbalaReading, ShadbalaRules, VaiseshikamsaChart, VaiseshikamsaReading, VimshopakaChart,
    VimshopakaReading,
};
use teistro_tajika::{
    Affliction, AnnualDasha, AnnualDashaRules, AnnualSky, AnnualStates, Between, DrishtiRules,
    Favour, Harsha, HarshaRules, MuddaBalance, Muntha, MunthaDegree, Natal, OfficeBearers,
    Panchavargiya, Pravesha, Qualification, Reading, SEVEN, Saham, SahamFormula, SahamPlace,
    SahamReading, SahamRules, SahamSky, SahamStrength, SahamStrengthRules, Strength, Varshesha,
    VarsheshaRules, YearCharts, YearYogas, YogaRules,
};
use teistro_vargas::chart::{Axis, chart as varga_chart};

use crate::area::{Answers, Plans};
use crate::area::{drik_sun, system_of};
use crate::context::Context;
use crate::ephemeris::no_ephemeris;
use crate::kp_request::KpRequest;
use crate::plan_request::{PlanInputs, PlanRequest};
use crate::reading::{ChartRequest, Sections};
use crate::rule_request::{Longevity, Present, RuleSet, RulesReading};
use crate::rules_bridge::RuleInputs;
use crate::varsha::{AnnualChart, AnnualPlace, VARSHA, Varsha, VarshaRequest, VarshaYear};
use teistro_rules::longevity::{Computed, dasayus, span_choice};

/// The grahas as ch. 46's ladder reads them, each one's sign and dignity, in
/// a chart whose lagna is `lagna`, with its arudha lagna counted to each
/// sign's pada lord under the co-lordship (crux C135): the one assembly the
/// sign dashas, the Vaiseshikamsa and the rules' padas all read. The
/// navamsa lagna and Brahma are the dasha reading's to fill.
fn placed_chart(
    lagna: Rashi,
    states: &[GrahaState],
    co_lordship: teistro_core::settings::NodeCoLordship,
) -> Result<RashiChart, Error> {
    let mut signs = [lagna; 9];
    let mut dignities = [teistro_core::catalogue::Dignity::Neutral; 9];
    for ((sign, dignity), graha) in signs
        .iter_mut()
        .zip(dignities.iter_mut())
        .zip(GRAHAS_IN_ORDER)
    {
        let placed = states
            .iter()
            .find(|state| state.graha == graha)
            .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))?;
        *sign = placed.sign;
        *dignity = placed.dignity;
    }
    let mut chart = RashiChart {
        lagna,
        arudha_lagna: lagna,
        navamsa_lagna: lagna,
        signs,
        dignities,
        brahma: None,
    };
    let sign_of = |graha: Graha| {
        GRAHAS_IN_ORDER
            .iter()
            .position(|each| *each == graha)
            .and_then(|at| signs.get(at).copied())
            .unwrap_or(lagna)
    };
    let arudha_lagna = arudha_by(
        lagna,
        1,
        |sign| pada_lord(&chart, sign, co_lordship),
        sign_of,
    )
    .sign;
    chart.arudha_lagna = arudha_lagna;
    Ok(chart)
}

/// Each graha's degrees within its sign, the Sun to Ketu: what the
/// Brahma graha is weighed by (BPHS ch. 46 v. 173).
///
/// # Errors
///
/// A founded chart that does not place one of the nine, named.
fn degrees_in_sign(foundation: &ChartFoundation) -> Result<[f64; 9], Error> {
    let mut out = [0.0; 9];
    for (slot, graha) in out.iter_mut().zip(GRAHAS_IN_ORDER) {
        *slot = foundation
            .graha(graha)
            .map(|at| at.longitude_deg.rem_euclid(30.0))
            .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))?;
    }
    Ok(out)
}

/// The furthest [`ChartArea::perfection`] searches ahead, days: ten
/// years, which a horizon set by a significator standing nearly still in
/// its sign would otherwise exceed without end.
const PERFECTION_HORIZON_CAP_DAYS: f64 = 3653.0;

/// The nine grahas a chart places, in the catalogue's order: the Sun to
/// Ketu, without the outer planets the catalogue also names.
const GRAHAS_IN_ORDER: [Graha; 9] = [
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
        let founded = self.founded(instants, place, offset, kind)?;
        Ok(Envelope::sealing(founded.value, founded.provenance))
    }

    /// The charts founded, **not yet sealed**: every public call seals
    /// once, over the value it publishes, and none pays for a hash a
    /// second call would replace.
    fn founded(
        self,
        instants: &[JulianDay<Utc>],
        place: &Place,
        offset: UtcOffset,
        kind: ChartKind,
    ) -> Result<Envelope<Vec<ChartFoundation>>, Error> {
        self.founding(offset, |founder| founder.found(instants, place, kind))
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
        let calendar = system_of(resolved.settings.calendars.civil_calendar)?;
        let model = drik_sun(self.context, provider);
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
        let Envelope { value, provenance } = self.founded(&[instant], place, offset, kind)?;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a batch of one instant founded no chart, which cannot happen",
            ));
        };
        // Sealed over the chart, because the hash is the hash of *this*
        // envelope's value: `found` and `found_many([one])` carry
        // different content hashes, which is right, since they carry
        // different values — and the list of one is never hashed at all.
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
        let read = self.read(instants, request)?;
        Ok(Envelope::sealing(read.value, read.provenance))
    }

    /// The readings, **not yet sealed**, as [`ChartArea::founded`].
    fn read(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
    ) -> Result<Envelope<Vec<Document>>, Error> {
        let place = request.place();
        self.founding(request.offset(), |founder| {
            let founded =
                founder.found_with(instants, place, request.kind(), request.outer_planets())?;
            let mut documents = Vec::with_capacity(founded.value.len());
            for foundation in &founded.value {
                documents.push(self.sections_of(founder, foundation, request)?);
            }
            Ok(Envelope::new(documents, founded.provenance))
        })
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
        let answered = self.answered(instants, request, set)?;
        Ok(Envelope::sealing(answered.value, answered.provenance))
    }

    /// The readings with what they answer by rule, **not yet sealed**, as
    /// [`ChartArea::founded`].
    fn answered<'r>(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
        set: &'r RuleSet,
    ) -> Result<Envelope<Vec<(Document, RulesReading<'r>)>>, Error> {
        // The longevity readings weigh strengths: the visible half's
        // strongest of several, and v. 33's strongest of the seven and the
        // lagna, whose strength is its bhava's. Dasayus reads Vimshottari's
        // balance at birth, as the settings measure it.
        let vimshottari = KeyId::from(DashaSystem::Vimshottari);
        let inputs = |points: bool| {
            let asked = request.clone().rule_inputs(set.rules(), points);
            if !set.longevity() {
                return asked;
            }
            let mut dashas = asked.dashas().to_vec();
            if !dashas.contains(&vimshottari) {
                dashas.push(vimshottari);
            }
            asked.with_shadbala().with_bhava_bala().with_dashas(dashas)
        };
        let (documents, provenance, unreadable) = match self.read(instants, &inputs(true)) {
            Ok(read) => (read.value, read.provenance, false),
            // Only the points a rule named, never ones the caller asked for.
            Err(error)
                if !request.asks_points()
                    && error
                        .field()
                        .is_some_and(|field| field.starts_with("points")) =>
            {
                let read = self.read(instants, &inputs(false))?;
                (read.value, read.provenance, true)
            }
            Err(error) => return Err(error),
        };
        let readings = set.readings().readings();
        let mut answered = Vec::with_capacity(documents.len());
        for document in documents {
            let inputs = RuleInputs::of(&document)?;
            let lords = self.pada_lords_of(&document)?;
            let evaluator = inputs
                .evaluator(readings)
                .with_rules(set.rules())
                .with_pada_lords(&lords);
            let present = set
                .rules()
                .iter()
                .filter_map(|rule| {
                    let result = evaluator.evaluate(rule);
                    result.present.then_some(Present { rule, result })
                })
                .collect();
            let houses = set.houses().then(|| evaluator.house_readings(set.rules()));
            let longevity = set.longevity_rules().map(|rules| {
                let ayurdaya = evaluator.ayurdaya(rules.ayurdaya);
                let rasmi = evaluator.rasmi(rules.rasmi);
                // Bhava bala in rupas, the unit of the grahas' Shadbala.
                let lagna = document
                    .bhava_bala
                    .as_ref()
                    .and_then(|bala| bala.bhavas.first())
                    .map(|first| first.virupas / 60.0);
                let dasayus = document
                    .dashas
                    .iter()
                    .find(|reading| {
                        reading.system == DashaName::Catalogued(DashaSystem::Vimshottari)
                    })
                    .and_then(|reading| dasayus(reading.first_lord, reading.balance?.remaining));
                let chakrayus = evaluator.chakrayus(rules.chakrayus);
                let computed = Computed {
                    ayurdaya: Some(&ayurdaya),
                    rasmi: Some(&rasmi),
                    dasayus: dasayus.as_ref(),
                    chakrayus: Some(&chakrayus),
                };
                Longevity {
                    three_pairs: evaluator.three_pairs(rules.three_pairs),
                    choice: span_choice(&evaluator.strengths(), lagna, computed),
                    ayurdaya,
                    rasmi,
                    dasayus,
                    chakrayus,
                    marakas: evaluator.marakas(),
                }
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
        Ok(Envelope::new(answered, provenance))
    }

    /// Charts founded, read and **said**, in one call: each chart's
    /// document, what it answers by rule when `inputs` names rules, its
    /// Sade Sati report when `inputs` names a window, and the plans
    /// `asked` names (`03-design/plans-at-the-boundary.md`).
    ///
    /// `inputs` is what is asked beside the charts; a rule set, or
    /// `Option<&RuleSet>`, converts into one. Each is answered **once for
    /// the batch** — the rules evaluated over every chart, Saturn scanned
    /// once for every chart's periods — and what it answered is on each
    /// chart as well as said by the plans that say it.
    ///
    /// The sections the composers read are computed whether or not the
    /// request named them ([`PlanRequest::sections`]), because a consumer
    /// asks for the plan and not for the knob underneath it. This is the
    /// call every binding's `interpret` option makes, so a plan is composed
    /// in one place for every language.
    ///
    /// # Errors
    ///
    /// `readings` asked for without rules and `sadeSati` without a window
    /// ([`PlanRequest::check`]); as [`ChartArea::readings`],
    /// [`ChartArea::readings_with_rules`] and [`ChartArea::sade_sati_many`].
    pub fn interpreted<'r>(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
        inputs: impl Into<PlanInputs<'r>>,
        asked: PlanRequest,
    ) -> Result<Envelope<Vec<Interpreted<'r>>>, Error> {
        let inputs = inputs.into();
        let rules = inputs.rules;
        // Named from the record every binding calls `interpret`, so a
        // refusal reads the same in Rust as in the language that wrote it.
        asked
            .check(&inputs)
            .map_err(|error| error.under("interpret"))?;
        let wanted = asked.sections(request.clone());
        // Sealed once, over what the batch publishes — the documents, and
        // what each answers by rule where rules were asked — with each
        // chart's own hash beside the batch's, from the same serialisation,
        // so a caller handing out one chart stamps it with its own.
        let (read, provenance): (Vec<SealedChart<'r>>, _) = match rules {
            None => {
                let Envelope { value, provenance } = self.read(instants, &wanted)?;
                let (read, each) = Envelope::sealing_each(value, provenance);
                let Envelope { value, provenance } = read;
                (
                    value
                        .into_iter()
                        .zip(each)
                        .map(|(document, hash)| (document, None, hash))
                        .collect(),
                    provenance,
                )
            }
            Some(set) => {
                let Envelope { value, provenance } = self.answered(instants, &wanted, set)?;
                let (read, each) = Envelope::sealing_each(value, provenance);
                let Envelope { value, provenance } = read;
                (
                    value
                        .into_iter()
                        .zip(each)
                        .map(|((document, reading), hash)| (document, Some(reading), hash))
                        .collect(),
                    provenance,
                )
            }
        };
        // One scan of Saturn for every chart, and none for a batch of none,
        // which `sade_sati_many` would refuse by a field this call has not.
        let mut periods = match inputs.sade_sati {
            Some(window) if !read.is_empty() => self
                .sade_sati_many(read.iter().map(|(document, _, _)| document), window)?
                .value
                .into_iter()
                .map(Some)
                .collect(),
            _ => Vec::new(),
        }
        .into_iter();
        let interpret = self.context.interpret();
        let mut charts = Vec::with_capacity(read.len());
        for (document, reading, content_hash) in read {
            let sade_sati = periods.next().flatten();
            let mut answers = Answers::none();
            if let Some(reading) = reading.as_ref() {
                answers = answers.with_reading(reading);
            }
            if let Some(report) = sade_sati.as_ref() {
                answers = answers.with_sade_sati(report);
            }
            let plans = interpret.plans(&document, answers, asked)?;
            charts.push(Interpreted {
                document,
                reading,
                sade_sati,
                plans,
                content_hash,
            });
        }
        Ok(Envelope::new(charts, provenance))
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
        let Envelope { value, provenance } = self.read(&[instant], request)?;
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
        if request.sections.has(Sections::JAIMINI) {
            document = document.with_jaimini(self.jaimini_of(foundation)?);
        }
        if request.sections.has(Sections::AVAKAHADA) {
            document = document.with_avakahada(crate::matching_chart::avakahada_of(foundation)?);
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
                .dasha_reading(&document, *system, request.lot_rules())
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
    /// sign-based one with its signs, a time lord from the lots read under
    /// `lots`.
    fn dasha_reading(
        self,
        chart: &Document,
        id: KeyId,
        lots: LotRequest,
    ) -> Result<DashaReading, Error> {
        let foundation = &chart.foundation;
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
                Balance::Temporal => Some(self.moon_span(foundation, udu.wheel)?),
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
        if let Some(cursor) = self.time_lord_of(chart, system, rules, lots)? {
            // A year is not divided, so a profection carries one level; a
            // firdar is divided once, into sevenths.
            let depth = match cursor {
                DashaCursor::Profection(_) => Depth::MIN,
                DashaCursor::Firdaria(_) => Depth::try_new(2).map_or(depth, |two| depth.min(two)),
                _ => depth,
            };
            return Ok(DashaReading::of_time_lord(system, &cursor, rules, depth));
        }
        let temporal = rules.balance == Balance::Temporal;
        if system == DashaSystem::Kalachakra {
            let moon_span = if temporal {
                Some(self.moon_span(foundation, Wheel::Nakshatras)?)
            } else {
                None
            };
            let dasha = Self::kalachakra_of(foundation, KalachakraRules::of(settings), moon_span)?;
            return Ok(DashaReading::of_kalachakra(&dasha, rules, depth, moon_span));
        }
        let row = teistro_dasha::row(system, rules.ashtottari_grouping)
            .ok_or_else(|| Self::not_built(system))?;
        // A system counted over its own wheel is read across its own
        // segment, Abhijit's included, not across the nakshatra.
        let moon_span = if temporal {
            Some(self.moon_span(foundation, row.wheel)?)
        } else {
            None
        };
        let dasha = Dasha::new(row, &Self::birth_of(foundation, moon_span)?, rules)?;
        Ok(DashaReading::of(&dasha, depth, moon_span))
    }

    /// A Hellenistic time lord of a chart, begun from the sign its system
    /// names (`03-design/hellenistic-time-lords.md`); nothing for a system
    /// of another kernel.
    fn time_lord_of(
        self,
        chart: &Document,
        system: DashaSystem,
        rules: DashaRules,
        lots: LotRequest,
    ) -> Result<Option<DashaCursor>, Error> {
        if system == DashaSystem::Firdaria {
            // The Sun begins a day birth's firdaria and the Moon a night
            // one's (`03-design/hellenistic-firdaria.md`).
            return FirdariaDasha::new(
                self.sect_luminary(chart, lots)?,
                chart.foundation.instant,
                rules.year_length,
                self.context.settings().dasha.firdaria_nodes,
            )
            .map(|dasha| Some(DashaCursor::Firdaria(dasha)));
        }
        if system == DashaSystem::Decennials {
            // The luminary of the sect is the apheta (VI.5,
            // `03-design/hellenistic-decennials.md`).
            let order = Self::decennial_order_of(chart, self.sect_luminary(chart, lots)?)?;
            return Self::decennials_in(chart, order, rules, self.decennial_division())
                .map(|dasha| Some(DashaCursor::Decennials(dasha)));
        }
        if !teistro_dasha::TIME_LORDS.contains(&system) {
            return Ok(None);
        }
        let start = self.time_lord_start(chart, system, lots)?;
        teistro_dasha::time_lord(system, start, chart.foundation.instant, rules.year_length)
            .transpose()
    }

    /// The luminary of a chart's sect, read under the request's lot rules
    /// as the lots are: the Sun by day, the Moon by night.
    fn sect_luminary(self, chart: &Document, lots: LotRequest) -> Result<Graha, Error> {
        Ok(match self.lots_with_request(chart, &[], lots)?.sect {
            Sect::Day => Graha::Sun,
            Sect::Night => Graha::Moon,
        })
    }

    /// The seven in the order the decennials run from `apheta`, by their
    /// longitudes on the chart.
    fn decennial_order_of(
        chart: &Document,
        apheta: Graha,
    ) -> Result<[Graha; teistro_dasha::DECENNIAL_STARS], Error> {
        let places = chart
            .foundation
            .grahas
            .iter()
            .filter(|position| CHALDEAN_ORDER.contains(&position.graha))
            .map(|position| (position.graha, position.longitude_deg))
            .collect::<Vec<_>>();
        teistro_dasha::decennial_order(apheta, &places)
    }

    /// The decennials of a chart in `order`, under `rules`' year.
    fn decennials_in(
        chart: &Document,
        order: [Graha; teistro_dasha::DECENNIAL_STARS],
        rules: DashaRules,
        division: DecennialDivision,
    ) -> Result<DecennialDasha, Error> {
        DecennialDasha::new(order, chart.foundation.instant, rules.year_length, division)
    }

    /// How the settings divide the decennials below the second level.
    fn decennial_division(self) -> DecennialDivision {
        self.context.settings().dasha.decennial_division
    }

    /// The sign a time lord starts from: the Ascendant's for the profected
    /// year (IV.11); the Lot of Fortune's or Daimon's for releasing, read
    /// under `lots`, and, when the two lots share a sign, the one the
    /// settings' `releasing_shared_sign` names (IV.4, crux C223).
    fn time_lord_start(
        self,
        chart: &Document,
        system: DashaSystem,
        lots: LotRequest,
    ) -> Result<Rashi, Error> {
        if system == DashaSystem::Profection {
            return Ok(Rashi::of_longitude(self.angles(chart)?.ascendant_deg));
        }
        let read = self.lots_with_request(chart, &[Lot::Fortune, Lot::Daimon], lots)?;
        let sign = |lot: Lot| {
            read.lots
                .iter()
                .find(|placed| placed.lot == lot)
                .map(|placed| placed.place.sign)
                .ok_or_else(|| Error::internal("the lots read leave out one asked for"))
        };
        let fortune = sign(Lot::Fortune)?;
        if system == DashaSystem::ReleasingFortune {
            return Ok(fortune);
        }
        let daimon = sign(Lot::Daimon)?;
        let shared = self.context.settings().dasha.releasing_shared_sign;
        Ok(
            if daimon == fortune && shared == ReleasingSharedSign::Next {
                step(daimon, Direction::Forward, 1)
            } else {
                daimon
            },
        )
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
        .with_hint(if teistro_tajika::ANNUAL_DASHAS.contains(&system) {
            format!(
                "{} divides one year: read it with sdk.chart().annual_dasha",
                system.key()
            )
        } else {
            format!("the dashas built are {}", built.join(", "))
        })
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

    /// The nakshatra-seeded dasha of a founded chart, from its Moon, on the
    /// row its rules choose.
    fn dasha_of(
        foundation: &ChartFoundation,
        system: DashaSystem,
        rules: DashaRules,
        moon_span: Option<Interval>,
    ) -> Result<Dasha, Error> {
        let row = teistro_dasha::row(system, rules.ashtottari_grouping)
            .ok_or_else(|| Self::not_built(system))?;
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
    /// What a sign dasha reads of a founded chart: the lagna's sign, the
    /// grahas' signs and dignities under the settings, the navamsa lagna
    /// and the arudha lagna. One assembly, for the dashas and for the
    /// Jaimini significators they start from.
    fn rashi_chart(self, foundation: &ChartFoundation) -> Result<RashiChart, Error> {
        self.rashi_chart_with(foundation, &state(foundation, self.context.settings())?)
    }

    /// [`rashi_chart`](Self::rashi_chart) over states already computed.
    fn rashi_chart_with(
        self,
        foundation: &ChartFoundation,
        states: &[GrahaState],
    ) -> Result<RashiChart, Error> {
        let lagna = Rashi::from_id(u16::from(foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        let settings = self.context.settings();
        let mut chart = placed_chart(lagna, states, settings.jaimini.node_co_lordship)?;
        chart.navamsa_lagna = varga_chart(foundation, Axis::of(Varga::D9))?.lagna.sign;
        // Brahma is read from the chart it then starts a dasha in, so the
        // chart is assembled first and the sign filled in after.
        chart.brahma = brahma(
            &chart,
            &degrees_in_sign(foundation)?,
            settings.jaimini.node_co_lordship,
            settings.jaimini.brahma,
        )
        .sign(&chart);
        Ok(chart)
    }

    /// A founded chart's Jaimini significators: its **karakamsha**, the
    /// Atmakaraka's navamsha sign with every graha's house from it in both
    /// charts, and its **Brahma graha**, the planet the Sthira dasa starts
    /// from, under the settings' `jaimini.brahma` rule and
    /// `jaimini.node_co_lordship` (`03-design/jaimini-significators.md`).
    ///
    /// The Atmakaraka is the one the settings' `jaimini.chara_karakas`
    /// scheme ranks first, seven karakas or eight.
    ///
    /// A document read with [`ChartRequest::with_jaimini`] answers with its
    /// own `jaimini` section, the reading under the settings it was cast
    /// with; any other is computed under this context's.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build()?;
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request)?.value;
    /// let jaimini = sdk.chart().jaimini(&chart)?;
    /// // Every graha has a house from the karakamsha in both charts.
    /// assert!(jaimini.karakamsha.in_rasi.iter().all(|house| (1..=12).contains(house)));
    /// // Brahma is either found, or its absence says why.
    /// assert_eq!(jaimini.brahma.graha.is_none(), jaimini.brahma.none.is_some());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A chart whose states cannot be computed, or whose divisional charts
    /// cannot be cast.
    pub fn jaimini(
        self,
        chart: &Document,
    ) -> Result<teistro_dasha::jaimini::JaiminiReading, Error> {
        match &chart.jaimini {
            Some(reading) => Ok(reading.clone()),
            None => self.jaimini_of(&chart.foundation),
        }
    }

    /// A founded chart's Jaimini significators, which a reading's `jaimini`
    /// section and [`jaimini`](Self::jaimini) both answer: the states are
    /// computed once, for the karakas and for the chart Brahma is read in.
    fn jaimini_of(
        self,
        foundation: &ChartFoundation,
    ) -> Result<teistro_dasha::jaimini::JaiminiReading, Error> {
        let settings = self.context.settings();
        let states = state(foundation, settings)?;
        let ruled = crate::rules_bridge::rule_chart(foundation, &states, None, None)?;
        let placed = |graha: Graha| ruled.placement(teistro_rules::Body::Graha(graha));
        let atmakaraka = GRAHAS_IN_ORDER
            .into_iter()
            .find(|graha| {
                let at = placed(*graha);
                let held = match settings.jaimini.chara_karakas {
                    CharaKarakas::Eight => at.karaka8,
                    _ => at.karaka7,
                };
                held == Some(CharaKaraka::Atmakaraka)
            })
            .ok_or_else(|| Error::internal("a chart ranks an Atmakaraka"))?;
        let signs = GRAHAS_IN_ORDER.map(|graha| placed(graha).sign);
        let navamshas = GRAHAS_IN_ORDER.map(|graha| placed(graha).navamsha);
        let degrees = degrees_in_sign(foundation)?;
        let rashi = self.rashi_chart_with(foundation, &states)?;
        Ok(JaiminiReading {
            karakamsha: karakamsha(atmakaraka, &signs, &navamshas),
            brahma: brahma(
                &rashi,
                &degrees,
                settings.jaimini.node_co_lordship,
                settings.jaimini.brahma,
            ),
            graha_arudhas: graha_arudhas(
                &rashi,
                settings.jaimini.node_co_lordship,
                settings.jaimini.graha_arudha_exception,
            ),
        })
    }

    /// Each sign's pada lord in a read document under the settings'
    /// `jaimini.node_co_lordship` (crux C135), which the rules' padas count
    /// to: the catalogue's lords under its default.
    fn pada_lords_of(self, document: &Document) -> Result<[Graha; 12], Error> {
        let co_lordship = self.context.settings().jaimini.node_co_lordship;
        let lagna = Rashi::from_id(u16::from(document.foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        // `RuleInputs::of` has refused a document without its states, so an
        // error here is the assembly's own and is not papered over.
        let states = document.state.as_deref().unwrap_or_default();
        let chart = placed_chart(lagna, states, co_lordship)?;
        Ok(pada_lords(&chart, co_lordship))
    }

    fn rashi_dasha_of_row(
        self,
        foundation: &ChartFoundation,
        row: &teistro_dasha::RashiRow,
        rules: DashaRules,
        rashi: RashiRules,
    ) -> Result<RashiDasha, Error> {
        let chart = self.rashi_chart(foundation)?;
        RashiDasha::new(
            row,
            &chart,
            foundation.instant,
            rules.year_length,
            rules.after_cycle,
            rashi,
        )
    }

    /// Gochar: the grahas in transit at each instant asked for, each read
    /// from the natal chart's reference sign — its Moon's by Phaladeepika
    /// ch. 26 v. 1 — with its good houses, its vedha and who obstructs it,
    /// under the settings' `gochar` group (`03-design/gochar.md`).
    ///
    /// The transit charts are founded at the natal place in one batch, so a
    /// year of daily snapshots is one founder's work; a transit's sign and
    /// degrees do not depend on a clock, so none is asked for.
    ///
    /// ```no_run
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// # use teistro::{ChartRequest, Context, Ephemeris, GocharRequest, UtcOffset};
    /// # use teistro::gochar::Verdict;
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let natal = sdk
    ///     .chart()
    ///     .reading(JulianDay::literal(2_447_995.489_583_333_5), &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)))?
    ///     .value;
    /// let today = sdk.chart().gochar(&natal, &GocharRequest::at(JulianDay::<Utc>::literal(2_461_000.5)))?;
    /// let good = today.value[0].grahas.iter().filter(|g| g.verdict == Verdict::Good).count();
    /// # let _ = good;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Whatever founding a transit chart refuses, such as an instant the
    /// ephemeris does not cover.
    pub fn gochar(
        self,
        natal: &Document,
        request: &crate::gochar_request::GocharRequest,
    ) -> Result<Envelope<Vec<teistro_gochar::GocharReading>>, Error> {
        let reference = crate::gochar_request::reference(&natal.foundation, request.from())?;
        let settings = self.context.settings();
        let rules = teistro_gochar::GocharRules::of(settings);
        // The natal prastara once for the batch, when the Ashtakavarga's
        // reading was asked for.
        let prastara = request
            .ashtakavarga()
            .then(|| {
                Self::ashtakavarga_chart_of(&natal.foundation).map(|chart| {
                    teistro_strength::ashtakavarga::prastara(
                        &chart,
                        settings.strength.moon_bindu_from_jupiter,
                    )
                })
            })
            .transpose()?;
        // The grahas' places alone, every instant in one request: a
        // reading reads nothing else of a transit chart, so founding one
        // (its day, houses and lagna) would be work thrown away.
        let place = natal.foundation.place;
        let read = self.founding(UtcOffset::UTC, |founder| {
            founder.longitudes(request.instants(), &place)
        })?;
        let readings = read
            .value
            .iter()
            .map(|longitudes| {
                crate::gochar_request::reading(longitudes, reference, rules, prastara.as_ref())
            })
            .collect();
        Ok(Envelope::sealing(readings, read.provenance))
    }

    /// Every ingress, station and aspect to a natal point of a window,
    /// against a natal chart: the transit hit list
    /// (`03-design/transit-hit-list.md`), sorted by instant, then graha,
    /// then kind.
    ///
    /// Each sign and nakshatra is the one a chart founded at that instant
    /// would give — the search reads the chart's own zodiac, not a frame's
    /// mean-ayanamsha reading — and a retrograde re-entry is its own event,
    /// entering the sign before the line it crossed.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, HitRequest};
    /// # use teistro::gochar::hits::HitEvent;
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let natal: Document = todo!();
    /// let year = HitRequest::between(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_461_041.5));
    /// for hit in sdk.chart().hits(&natal, &year)?.value {
    ///     if let HitEvent::SignIngress { into, .. } = hit.event {
    ///         println!("{:?} enters {into:?} at {}", hit.graha, hit.instant.get());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`HitRequest::check`] refuses; whatever the search refuses,
    /// such as a window the ephemeris does not cover.
    pub fn hits(
        self,
        natal: &Document,
        request: &crate::hit_request::HitRequest,
    ) -> Result<Envelope<Vec<teistro_gochar::hits::Hit>>, Error> {
        self.hits_many([natal], request).map(|found| {
            let provenance = found.provenance.clone();
            let one = found.value.into_iter().next().unwrap_or_default();
            Envelope::sealing(one, provenance)
        })
    }

    /// [`Chart::hits`] for many natal charts over one window, **searching
    /// the sky once**: each graha's longitude is scanned one time for every
    /// chart, its ingresses and stations — which are the sky's and not a
    /// chart's — are refined once and handed to each, and only the aspects
    /// to each chart's own points are refined per chart. One list per
    /// chart, in the order given.
    ///
    /// Under a topocentric frame the sky is the observer's, so the charts
    /// are searched once per place instead; the envelope's provenance is
    /// the first place's.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, HitRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let family: Vec<Document> = todo!();
    /// let year = HitRequest::between(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_461_041.5));
    /// let lists = sdk.chart().hits_many(&family, &year)?.value;
    /// assert_eq!(lists.len(), family.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Chart::hits`]; no chart at all is `INVALID_ARG` naming `natals`.
    pub fn hits_many<'d>(
        self,
        natals: impl IntoIterator<Item = &'d Document>,
        request: &crate::hit_request::HitRequest,
    ) -> Result<Envelope<Vec<Vec<teistro_gochar::hits::Hit>>>, Error> {
        use crate::hit_request::{HitKind, groups, hit_of, lattices_of};
        use teistro_core::settings::Centre;
        use teistro_gochar::hits::{self, Hit};

        request.check()?;
        let natals: Vec<&Document> = natals.into_iter().collect();
        if natals.is_empty() {
            return Err(
                Error::invalid_arg("no natal chart to search the transits against")
                    .with_field("natals"),
            );
        }
        let topocentric = self.context.settings().frame.centre == Centre::Topocentric;
        let mut lists: Vec<Vec<Hit>> = vec![Vec::new(); natals.len()];
        let mut provenance = None;
        for (place, members) in groups(&natals, topocentric) {
            let meanings = lattices_of(request, &natals, &members)?;
            let lattices: Vec<_> = meanings.iter().map(|(_, lattice)| *lattice).collect();
            let found = self.founding(UtcOffset::UTC, |founder| {
                founder.transit_events(
                    &place,
                    request.grahas(),
                    &lattices,
                    request.asks(HitKind::Station),
                    (request.from(), request.to()),
                )
            })?;
            for event in &found.value {
                let Some((happened, only)) = hit_of(event, &meanings, request) else {
                    continue;
                };
                let hit = Hit {
                    instant: event.instant,
                    graha: event.graha,
                    event: happened,
                };
                // An event of the sky goes to every chart of the group; an
                // aspect to its own chart alone.
                let to: &[usize] = match &only {
                    Some(chart) => std::slice::from_ref(chart),
                    None => &members,
                };
                for chart in to {
                    if let Some(list) = lists.get_mut(*chart) {
                        list.push(hit);
                    }
                }
            }
            provenance.get_or_insert(found.provenance);
        }
        for list in &mut lists {
            hits::sort(list);
        }
        let provenance = provenance
            .ok_or_else(|| Error::internal("a batch of charts was searched in no group"))?;
        Ok(Envelope::sealing(lists, provenance))
    }

    /// A chart read as KP (`03-design/kp.md`): every cusp and every graha
    /// the chart carries with its sign, star, sub and sub-sub lords, and
    /// each graha in the house whose cusp it follows.
    ///
    /// The cusps are the house system `houses.module_overrides.kp` names,
    /// Placidus's under `kp-default` and when nothing is named, as the
    /// Readers take them; inside the polar circle the polar policy decides,
    /// and [`KpChart::system`] says which system built them. It needs **no
    /// ephemeris**: the cusps are rebuilt from the chart's instant, place
    /// and zodiac, so a stored chart answers it.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// # use teistro::catalogue::HouseSystem;
    /// let sdk = Context::builder()
    ///     .ephemeris([Ephemeris::Builtin])
    ///     .profile("kp-default")
    ///     .build()?;
    /// let chennai = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_451_545.0), &ChartRequest::at(chennai, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let kp = sdk.chart().kp(&chart, &KpRequest::new())?;
    /// assert_eq!(kp.system, HouseSystem::Placidus);
    /// let lagna = kp.cusp(1).expect("twelve cusps");
    /// println!("lagna sub lord {:?}", lagna.lords.sub.lord);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A chart founded under an ayanamsha [`KpRequest`] does not take,
    /// named `frame.ayanamsha` (C157); a chart whose angles were its
    /// provider's; and a house system the polar policy refuses at the
    /// chart's latitude.
    pub fn kp(self, chart: &Document, request: &KpRequest) -> Result<KpChart, Error> {
        self.kp_cusps(chart, request, request.number())
    }

    /// A KP chart with the chart's own cusps, or those raising a horary
    /// number's start.
    fn kp_cusps(
        self,
        chart: &Document,
        request: &KpRequest,
        number: Option<KpNumber>,
    ) -> Result<KpChart, Error> {
        let foundation = &chart.foundation;
        request.check(&foundation.zodiac)?;
        let settings = self.context.settings();
        let system = override_of(settings, teistro_kp::MODULE).unwrap_or(HouseSystem::Placidus);
        let (delta_t, policy) = (self.context.delta_t(), settings.houses.polar_policy);
        let (cusps, built) = match number {
            None => cusps_of(foundation, system, delta_t, policy)?,
            Some(number) => {
                let lagna = number.start().to_degrees();
                cusps_raising(foundation, system, lagna, delta_t, policy)?
            }
        };
        let mut longitudes = [Nas::ZERO; 12];
        for (longitude, cusp) in longitudes.iter_mut().zip(cusps) {
            *longitude = Nas::try_from_degrees(cusp)?;
        }
        // The number's start is the lagna exactly, not the search's
        // nanodegree from it.
        if let (Some(number), Some(lagna)) = (number, longitudes.first_mut()) {
            *lagna = number.start();
        }
        let planets = foundation
            .grahas
            .iter()
            .map(|graha| {
                Ok(
                    Position::new(graha.graha, Nas::try_from_degrees(graha.longitude_deg)?)
                        .retrograde(graha.is_retrograde()),
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(KpChart::new(built, longitudes, planets))
    }

    /// A **KP horary** chart from the querent's number (KP Reader I; crux
    /// C156): the lagna at the start of the number's arc, the other cusps
    /// those that ascendant has at the place, from the meridian that raises
    /// it at the moment's obliquity and ayanamsha, and the planets where
    /// `moment` placed them — the chart founded for the moment of judgement
    /// at the place the question is judged.
    ///
    /// The cusps are the system [`ChartArea::kp`] reads; the ayanamsha is
    /// checked as there.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpNumber, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// // The querent says 74: the lagna is Cancer 14°53′20″.
    /// let horary = sdk.chart().kp_horary(&now, KpNumber::new(74)?, &KpRequest::new())?;
    /// assert_eq!(horary.cusp(1).map(|cusp| cusp.longitude), Some(KpNumber::new(74)?.start()));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`]; and a latitude where no meridian raises the
    /// number's start, named `place.latitude`.
    pub fn kp_horary(
        self,
        moment: &Document,
        number: KpNumber,
        request: &KpRequest,
    ) -> Result<KpChart, Error> {
        self.kp(moment, &request.for_number(number))
    }

    /// A KP chart's **significators** (`03-design/kp.md` §1): each house's
    /// four levels in the Reader's order, the planets conjoined with or
    /// aspected by them, its intercepted signs, and whose results each node
    /// gives, a node's own aspects read from `aspect.node_aspects`.
    ///
    /// Needs nothing but the chart, from [`ChartArea::kp`].
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_451_545.0), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let kp = sdk.chart().kp(&chart, &KpRequest::new())?;
    /// let significators = sdk.chart().kp_significators(&kp);
    /// // The 7th house's significators, strongest first.
    /// println!("{:?}", significators.house(7).map(|house| house.in_order()));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn kp_significators(self, kp: &KpChart) -> Significators {
        Significators::of(kp, self.context.settings().aspect.node_aspects)
    }

    /// The KP **ruling planets** at the moment a chart was cast for, the
    /// moment of judgement (KP Reader VI): the lagna's and the Moon's star
    /// and sign lords and the day's lord, the nodes standing for them, and
    /// the rulers a retrograde planet rejects, under the settings' `kp`
    /// group (cruxes C150 to C153).
    ///
    /// The day is the chart's own, sunrise to sunrise, unless
    /// `kp.day_lord_day` is `CIVIL`, which takes the weekday of the civil
    /// date on the clock the request names ([`KpRequest::on_clock`]).
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let ruling = sdk.chart().kp_ruling(&now, &KpRequest::new())?;
    /// println!("ruling planets {:?}", ruling.accepted());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`]; and under `CIVIL` a request without a clock,
    /// named `kp.day_lord_day`.
    pub fn kp_ruling(self, chart: &Document, request: &KpRequest) -> Result<RulingPlanets, Error> {
        let moment = self.kp_cusps(chart, request, None)?;
        self.ruling_of(chart, &moment, request)
    }

    /// A chart read as KP whole (`03-design/kp.md` §5): the chart — the
    /// horary one when the request names a number — its significators,
    /// and the ruling planets at its moment, which are the moment's own
    /// whatever the number.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, KpNumber, KpRequest, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).profile("kp-default").build()?;
    /// let place = Place::new(Latitude::try_new(13.08)?, Longitude::try_new(80.27)?, Altitude::try_new(6.0)?);
    /// let now = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(place, UtcOffset::literal(5, 30, 0)))?
    ///     .value;
    /// let reading = sdk.chart().kp_reading(&now, &KpRequest::new().for_number(KpNumber::new(74)?))?;
    /// println!("{:?} rule", reading.ruling.accepted());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::kp`] and [`ChartArea::kp_ruling`].
    pub fn kp_reading(self, chart: &Document, request: &KpRequest) -> Result<KpReading, Error> {
        let moment = self.kp_cusps(chart, request, None)?;
        let ruling = self.ruling_of(chart, &moment, request)?;
        let read = match request.number() {
            None => moment,
            Some(number) => self.kp_cusps(chart, request, Some(number))?,
        };
        let significators = self.kp_significators(&read);
        Ok(KpReading::new(read, significators, ruling))
    }

    /// The ruling planets of a moment already read as KP.
    fn ruling_of(
        self,
        chart: &Document,
        moment: &KpChart,
        request: &KpRequest,
    ) -> Result<RulingPlanets, Error> {
        let settings = self.context.settings();
        let foundation = &chart.foundation;
        let day_lord = match settings.kp.day_lord_day {
            DayLordDay::Civil => {
                let clock = request.clock().ok_or_else(|| {
                    Error::invalid_arg(
                        "the civil day lord is the weekday on a clock, and the request names none",
                    )
                    .with_field("kp.day_lord_day")
                    .with_hint(
                        "ask KpRequest::new().on_clock(offset), or read the day from sunrise",
                    )
                })?;
                weekday_lord(foundation.instant.get() + clock.days())?
            }
            DayLordDay::Sunrise | _ => foundation.day.day.vara.attributes().lord,
        };
        Ok(RulingPlanets::of(
            moment,
            day_lord,
            RulingRules::of(settings),
        ))
    }

    /// Saturn's **Sade Sati** and smaller spells from the natal Moon over a
    /// window (`03-design/sade-sati.md`): every period reaching into it,
    /// **whole** — its first entry and last exit, however far outside the
    /// window — with each phase and every retrograde re-entry as a visit
    /// of its own.
    ///
    /// No classical verse names the seven and a half years, so this is
    /// practice's definition (C147): Saturn in the 12th, 1st and 2nd signs
    /// from the natal Moon, the phases rising, peak and setting; the
    /// request reckons by the Moon's degree instead, counts from the lagna,
    /// or names other smaller spells than the 4th and 8th.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, SadeSatiRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let natal: Document = todo!();
    /// let now = JulianDay::<Utc>::literal(2_461_000.5);
    /// let report = sdk.chart().sade_sati(&natal, &SadeSatiRequest::at(now))?.value;
    /// if let Some(phase) = report.phase_at(now) {
    ///     let whole = &report.sade_sati[0];
    ///     println!("{phase:?}, from {:?} to {:?}", whole.begins(), whole.ends());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`SadeSatiRequest::check`] refuses; whatever the search
    /// refuses, such as a window the ephemeris does not cover.
    pub fn sade_sati(
        self,
        natal: &Document,
        request: &crate::sade_sati_request::SadeSatiRequest,
    ) -> Result<Envelope<teistro_gochar::sade_sati::Report>, Error> {
        let found = self.sade_sati_many([natal], request)?;
        let provenance = found.provenance.clone();
        let one = found
            .value
            .into_iter()
            .next()
            .ok_or_else(|| Error::internal("one chart was asked about and none answered"))?;
        Ok(Envelope::sealing(one, provenance))
    }

    /// [`ChartArea::sade_sati`] for many natal charts over one window,
    /// scanning Saturn **once** for all of them: under whole signs every
    /// chart reads the same crossings, and under the degree reckoning
    /// each chart's own lattice is tested in the same scan. One report per
    /// chart, in the order given; a topocentric frame searches once per
    /// place.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sade_sati`]; no chart at all is `INVALID_ARG`
    /// naming `natals`.
    pub fn sade_sati_many<'d>(
        self,
        natals: impl IntoIterator<Item = &'d Document>,
        request: &crate::sade_sati_request::SadeSatiRequest,
    ) -> Result<Envelope<Vec<teistro_gochar::sade_sati::Report>>, Error> {
        use crate::hit_request::groups;
        use crate::sade_sati_request::{Widening, crossings_of, lattices_of};
        use teistro_core::settings::Centre;
        use teistro_gochar::sade_sati::{self, Outcome, Report, Searched};

        request.check()?;
        let natals: Vec<&Document> = natals.into_iter().collect();
        if natals.is_empty() {
            return Err(
                Error::invalid_arg("no natal chart to read Saturn's spells against")
                    .with_field("natals"),
            );
        }
        let coverage = self
            .context
            .ephemeris()
            .ok_or_else(no_ephemeris)?
            .capabilities()
            .jd_range;
        let topocentric = self.context.settings().frame.centre == Centre::Topocentric;
        let mut reports: Vec<Option<Report>> = vec![None; natals.len()];
        let mut provenance = None;
        for (place, members) in groups(&natals, topocentric) {
            let (lattices, charts) = lattices_of(request, &natals, &members)?;
            let mut widening = Widening::new((request.start(), request.end()), coverage);
            let (found, periods) = loop {
                let (start, end) = widening.window();
                let found = self.founding(UtcOffset::UTC, |founder| {
                    founder.transit_events(&place, &[Graha::Saturn], &lattices, false, (start, end))
                })?;
                let crossings = crossings_of(&found.value, lattices.len());
                let (before, after) = widening.at_coverage();
                let mut periods = Vec::with_capacity(charts.len());
                let (mut wider_before, mut wider_after) = (false, false);
                for chart in &charts {
                    let searched = Searched::new(
                        crossings.get(chart.lattice).map_or(&[][..], Vec::as_slice),
                        chart.origin_deg,
                    )
                    .at_coverage(before, after);
                    match sade_sati::periods(
                        &searched,
                        (request.start(), request.end()),
                        request.spells(),
                    )? {
                        Outcome::Found(found) => periods.push(found),
                        Outcome::Widen { before, after } => {
                            wider_before |= before;
                            wider_after |= after;
                        }
                    }
                }
                if !(wider_before || wider_after) {
                    break (found, periods);
                }
                widening.widen(wider_before, wider_after)?;
            };
            for (chart, periods) in charts.iter().zip(periods) {
                if let Some(slot) = reports.get_mut(chart.index) {
                    *slot = Some(Report::new(chart.reference, request.reckoning(), periods));
                }
            }
            provenance.get_or_insert(found.provenance);
        }
        let reports = reports
            .into_iter()
            .map(|report| report.ok_or_else(|| Error::internal("a chart was searched in no group")))
            .collect::<Result<Vec<_>, Error>>()?;
        let provenance = provenance
            .ok_or_else(|| Error::internal("a batch of charts was searched in no group"))?;
        Ok(Envelope::sealing(reports, provenance))
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
        let searched = foundation.zodiac.search(
            &completion,
            settings.frame.ayanamsha_basis,
            (PrecessionModel::default(), self.context.delta_t()),
        )?;
        let (frame, zodiac) = (searched.frame, searched.zodiac);
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(foundation.place);
        }
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

    /// The **annual charts** a request asks of one birth: each year's
    /// return and Muntha, and — where [`VarshaRequest::place`] asks for the
    /// charts — each year's own chart read down to its office-bearers,
    /// year lord, yogas by matter, sahams, Harsha bala and annual dashas
    /// (`03-design/annual-chart.md`).
    ///
    /// The one call every binding's `varsha` makes, so a year is composed
    /// once, here; the parts are each a method of their own for a caller
    /// who wants one.
    ///
    /// `clock` is the one the birth was cast under, which
    /// [`AnnualPlace::Birth`] casts each year's chart under too.
    ///
    /// ```no_run
    /// # use teistro::{AnnualPlace, Asked, ChartRequest, Context, Document, Ephemeris, VarshaRequest};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (birth, request): (Document, ChartRequest) = todo!();
    /// let asked = VarshaRequest::through(40)
    ///     .at(AnnualPlace::Birth)
    ///     .with_sahams(Asked::All);
    /// let varsha = sdk.chart().varsha(&birth, request.offset(), &asked)?;
    /// let fortieth = varsha.years.last().and_then(|year| year.annual.as_ref());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A request [`VarshaRequest::check`] refuses; no ephemeris; whatever
    /// the provider refuses while searching or founding. Each is named
    /// under `varsha`, the record's name in every binding.
    pub fn varsha(
        self,
        birth: &Document,
        clock: UtcOffset,
        asked: &VarshaRequest,
    ) -> Result<Varsha, Error> {
        asked.check()?;
        self.varsha_of(birth, clock, asked)
            .map_err(|error| error.under(VARSHA))
    }

    /// [`ChartArea::varsha`] for a request already checked, its refusals
    /// named as the parts name them.
    fn varsha_of(
        self,
        birth: &Document,
        clock: UtcOffset,
        asked: &VarshaRequest,
    ) -> Result<Varsha, Error> {
        let years = self
            .praveshas(birth, asked.reading, asked.through)?
            .into_iter()
            .map(|pravesha| {
                // The Muntha is progressed by the years the return
                // *completes*, which is exactly what `year` counts.
                let muntha = self.muntha(birth, pravesha.year, asked.muntha)?;
                let annual = asked
                    .place
                    .map(|place| self.annual_chart(birth, clock, place, asked, pravesha))
                    .transpose()?;
                Ok(VarshaYear {
                    pravesha,
                    muntha,
                    annual,
                })
            })
            .collect::<Result<Vec<VarshaYear>, Error>>()?;
        // The birth's own sahams, which have no year lord.
        let natal_sahams = match &asked.sahams {
            Some(sahams) => self.saham_strength_with_rules(
                birth,
                sahams.members(),
                None,
                asked.strength_rules(),
            )?,
            None => Vec::new(),
        };
        Ok(Varsha {
            years,
            natal_sahams,
        })
    }

    /// A year's own chart, founded where the caller said and read down to
    /// what Tajika reads from it.
    ///
    /// Founded with a bare request — the foundation and nothing else —
    /// because a year's chart asked for the birth's sections too would cost
    /// each year a whole reading nobody requested.
    fn annual_chart(
        self,
        birth: &Document,
        clock: UtcOffset,
        place: AnnualPlace,
        asked: &VarshaRequest,
        pravesha: Pravesha,
    ) -> Result<AnnualChart, Error> {
        let request = match place {
            AnnualPlace::Birth => ChartRequest::at(birth.foundation.place, clock),
            AnnualPlace::At { place, offset } => ChartRequest::at(place, offset),
        };
        let annual = self.reading(pravesha.at, &request)?.value;
        let bearers = self.office_bearers(birth, &annual, pravesha.year)?;
        let year_lord = self.varshesha(birth, &annual, pravesha.year, asked.varshesha)?;
        let yogas = self
            .drishtis(&annual)?
            .into_iter()
            .filter(|pair| pair.yoga.is_some())
            .collect();
        let matters = match &asked.matters {
            Some(matters) => self.tajika_yogas_many(&annual, matters.members(), asked.yogas)?,
            None => Vec::new(),
        };
        let sahams = match &asked.sahams {
            Some(sahams) => self.saham_strength_with_rules(
                &annual,
                sahams.members(),
                Some(year_lord.graha),
                asked.strength_rules(),
            )?,
            None => Vec::new(),
        };
        let harsha = self.harsha_with_rules(&annual, asked.harsha_rules)?;
        // One call for every system asked, so the Sun is read over the year
        // once however many divide it.
        let dashas = match &asked.dashas {
            Some(dashas) => self.annual_dashas(
                birth,
                &annual,
                pravesha.year,
                dashas.members(),
                asked.dasha_rules,
            )?,
            None => Vec::new(),
        };
        Ok(AnnualChart {
            lagna_deg: annual.foundation.lagna_deg,
            bearers,
            year_lord,
            yogas,
            states: self.annual_states(&annual)?,
            matters,
            sahams,
            harsha,
            dashas,
        })
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

    /// The **Harsha bala** of the seven, read from an annual chart you
    /// founded under the source's readings: four places a planet is
    /// "happy" in, five units each (`03-design/tajika-harsha.md`).
    ///
    /// The houses are whole signs from the chart's lagna, and the fourth
    /// part is read from whether the year opened by day. It needs **no
    /// ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// An annual chart that does not place one of the seven, or whose
    /// lagna is not a number.
    pub fn harsha(self, annual: &Document) -> Result<[Harsha; 7], Error> {
        self.harsha_with_rules(annual, HarshaRules::default())
    }

    /// The **Harsha bala** under the readings you name: Venus's house of
    /// joy the verse's 5th, or the 12th a widely used program reads.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::harsha`].
    pub fn harsha_with_rules(
        self,
        annual: &Document,
        rules: HarshaRules,
    ) -> Result<[Harsha; 7], Error> {
        let foundation = &annual.foundation;
        teistro_tajika::harsha(
            &Self::sky_of(annual)?,
            foundation.lagna_deg,
            foundation.day.part.is_daylight(),
            rules,
        )
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
        let sky = Self::sky_of(annual)?;
        let strengths = teistro_tajika::panchavargiya(&sky)?;
        teistro_tajika::varshesha(
            &bearers,
            &sky,
            &strengths,
            annual.foundation.lagna_deg,
            rules,
        )
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
    /// A yoga a call cannot answer for is **listed** in
    /// [`YearYogas::unanswered`], and [`YearYogas::why`] says what it
    /// needs, because "Kuttha did not hold" and "this call cannot tell
    /// you about Kuttha" are different statements; [`YearYogas::holds`]
    /// answers `None` for it rather than `false`.
    ///
    /// Retrograde and combustion — which Rudda, Duhphali-kuttha,
    /// Tambira and Durapha read — are taken from the founded chart's own
    /// graha states, under this context's combustion table, so every one
    /// of the sixteen is answered and the list comes back empty.
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
        self.tajika_yogas_with_rules(annual, house, YogaRules::default())
    }

    /// The sixteen Tajika yogas for one matter, under stated readings.
    ///
    /// [`YogaRules`] carries every reading the sixteen leave open: the
    /// aspects' own ([`crate::SubDegree`]), which each pair yoga inherits,
    /// and the two strength floors (crux C116).
    ///
    /// # Errors
    ///
    /// As [`ChartArea::tajika_yogas`]; and floors that would let a planet
    /// be strong and weak at once, named `strong_from`.
    pub fn tajika_yogas_with_rules(
        self,
        annual: &Document,
        house: House,
        rules: YogaRules,
    ) -> Result<YearYogas, Error> {
        teistro_tajika::year_yogas_with_states(
            annual.foundation.lagna_deg,
            house,
            &Self::sky_of(annual)?,
            &self.annual_states(annual)?,
            rules,
        )
    }

    /// The sixteen Tajika yogas for **several matters** of one annual
    /// chart, in the order the houses are given.
    ///
    /// What the matters share — the chart's sky, its retrograde and
    /// combust planets, the rules' check and the seven strengths — is
    /// read once for the chart and not once a matter, so asking for all
    /// twelve costs far less than twelve calls to
    /// [`ChartArea::tajika_yogas_with_rules`], and answers exactly as they
    /// would.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, House, YogaRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let every = sdk.chart().tajika_yogas_many(&annual, &House::ALL, YogaRules::default())?;
    /// for matter in &every {
    ///     println!("house {}: {} held", matter.house.get(), matter.held.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::tajika_yogas_with_rules`], refused before any
    /// matter is judged.
    pub fn tajika_yogas_many(
        self,
        annual: &Document,
        houses: &[House],
        rules: YogaRules,
    ) -> Result<Vec<YearYogas>, Error> {
        teistro_tajika::year_yogas_many(
            annual.foundation.lagna_deg,
            houses,
            &Self::sky_of(annual)?,
            Some(&self.annual_states(annual)?),
            rules,
        )
    }

    /// The **sahams** asked for, in a founded chart, under the source's
    /// readings (`03-design/tajika-sahams.md`).
    ///
    /// Any chart: the source reads an annual chart's sahams beside the
    /// birth chart's, and both are a [`Document`]. Day or night is the
    /// chart's own; a house's point is Sripati's mid-point, built from the
    /// chart's lagna and midheaven as the source builds it, whatever chalit
    /// the profile gives the chart. Each saham is computed once however
    /// many read it.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Saham};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().build()?;
    /// # let annual: Document = todo!();
    /// let read = sdk.chart().sahams(&annual, &[Saham::Punya, Saham::Vivaha])?;
    /// for asked in &read.points {
    ///     println!("{:?}: {:?}, house {}", asked.saham, asked.point.sign, asked.point.house.get());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart that does not place one of the seven, or whose lagna or a
    /// bhava is not a number.
    pub fn sahams(self, chart: &Document, which: &[Saham]) -> Result<SahamReading, Error> {
        self.sahams_with_rules(chart, which, SahamRules::default())
    }

    /// The sahams asked for, under stated readings: when a sign is added,
    /// where a house stands, and which Roga is meant.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`].
    pub fn sahams_with_rules(
        self,
        chart: &Document,
        which: &[Saham],
        rules: SahamRules,
    ) -> Result<SahamReading, Error> {
        teistro_tajika::sahams(&self.saham_sky_of(chart)?, which, rules)
    }

    /// Where a saham of the caller's own falls: another authority's, or
    /// one the source does not give, over the same factors and rules.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`]; and a node or an unreadable fixed degree
    /// as a factor, named by the factor.
    pub fn saham_point(
        self,
        chart: &Document,
        formula: &SahamFormula,
        rules: SahamRules,
    ) -> Result<SahamPlace, Error> {
        teistro_tajika::saham_point(&self.saham_sky_of(chart)?, formula, rules)
    }

    /// A founded chart's angles — its ascendant, its midheaven and the
    /// obliquity they were built on — in its own zodiac, as it was founded.
    ///
    /// By the sphere from its instant and place, which needs no
    /// ephemeris; or, for a chart whose provider **defined** its angles (a
    /// classical text, whose Lagna and meridian are its own reckoning,
    /// `03-design/classical-chart.md` §5), from this context's provider,
    /// which must be that one. A module whose source names its own
    /// division reads the midheaven from here: the Tajika sahams read a
    /// house's Sripati mid-point whatever chalit the profile gives.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build().unwrap();
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request).unwrap().value;
    /// let angles = sdk.chart().angles(&chart).unwrap();
    /// assert!((angles.ascendant_deg - chart.foundation.lagna_deg).abs() < 1e-9);
    /// ```
    ///
    /// # Errors
    ///
    /// An instant outside the Delta T model's range, a chart the polar
    /// policy refuses, or — for a chart whose angles were its provider's —
    /// a context with no ephemeris, or one whose provider does not define
    /// them.
    pub fn angles(self, chart: &Document) -> Result<ChartAngles, Error> {
        let foundation = &chart.foundation;
        if !foundation.angles_are_the_providers() {
            return angles_of(
                foundation,
                self.context.delta_t(),
                self.context.settings().houses.polar_policy,
            );
        }
        // The angles read no clock, so the zone the founder is given is
        // immaterial to them.
        let angles = self.founding(UtcOffset::UTC, |founder| {
            founder.providers_angles_at(foundation.instant, &foundation.place, &foundation.zodiac)
        })?;
        Ok(angles)
    }

    /// The seven planets' **essential dignities** in a chart you founded:
    /// house, exaltation, triplicity, term and face, detriment and fall,
    /// each a flag, and a score (`03-design/essential-dignities.md`).
    ///
    /// The sect is read by the request's rule, Valens's horizon unless
    /// asked otherwise (crux C209), and the answer reports it with the
    /// rules and scores it was read under. The longitudes are the chart's
    /// own, in its zodiac: a tropical chart reads the Western dignities.
    /// The horizon is read from the chart's own angles ([`ChartArea::angles`]),
    /// so it needs **no ephemeris** unless they were its provider's.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::catalogue::Graha;
    /// use teistro::{DignityRequest, Sect, SectRule};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build().unwrap();
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    /// // 12:00 UTC is 17:45 in Kathmandu, after a January sunset.
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request).unwrap().value;
    /// let read = sdk.chart().dignities(&chart, &DignityRequest::default()).unwrap();
    /// assert_eq!((read.sect, read.sect_rule), (Sect::Night, SectRule::Horizon));
    /// // The chart's own sunset agrees, a quarter of an hour on.
    /// let lit = DignityRequest::default().with_sect_rule(SectRule::Daylight);
    /// assert_eq!(sdk.chart().dignities(&chart, &lit).unwrap().sect, Sect::Night);
    /// let saturn = read.planets[0];
    /// assert_eq!(saturn.planet, Graha::Saturn);
    /// assert_eq!(saturn.score, saturn.dignity.score(&read.scores));
    /// ```
    ///
    /// # Errors
    ///
    /// A chart that does not place one of the seven, or whatever
    /// [`ChartArea::angles`] refuses.
    pub fn dignities(self, chart: &Document, request: &DignityRequest) -> Result<Dignities, Error> {
        request.read(&self.chart_sky(chart)?)
    }

    /// Both halves of Lilly's table in a chart you founded: the
    /// **essential dignities** and the **accidental fortitudes** — the
    /// house a planet is counted in, its motion, its place about the Sun,
    /// its partile aspects, a siege and three fixed stars
    /// (`03-design/essential-dignities.md` §Accidental fortitudes).
    ///
    /// The houses are Lilly's Regiomontanus unless the profile overrides
    /// the division under `houses.module_overrides.hellenistic`, and the
    /// answer carries the cusps and the division they are of, with
    /// everything else it was read from (`Fortitudes::sky`). Regulus, Spica and Algol are read at
    /// their apparent places of the chart's date, in its zodiac, by the
    /// SDK's own star catalogue. Like [`ChartArea::dignities`] it needs
    /// **no ephemeris**.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::catalogue::{Graha, HouseSystem};
    /// use teistro::FortitudeRequest;
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build().unwrap();
    /// let london = Place::new(
    ///     Latitude::literal(51.5),
    ///     Longitude::literal(-0.12),
    ///     Altitude::literal(20.0),
    /// );
    /// let request = ChartRequest::at(london, UtcOffset::literal(0, 0, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request).unwrap().value;
    /// let read = sdk.chart().fortitudes(&chart, &FortitudeRequest::default()).unwrap();
    /// assert_eq!(read.sky.houses, HouseSystem::Regiomontanus);
    /// let sun = &read.planets[3];
    /// assert_eq!(sun.planet, Graha::Sun);
    /// assert!(read.net(Graha::Sun).is_some());
    /// // Lilly's almuten of the figure holds the greatest net.
    /// let greatest = read.almutens.figure.totals.iter().copied().max();
    /// for lord in read.almutens.figure.almutens() {
    ///     assert_eq!(read.net(lord), greatest);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::dignities`]; a chart that does not place the North
    /// Node; and `UNSUPPORTED` for a chart whose angles were its
    /// provider's own reckoning, whose cusps the sphere does not
    /// reproduce.
    pub fn fortitudes(
        self,
        chart: &Document,
        request: &FortitudeRequest,
    ) -> Result<Fortitudes, Error> {
        request.read(&self.chart_sky(chart)?, &self.accidental_sky(chart)?)
    }

    /// Lilly's **considerations before judgement** of a horary figure
    /// (*Christian Astrology* I.XIX): whether it is radical, the
    /// Ascendant's and the Moon's degrees, the Moon's course to the end of
    /// her sign, the seventh house and Alkindi's cautions, each reported as
    /// the facts it rests on and none folded into a verdict
    /// (`03-design/hellenistic-considerations.md`).
    ///
    /// The lord of the hour is the chart's own planetary hour, under the
    /// settings' `hora_reckoning`; everything else is read from
    /// [`ChartArea::fortitudes`] under `request`. Like it, it needs **no
    /// ephemeris**: the Moon's course is projected from the chart's places
    /// at their present speeds, within a hundredth of a degree over two
    /// days of a geocentric chart. Read a figure geocentrically, as
    /// Lilly's ephemerides were; a topocentric Moon swings with parallax
    /// by up to a degree a day, which the projection does not follow.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::{ConsiderationRules, FortitudeRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build().unwrap();
    /// let london = Place::new(
    ///     Latitude::literal(51.5),
    ///     Longitude::literal(-0.12),
    ///     Altitude::literal(20.0),
    /// );
    /// let request = ChartRequest::at(london, UtcOffset::literal(0, 0, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request).unwrap().value;
    /// let read = sdk
    ///     .chart()
    ///     .considerations(&chart, &FortitudeRequest::default(), ConsiderationRules::LILLY)
    ///     .unwrap();
    /// // The figure is radical exactly when some ground holds.
    /// assert_eq!(read.radicality.radical(), !read.radicality.grounds.is_empty());
    /// // Void of course means no aspect perfects before she leaves her sign.
    /// assert_eq!(read.moon.course.void(), read.moon.course.next.is_none());
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::fortitudes`].
    pub fn considerations(
        self,
        chart: &Document,
        request: &FortitudeRequest,
        rules: ConsiderationRules,
    ) -> Result<Considerations, Error> {
        let fortitudes = self.fortitudes(chart, request)?;
        considerations(&fortitudes, chart.foundation.timing.hora.lord, rules)
    }

    /// Whether a horary matter is brought to pass
    /// (`03-design/hellenistic-perfection.md`): the relations between the
    /// querent's and the quesited's significators — named in `asked`, or
    /// the lords of the Ascendant and of the house of the matter — each
    /// reported with
    /// what it rests on and never summed into a verdict — the application
    /// and which of Lilly's three it is, the separation still inside the
    /// moieties, each prohibition, frustration and refranation before it,
    /// each translation of light and, when they do not apply, each
    /// collection.
    ///
    /// The timeline is searched on the ephemeris from the chart's instant
    /// to the rules' horizon (unset, until the swifter significator leaves
    /// its sign at its present motion, crux C232; never more than ten
    /// years): every pair of the seven's
    /// Ptolemaic aspects, and the significators' stations, so a
    /// refranation is seen where it happens. The dignities a reception
    /// reads are `request`'s. Read a horary figure geocentrically, as
    /// Lilly's ephemerides were.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::{FortitudeRequest, House, PerfectionRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let london = Place::new(Latitude::try_new(51.5)?, Longitude::try_new(-0.12)?, Altitude::try_new(0.0)?);
    /// let figure = sdk
    ///     .chart()
    ///     .reading(JulianDay::<Utc>::literal(2_461_000.25), &ChartRequest::at(london, UtcOffset::UTC))?
    ///     .value;
    /// // "Shall I marry?": the Ascendant's lord and the seventh's.
    /// let marriage = PerfectionRequest::of_house(House::try_new(7)?);
    /// let matter = sdk.chart().perfection(&figure, &FortitudeRequest::default(), &marriage)?;
    /// if let Some(application) = matter.application {
    ///     println!("{:?} in {:.1} days, {} impediment(s) first", application.aspect, application.days, matter.impediments.len());
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::fortitudes`]; a request
    /// [`PerfectionRequest::from_json`] refuses; a significator outside
    /// the seven, or one planet for both, named `querent` or `quesited`;
    /// and the search's own, such as a window the ephemeris does not
    /// cover.
    pub fn perfection(
        self,
        chart: &Document,
        request: &FortitudeRequest,
        asked: &PerfectionRequest,
    ) -> Result<Matter, Error> {
        self.perfection_in(chart, &self.fortitudes(chart, request)?, asked)
    }

    /// [`ChartArea::perfection`] over fortitudes already read for the
    /// chart, so a caller holding them pays for the search alone.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::perfection`], but for the fortitudes' own.
    pub fn perfection_in(
        self,
        chart: &Document,
        fortitudes: &Fortitudes,
        asked: &PerfectionRequest,
    ) -> Result<Matter, Error> {
        let (querent, quesited) = asked.significators(fortitudes)?;
        let rules = asked.rules;
        let dignities = &fortitudes.dignities;
        let places = dignities
            .planets
            .each_ref()
            .map(|planet| planet.longitude_deg);
        let speeds = fortitudes.sky.speeds_deg_per_day;
        let horizon = rules
            .horizon_for(&places, &speeds, querent, quesited)?
            .min(PERFECTION_HORIZON_CAP_DAYS);
        let foundation = &chart.foundation;
        let from = foundation.instant;
        let to = JulianDay::<Utc>::literal(from.get() + horizon);
        let found = self.founding(UtcOffset::UTC, |founder| {
            founder.contact_events(
                &foundation.place,
                &CHALDEAN_ORDER,
                &[querent, quesited],
                (from, to),
            )
        })?;
        let days = |instant: JulianDay<Utc>| (instant.get() - from.get()).max(0.0);
        let mut contacts = Vec::new();
        for event in &found.value.contacts {
            if let Some(aspect) = teistro_hellenistic::aspect_at(event.separation_deg) {
                let [first, second] = event.planets;
                contacts.push(teistro_hellenistic::Contact::new(
                    first,
                    second,
                    aspect,
                    days(event.instant),
                )?);
            }
        }
        let mut stations = Vec::new();
        for event in &found.value.stations {
            if let TransitEventKind::Station { kind, .. } = event.kind {
                stations.push(teistro_hellenistic::Station::new(
                    event.graha,
                    days(event.instant),
                    kind == StationKind::Retrograde,
                )?);
            }
        }
        let timeline =
            teistro_hellenistic::AspectTimeline::new(places, speeds, horizon, contacts, stations)?;
        teistro_hellenistic::perfection(
            &timeline,
            querent,
            quesited,
            &teistro_hellenistic::Standing::of(fortitudes),
            &rules,
        )
    }

    /// The **lots** Valens gives, in a chart you founded: each the distance
    /// between two of its points counted from a third, the night's arc
    /// taken in a night chart (`03-design/hellenistic-lots.md`).
    ///
    /// Read under Valens's rules: his horizon for the sect, and Fortune
    /// reversed by night, as Daimon is counted the other way
    /// ([`ChartArea::lots_with_request`] reads them otherwise). Each lot is
    /// computed once however many read it, and each answer names its sign,
    /// that sign's lord and its place counted in whole signs from the
    /// ascendant. Like [`ChartArea::dignities`] it needs **no ephemeris**.
    ///
    /// ```
    /// # use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::{Lot, Sect};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Test]).build().unwrap();
    /// let kathmandu = Place::new(
    ///     Latitude::literal(27.7172),
    ///     Longitude::literal(85.324),
    ///     Altitude::literal(1400.0),
    /// );
    /// let request = ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0));
    /// let chart = sdk.chart().reading(JulianDay::<Utc>::literal(2_451_545.0), &request).unwrap().value;
    /// let read = sdk.chart().lots(&chart, &[Lot::Fortune, Lot::Daimon]).unwrap();
    /// assert_eq!(read.sect, Sect::Night);
    /// assert!(read.fortune_reversed);
    /// // Daimon is Fortune reflected in the ascendant.
    /// let ascendant = sdk.chart().angles(&chart).unwrap().ascendant_deg;
    /// let (fortune, daimon) = (read.lots[0].place.longitude_deg, read.lots[1].place.longitude_deg);
    /// let gap = (fortune + daimon - 2.0 * ascendant).rem_euclid(360.0);
    /// assert!(gap < 1e-9 || gap > 360.0 - 1e-9);
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::dignities`].
    pub fn lots(self, chart: &Document, which: &[Lot]) -> Result<LotReading, Error> {
        self.lots_with_request(chart, which, LotRequest::VALENS)
    }

    /// The lots asked for, under a stated sect rule and Fortune's rule by
    /// night: Lilly's, the same by day and night, or Valens's own at
    /// *Anthologies* III.11, reversed while the Moon is up.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::lots`].
    pub fn lots_with_request(
        self,
        chart: &Document,
        which: &[Lot],
        request: LotRequest,
    ) -> Result<LotReading, Error> {
        teistro_hellenistic::lots(&self.lot_sky(chart)?, which, request)
    }

    /// Where a lot of the caller's own falls: another author's, or one
    /// Valens gives by the native rather than the sect, over the same
    /// points and request.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::lots`]; and a planet outside the seven or an
    /// unreadable fixed degree, named by the point it stands in.
    pub fn lot_place(
        self,
        chart: &Document,
        formula: &LotFormula,
        request: LotRequest,
    ) -> Result<LotPlace, Error> {
        teistro_hellenistic::lot_place(&self.lot_sky(chart)?, formula, request)
    }

    /// Where one point a lot is counted from falls on a chart: the
    /// Ascendant, a planet, a lot under `request`, an exaltation or a
    /// fixed degree, with its sign, lord and whole-sign house.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::lots`]; and a planet outside the seven or an
    /// unreadable fixed degree, named `point`.
    pub fn point_place(
        self,
        chart: &Document,
        point: LotPoint,
        request: LotRequest,
    ) -> Result<LotPlace, Error> {
        teistro_hellenistic::point_place(&self.lot_sky(chart)?, point, request)
    }

    /// Valens's decennials begun from any of the seven: his apheta when
    /// the luminary of the sect is "badly situated", which he leaves to
    /// judgement (VI.5, crux C227), or any star a consumer reads from. The
    /// others follow by their longitudes on the chart, under the
    /// `DECENNIALS` system's year and the settings' division.
    ///
    /// ```
    /// use teistro::catalogue::Graha;
    /// use teistro::dasha::Timeline;
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let place = Place::new(
    ///     Latitude::try_new(51.5)?,
    ///     Longitude::try_new(-0.12)?,
    ///     Altitude::try_new(0.0)?,
    /// );
    /// let birth = JulianDay::<Utc>::literal(2_451_545.25);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(birth, &ChartRequest::at(place, UtcOffset::UTC))?
    ///     .value;
    /// // Begun from Jupiter, which takes the first 10 years 9 months.
    /// let decennials = sdk.chart().decennials_from(&chart, Graha::Jupiter)?;
    /// assert_eq!(decennials.order()[0], Graha::Jupiter);
    /// let first = decennials.mahadasha(0, 0).map(|period| period.lord);
    /// assert_eq!(first, Some(Graha::Jupiter));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An `apheta` outside the seven, named `apheta`; a chart that does
    /// not place the seven, named `places`.
    pub fn decennials_from(self, chart: &Document, apheta: Graha) -> Result<DecennialDasha, Error> {
        let rules = DashaRules::of(&self.context.settings().dasha, DashaSystem::Decennials);
        let order = Self::decennial_order_of(chart, apheta)?;
        Self::decennials_in(chart, order, rules, self.decennial_division())
    }

    /// The profected year counted from any point of a chart rather than
    /// the Ascendant: Valens profects from "every point", the Sun, the
    /// Moon, Fortune and Daimon among them (*Anthologies* IV.11). It
    /// starts at the point's sign at birth, one sign a year, each year
    /// the `PROFECTION` system's year length in the context's settings;
    /// a lot is read under `request`. The catalogue's `PROFECTION`
    /// member is this from the Ascendant.
    ///
    /// ```
    /// use teistro::catalogue::{Graha, Rashi};
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// use teistro::{ChartRequest, Context, Ephemeris, LotPoint, LotRequest, UtcOffset};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let place = Place::new(
    ///     Latitude::try_new(51.5)?,
    ///     Longitude::try_new(-0.12)?,
    ///     Altitude::try_new(0.0)?,
    /// );
    /// let birth = JulianDay::<Utc>::literal(2_451_545.25);
    /// let chart = sdk
    ///     .chart()
    ///     .reading(birth, &ChartRequest::at(place, UtcOffset::UTC))?
    ///     .value;
    /// let moon = LotPoint::Planet(Graha::Moon);
    /// let years = sdk.chart().profection_from(&chart, moon, LotRequest::VALENS)?;
    /// let at_birth = sdk.chart().point_place(&chart, moon, LotRequest::VALENS)?.sign;
    /// assert_eq!(years.start(), at_birth);
    /// // The 13th year comes back to the Moon's sign.
    /// assert_eq!(years.sign_of_year(13), Some(at_birth));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::point_place`]; and a year length of no days.
    pub fn profection_from(
        self,
        chart: &Document,
        point: LotPoint,
        request: LotRequest,
    ) -> Result<ProfectionDasha, Error> {
        let start = self.point_place(chart, point, request)?.sign;
        let rules = DashaRules::of(&self.context.settings().dasha, DashaSystem::Profection);
        ProfectionDasha::new(start, chart.foundation.instant, rules.year_length)
    }

    /// What a chart's dignities are read from: the seven's longitudes and
    /// the Sun's place about the horizon.
    fn chart_sky(self, chart: &Document) -> Result<ChartSky, Error> {
        let at = |graha: Graha| Self::longitude_of(chart, graha);
        Ok(ChartSky {
            saturn_deg: at(Graha::Saturn)?,
            jupiter_deg: at(Graha::Jupiter)?,
            mars_deg: at(Graha::Mars)?,
            sun_deg: at(Graha::Sun)?,
            venus_deg: at(Graha::Venus)?,
            mercury_deg: at(Graha::Mercury)?,
            moon_deg: at(Graha::Moon)?,
            sun_altitude_deg: self.altitude_deg(chart, Graha::Sun)?,
            moon_altitude_deg: self.altitude_deg(chart, Graha::Moon)?,
            daylight: chart.foundation.day.part.is_daylight(),
        })
    }

    /// What a chart's accidental fortitudes are read from beside its
    /// longitudes: the seven's motions, the cusps of the module's
    /// division, the angles, the North Node, and the three stars of date
    /// in the chart's zodiac.
    fn accidental_sky(self, chart: &Document) -> Result<AccidentalSky, Error> {
        let foundation = &chart.foundation;
        let settings = self.context.settings();
        let delta_t = self.context.delta_t();
        let system = override_of(settings, teistro_hellenistic::MODULE)
            .unwrap_or(HouseSystem::Regiomontanus);
        let (cusps_deg, houses) =
            cusps_of(foundation, system, delta_t, settings.houses.polar_policy)?;
        let mut speeds_deg_per_day = [0.0; 7];
        for (speed, planet) in speeds_deg_per_day.iter_mut().zip(CHALDEAN_ORDER) {
            *speed = foundation
                .graha(planet)
                .ok_or_else(|| Error::internal(format!("a founded chart places {planet:?}")))?
                .speed_deg_per_day;
        }
        let north_node_deg = foundation
            .graha(Graha::Rahu)
            .ok_or_else(|| {
                Error::invalid_arg("this chart does not place the North Node")
                    .with_field("chart")
                    .with_hint("found it with Rahu among its grahas")
            })?
            .longitude_deg;
        let (tt, _) = tt_of(JulianDay::<Ut1>::literal(foundation.instant.get()), delta_t)?;
        let star = |star: Star| -> Result<f64, Error> {
            let place = place_of(star, tt, &StarOptions::APPARENT)?;
            Ok(foundation.zodiac.of_tropical(place.lon_deg))
        };
        let angles = self.angles(chart)?;
        Ok(AccidentalSky {
            speeds_deg_per_day,
            houses,
            cusps_deg,
            ascendant_deg: angles.ascendant_deg,
            midheaven_deg: angles.midheaven_deg,
            north_node_deg,
            regulus_deg: star(Star::Regulus)?,
            spica_deg: star(Star::Spica)?,
            algol_deg: star(Star::Algol)?,
        })
    }

    /// A body's centre above the true horizon, geocentric and geometric,
    /// read from the chart's own midheaven: the chart's zodiac is shifted
    /// back to the equinox by the body's own two longitudes, so a sidereal
    /// chart reads the same sky.
    fn altitude_deg(self, chart: &Document, body: Graha) -> Result<f64, Error> {
        let placed = chart
            .foundation
            .graha(body)
            .ok_or_else(|| Error::internal(format!("a founded chart places {body:?}")))?;
        let angles = self.angles(chart)?;
        let to_equinox = placed.tropical_deg - placed.longitude_deg;
        Ok(altitude_by_midheaven_deg(
            Spherical {
                lon_deg: placed.tropical_deg,
                lat_deg: placed.latitude_deg,
            },
            angles.midheaven_deg + to_equinox,
            angles.obliquity_deg,
            chart.foundation.place.latitude.get(),
        ))
    }

    /// What a chart's lots are read from: its seven and their altitudes,
    /// and its angles.
    fn lot_sky(self, chart: &Document) -> Result<LotSky, Error> {
        let angles = self.angles(chart)?;
        Ok(LotSky {
            chart: self.chart_sky(chart)?,
            ascendant_deg: angles.ascendant_deg,
            midheaven_deg: angles.midheaven_deg,
        })
    }

    /// What a saham is read from, off a founded chart: its midheaven
    /// ([`ChartArea::angles`]), and its own chalit for a caller who asks for
    /// that.
    fn saham_sky_of(self, chart: &Document) -> Result<SahamSky, Error> {
        let foundation = &chart.foundation;
        let angles = self.angles(chart)?;
        let sky = SahamSky::new(
            Self::sky_of(chart)?,
            foundation.lagna_deg,
            angles.midheaven_deg,
            foundation.day.part.is_daylight(),
        )
        .with_chalit(foundation.chalit.madhya);
        // The nodes only feed a saham's strength, which reads whether a
        // saham stands in their axis; a chart that does not place Rahu
        // leaves that fact unread rather than guessed.
        Ok(match foundation.graha(Graha::Rahu) {
            Some(rahu) => sky.with_rahu(rahu.longitude_deg),
            None => sky,
        })
    }

    /// A saham's **strength**, clause by clause: every clause of the
    /// source's strong and weak lists, named and evaluated, with the facts
    /// they were read from — and **no verdict**, since the source never
    /// scores a saham (`03-design/tajika-saham-strength.md`).
    ///
    /// `year_lord` is the annual chart's lord of the year, from
    /// [`ChartArea::varshesha`], or `None` for a birth chart, which has
    /// none. It needs **no ephemeris**.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sahams`]; a year lord outside the seven, named
    /// `year_lord`.
    pub fn saham_strength(
        self,
        chart: &Document,
        which: &[Saham],
        year_lord: Option<Graha>,
    ) -> Result<Vec<SahamStrength>, Error> {
        self.saham_strength_with_rules(chart, which, year_lord, SahamStrengthRules::default())
    }

    /// A saham's strength under the readings you name: the chapter's or
    /// the catalogue's natures, positional or natural friendship, the
    /// Vishwa floor, and the sahams' and the Harsha bala's own rules.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::saham_strength`].
    pub fn saham_strength_with_rules(
        self,
        chart: &Document,
        which: &[Saham],
        year_lord: Option<Graha>,
        rules: SahamStrengthRules,
    ) -> Result<Vec<SahamStrength>, Error> {
        teistro_tajika::saham_strength(&self.saham_sky_of(chart)?, which, year_lord, rules)
    }

    /// Which of an annual chart's seven are **retrograde** and which
    /// **combust** — the two things its longitudes cannot say.
    ///
    /// Read from the founded chart's own graha states, under this
    /// context's combustion table (`state.combustion_orbs`), so a
    /// consumer who wants another table changes the setting and not
    /// this call.
    ///
    /// # Errors
    ///
    /// A combustion table the SDK does not ship, named by its setting.
    pub fn annual_states(self, annual: &Document) -> Result<AnnualStates, Error> {
        let mut states = AnnualStates::default();
        for one in state(&annual.foundation, self.context.settings())? {
            // The nodes are not among the seven the sixteen read.
            if !SEVEN.contains(&one.graha) {
                continue;
            }
            if one.motion.retrograde {
                states.retrograde.push(one.graha);
            }
            if one.combustion.is_combust() {
                states.combust.push(one.graha);
            }
        }
        states.check()
    }

    /// How a planet of an annual chart stands to the source's
    /// **afflictions** — retrograde, combust, debilitated, in the 6th,
    /// 8th or 12th, or under malefic influence — clause by clause.
    ///
    /// Rudda holds where either of a pair is afflicted at all, and
    /// Durapha reads three of the five; carrying the clauses is what lets
    /// a reader asking *why* be answered.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; as
    /// [`ChartArea::annual_states`].
    pub fn affliction(self, annual: &Document, graha: Graha) -> Result<Affliction, Error> {
        teistro_tajika::affliction(
            graha,
            annual.foundation.lagna_deg,
            &Self::sky_of(annual)?,
            &self.annual_states(annual)?,
        )
    }

    /// Whether a planet of an annual chart is **unqualified**, clause by
    /// clause, in the source's own sense of the word.
    ///
    /// "Neither exalted nor debilitated, nor aspected/associated, nor in
    /// its own Hudda, Drekkana or Navamsha." Khallasara and
    /// Gairi-Kamboola both turn on it, and it is strict: Tajika counts
    /// eight of the twelve sign relations as an aspect, so a planet
    /// nothing aspects needs the other six inside the four neutral
    /// houses at once.
    ///
    /// Every clause is carried rather than collapsed into the verdict,
    /// so a reader asking why a yoga did not hold gets the clause.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn qualification(self, annual: &Document, graha: Graha) -> Result<Qualification, Error> {
        teistro_tajika::qualification(graha, &Self::sky_of(annual)?)
    }

    /// How a planet of an annual chart stands to **strong** and **weak**,
    /// clause by clause.
    ///
    /// The source treats its three as alternatives — "exalted, in its own
    /// house or otherwise strong" — so strength is a disjunction and
    /// weakness is its denial, with no third state between them.
    ///
    /// The floor of the third clause is the one number the source does
    /// **not** give for the yogas: it floors strength at five Vishwa
    /// units for the office-bearers when choosing the year lord, and
    /// says nothing here. That figure is the default and
    /// [`Chart::strength_with_rules`] moves it; crux C116 records why,
    /// and `03-design/muntha-measured.md` §11 measures what moving it
    /// costs.
    ///
    /// It needs **no ephemeris**: the chart is already founded.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn strength(self, annual: &Document, graha: Graha) -> Result<Strength, Error> {
        teistro_tajika::strength(graha, &Self::sky_of(annual)?)
    }

    /// [`Chart::strength`] with the floor between strong and weak chosen.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn strength_with_rules(
        self,
        annual: &Document,
        graha: Graha,
        rules: YogaRules,
    ) -> Result<Strength, Error> {
        teistro_tajika::strength_with_rules(graha, &Self::sky_of(annual)?, rules)
    }

    /// How a planet of an annual chart stands to what **Kuttha** asks of
    /// each lord — powerful, in a kendra or a panaphara, under a benefic's
    /// aspect and no malefic's — clause by clause.
    ///
    /// Kuttha holds where both lords are favoured. A reader asking why it
    /// did not hold for a matter asks this of the two lords, and gets the
    /// clause that stopped it (crux C117).
    ///
    /// It needs **no ephemeris** and no retrograde or combustion: the
    /// chart is already founded, and Kuttha reads neither.
    ///
    /// # Errors
    ///
    /// A body outside the seven, named `graha`; an annual chart that
    /// does not place it.
    pub fn favour(self, annual: &Document, graha: Graha) -> Result<Favour, Error> {
        self.favour_with_rules(annual, graha, YogaRules::default())
    }

    /// [`Chart::favour`] with the strength floors and the Moon's reading
    /// chosen.
    ///
    /// # Errors
    ///
    /// As [`Chart::favour`]; and floors that would let a planet be both,
    /// named `strong_from`.
    pub fn favour_with_rules(
        self,
        annual: &Document,
        graha: Graha,
        rules: YogaRules,
    ) -> Result<Favour, Error> {
        teistro_tajika::favour(
            graha,
            annual.foundation.lagna_deg,
            &Self::sky_of(annual)?,
            rules,
        )
    }

    /// One year's **annual dasha**: the Mudda, the Varsha Yogini or the
    /// Patyayini, from a birth chart and an annual chart **you founded**
    /// (`03-design/annual-dashas.md`).
    ///
    /// The year opens at the annual chart's instant, which is the return.
    /// Under the default clock it closes on the next return, each of its
    /// 360 units the Sun's motion through one degree. The answer carries
    /// the ring the year runs round, the year itself, and every period to
    /// the rules' depth; the readings the sources differ on are
    /// [`AnnualDashaRules`], each named on the answer.
    ///
    /// Asking for more than one system of the same year?
    /// [`ChartArea::annual_dashas`] reads the Sun over the year once for
    /// all of them.
    ///
    /// ```no_run
    /// # use teistro::{AnnualDashaRules, ChartRequest, Context, Document, Ephemeris};
    /// # use teistro::catalogue::DashaSystem;
    /// # use teistro::tajika::Reading;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let natal: Document = todo!();
    /// # let request: ChartRequest = todo!();
    /// let annual = sdk.chart().annual(&natal, Reading::Sidereal, 40, &request)?;
    /// let mudda = sdk.chart().annual_dasha(
    ///     &natal,
    ///     &annual.value,
    ///     40,
    ///     DashaSystem::Mudda,
    ///     AnnualDashaRules::default(),
    /// )?;
    /// let first = &mudda.periods[0];
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ChartArea::annual_dashas`].
    pub fn annual_dasha(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        system: DashaSystem,
        rules: AnnualDashaRules,
    ) -> Result<AnnualDasha, Error> {
        self.annual_dashas(natal, annual, completed_years, &[system], rules)?
            .pop()
            .ok_or_else(|| Error::internal("one system asked is one answered"))
    }

    /// Several annual dashas of **one** year, in the order asked, over one
    /// clock: the Sun's crossings of the year are found once however many
    /// systems read them.
    ///
    /// ```no_run
    /// # use teistro::{AnnualDashaRules, Context, Document, Ephemeris};
    /// # use teistro::tajika::ANNUAL_DASHAS;
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (natal, annual): (Document, Document) = todo!();
    /// let all_three =
    ///     sdk.chart().annual_dashas(&natal, &annual, 40, &ANNUAL_DASHAS, AnnualDashaRules::default())?;
    /// assert_eq!(all_three.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A system other than the three, named `system` with the three in the
    /// hint, before anything is searched; a `completed_years` past two
    /// hundred, named `completed_years`; no ephemeris, for a clock that
    /// reads the Sun or a balance measured by time; what the rules' clock
    /// refuses; a chart that does not place one of the seven.
    pub fn annual_dashas(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        systems: &[DashaSystem],
        rules: AnnualDashaRules,
    ) -> Result<Vec<AnnualDasha>, Error> {
        if completed_years > teistro_tajika::MOST_YEARS {
            return Err(Error::invalid_arg(format!(
                "an annual dasha is read for 0 to {} completed years, not {completed_years}",
                teistro_tajika::MOST_YEARS
            ))
            .with_field("completed_years"));
        }
        if let Some(stranger) = systems
            .iter()
            .find(|system| !teistro_tajika::ANNUAL_DASHAS.contains(system))
        {
            let built: Vec<&str> = teistro_tajika::ANNUAL_DASHAS
                .iter()
                .map(|one| one.key())
                .collect();
            return Err(
                Error::invalid_arg(format!("{} is not an annual dasha", stranger.key()))
                    .with_field("system")
                    .with_hint(format!("the annual dashas are {}", built.join(", "))),
            );
        }
        if systems.is_empty() {
            return Ok(Vec::new());
        }
        let year = &annual.foundation;
        let knots = match rules.clock.divisions() {
            Some(divisions) => {
                let sun = year
                    .graha(Graha::Sun)
                    .ok_or_else(|| Error::internal("a founded chart places the Sun"))?
                    .longitude_deg;
                Some(self.in_chart_sky(year, |longitudes, zodiac| {
                    teistro_tajika::sun_knots(longitudes, zodiac, year.instant, sun, divisions)
                })?)
            }
            None => None,
        };
        let clock = teistro_tajika::year_clock(rules.clock, year.instant, knots)?;
        systems
            .iter()
            .map(|&system| {
                let (ring, seed) =
                    self.annual_ring(natal, annual, completed_years, system, rules)?;
                let dasha = YearDasha::new(ring, clock.clone(), rules.birth_period)?;
                Ok(AnnualDasha::of(
                    &dasha,
                    system,
                    completed_years,
                    rules,
                    seed,
                ))
            })
            .collect()
    }

    /// The ring one annual dasha runs round in a year, and the birth
    /// nakshatra it is seeded from when it is a nakshatra year.
    fn annual_ring(
        self,
        natal: &Document,
        annual: &Document,
        completed_years: u16,
        system: DashaSystem,
        rules: AnnualDashaRules,
    ) -> Result<(YearRing, Option<Nakshatra>), Error> {
        let moon_of = |foundation: &ChartFoundation| {
            foundation
                .graha(Graha::Moon)
                .ok_or_else(|| Error::internal("a founded chart places the Moon"))
                .and_then(|moon| Ok(Nas::try_from_degrees(moon.longitude_deg)?))
        };
        if system == DashaSystem::Patyayini {
            let sky = Self::sky_of(annual)?;
            let strengths = teistro_tajika::panchavargiya(&sky)?;
            let ring =
                teistro_tajika::patyayini_ring(&sky, annual.foundation.lagna_deg, &strengths)?;
            return Ok((ring, None));
        }
        let birth_moon = moon_of(&natal.foundation)?;
        let remaining = match rules.balance {
            MuddaBalance::Whole => None,
            MuddaBalance::NatalMoon | MuddaBalance::EntryMoon => {
                let foundation = if rules.balance == MuddaBalance::NatalMoon {
                    &natal.foundation
                } else {
                    &annual.foundation
                };
                Some(match rules.measure() {
                    Balance::Temporal => teistro_tajika::remaining_by_time(
                        foundation.instant,
                        self.moon_span(foundation, Wheel::Nakshatras)?,
                    )?,
                    _ => teistro_tajika::remaining_by_arc(moon_of(foundation)?),
                })
            }
        };
        let ring = teistro_tajika::nakshatra_ring(system, birth_moon, completed_years, remaining)?;
        Ok((ring, Some(birth_moon.nakshatra())))
    }

    /// Where the seven stand in a founded chart, which both the strengths
    /// and the aspects read.
    /// Where a founded chart places one of the grahas.
    fn longitude_of(chart: &Document, graha: Graha) -> Result<f64, Error> {
        chart
            .foundation
            .graha(graha)
            .map(|placed| placed.longitude_deg)
            .ok_or_else(|| Error::internal(format!("a founded chart places {graha:?}")))
    }

    fn sky_of(annual: &Document) -> Result<AnnualSky, Error> {
        let at = |graha: Graha| Self::longitude_of(annual, graha);
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

    /// Runs a search on a founded chart's **own** sky: its frame,
    /// topocentric when the chart is, and its zodiac on the settings'
    /// ayanamsha basis, so what is found agrees with what the chart placed.
    fn in_chart_sky<R>(
        self,
        foundation: &ChartFoundation,
        search: impl FnOnce(
            &FrameLongitudes<'_, dyn EphemerisProvider + '_>,
            LimbZodiac,
        ) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let settings = self.context.settings();
        let completion = Completion::new(
            provider,
            settings.provider.overrides,
            self.context.delta_t(),
        );
        let searched = foundation.zodiac.search(
            &completion,
            settings.frame.ayanamsha_basis,
            (PrecessionModel::default(), self.context.delta_t()),
        )?;
        let (frame, zodiac) = (searched.frame, searched.zodiac);
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(foundation.place);
        }
        search(&longitudes, zodiac)
    }

    /// The Moon's stay in its segment of a wheel around the birth: its
    /// nakshatra on the twenty-seven, or its segment of the twenty-eight
    /// with Abhijit. Searched in the chart's own frame and zodiac:
    /// topocentric when the chart is, since a topocentric Moon can stand a
    /// degree from the geocentric one and move a segment's edge by hours.
    fn moon_span(self, foundation: &ChartFoundation, wheel: Wheel) -> Result<Interval, Error> {
        let moon = Self::birth_of(foundation, None)?.moon;
        self.in_chart_sky(foundation, |longitudes, zodiac| match wheel {
            Wheel::Nakshatras => Ok(nakshatra_at(longitudes, foundation.instant, zodiac)?.whole),
            Wheel::WithAbhijit => moon_between(
                longitudes,
                foundation.instant,
                zodiac,
                wheel.segment(moon).degrees(),
            ),
        })
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
        if let Some(start) = reading.start_sign() {
            // A time lord rebuilds from the sign its first period names.
            if let Some(cursor) = teistro_dasha::time_lord(
                system,
                start,
                document.foundation.instant,
                reading.rules.year_length,
            ) {
                return cursor;
            }
        }
        if system == DashaSystem::Decennials {
            // The decennials rebuild from the order their first periods
            // run in and the division their third level shows, or the
            // settings' when they store no third level.
            let order = reading.decennial_order().ok_or_else(|| {
                Error::invalid_arg("the document's decennials store fewer than seven periods")
                    .with_field("system")
            })?;
            let division = reading
                .decennial_division()
                .unwrap_or_else(|| self.decennial_division());
            return Self::decennials_in(document, order, reading.rules, division)
                .map(DashaCursor::Decennials);
        }
        if system == DashaSystem::Firdaria {
            // The firdaria rebuild from the luminary that begins them and
            // the nodes' place their stored round shows.
            return FirdariaDasha::new(
                reading.first_lord,
                document.foundation.instant,
                reading.rules.year_length,
                reading.firdaria_nodes(),
            )
            .map(DashaCursor::Firdaria);
        }
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
        Ok(AshtakavargaReading::of(
            &Self::ashtakavarga_chart_of(foundation)?,
            AshtakavargaRules::of(settings),
        ))
    }

    /// What an Ashtakavarga reads of a founded chart: the lagna's sign and
    /// the seven's.
    fn ashtakavarga_chart_of(foundation: &ChartFoundation) -> Result<AshtakavargaChart, Error> {
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
        Ok(AshtakavargaChart {
            lagna: sign(foundation.lagna_sign_index())?,
            signs,
        })
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
        let lagna = Rashi::from_id(u16::from(foundation.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign"))?;
        let arudha_lagna =
            placed_chart(lagna, &states, settings.jaimini.node_co_lordship)?.arudha_lagna;
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
            arudha_lagna,
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
        weekday_lord(found.instant.get())
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

/// One chart of a batch [`ChartArea::interpreted`] sealed: its document,
/// what it answers by rule, and its own content hash.
type SealedChart<'r> = (Document, Option<RulesReading<'r>>, Hash);

/// One chart as [`ChartArea::interpreted`] answers it: the reading, what
/// it answers by rule when rules were asked for, and what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct Interpreted<'r> {
    /// The chart's reading, with the sections asked for and the ones the
    /// composers read.
    pub document: Document,
    /// What the chart answers by rule; `None` when no rules were asked
    /// for.
    pub reading: Option<RulesReading<'r>>,
    /// Saturn's periods from the chart's Moon over the window asked;
    /// `None` when no Sade Sati window was asked for. It is not part of
    /// [`Interpreted::content_hash`], which seals the chart's own value.
    pub sade_sati: Option<teistro_gochar::sade_sati::Report>,
    /// The plans asked for, each `None` where it was not.
    pub plans: Plans,
    /// The hash of this chart's own value — its document, with what it
    /// answers by rule where rules were asked — which the batch's
    /// provenance, hashing the list, does not carry: what a caller holding
    /// this chart alone stamps it with.
    pub content_hash: Hash,
}

/// The lord of the weekday a Julian day falls on, the day read on
/// whichever clock the day number is already shifted to.
fn weekday_lord(jd: f64) -> Result<Graha, Error> {
    // Julian day 0.5 began a Monday; counted from Sunday it is day 1.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rem_euclid of 7 is 0 to 6"
    )]
    let weekday = ((jd + 1.5).floor().rem_euclid(7.0)) as u8;
    Vara::ALL
        .into_iter()
        .find(|vara| vara.attributes().weekday == weekday)
        .map(|vara| vara.attributes().lord)
        .ok_or_else(|| Error::internal("a weekday past Saturday"))
}
