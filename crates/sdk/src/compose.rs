//! A chart request composed whole: the charts founded with what they
//! answer by rule and say, and every section a record beside the request
//! asks to be read off them (`03-design/mcp-server.md`, step 2).
//!
//! The boundary composed these itself, beside the blob it encodes, so a
//! second caller of the same request, the agent server first, would have
//! been a second copy of the composition. Here it is one call, and the
//! boundary reads its C request into [`ChartRecords`] and encodes what
//! [`ChartArea::compose`] answers.

use teistro_core::envelope::{Envelope, Hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_serial::Document;

use crate::gochar::GocharReading;
use crate::sade_sati::Report;
use crate::{
    ChartArea, ChartRequest, ConsiderationRules, Considerations, Dignities, DignityRequest,
    FortitudeRequest, Fortitudes, GocharRequest, Hit, HitRequest, Lot, LotReading, LotRequest,
    Matched, Matter, PartnerMatching, PerfectionRequest, PerfectionRules, PlanInputs, PlanRequest,
    Plans, RuleRequest, RuleSet, RulesReading, SadeSatiRequest,
};

/// The records a chart request carries beside its sections, each read and
/// checked by its own reader before anything is founded, so a bad record
/// is refused before a chart is paid for. Every one is optional, and one
/// left out is a section not read.
#[derive(Clone, Debug, Default)]
pub struct ChartRecords {
    /// The rules every chart is read under.
    pub rules: Option<RuleSet>,
    /// The plans every chart is to say.
    pub plans: PlanRequest,
    /// The window Saturn's periods are reported over.
    pub sade_sati: Option<SadeSatiRequest>,
    /// The transits read against every chart.
    pub gochar: Option<GocharRequest>,
    /// The hit list searched against every chart.
    pub hits: Option<HitRequest>,
    /// The essential dignities.
    pub dignities: Option<DignityRequest>,
    /// The accidental fortitudes, with their own essential dignities.
    pub fortitudes: Option<FortitudeRequest>,
    /// The lots, and the lots the time lords release from.
    pub lots: Option<LotRequest>,
    /// Hellenistic considerations before judgement.
    pub considerations: Option<ConsiderationRules>,
    /// A horary matter's perfection.
    pub perfection: Option<PerfectionRequest>,
    /// A partner every chart is matched with.
    pub matching: Option<PartnerMatching>,
    /// The theme every chart's drawings are written as SVG in.
    #[cfg(feature = "svg")]
    pub theme: Option<teistro_render_svg::Theme>,
    /// The KP reading; one naming no clock reads the request's.
    #[cfg(feature = "kp")]
    pub kp: Option<crate::KpRequest>,
    /// The prashna, which weighs the seven by their Shadbala.
    #[cfg(feature = "prashna")]
    pub prashna: Option<crate::PrashnaRequest>,
    /// The remedies; one read at an instant reads the Vimśottarī daśā
    /// running then.
    #[cfg(feature = "remedies")]
    pub remedies: Option<crate::RemedyRequest>,
    /// The Lal Kitab reading.
    #[cfg(feature = "lalkitab")]
    pub lalkitab: Option<crate::LalKitabRequest>,
    /// The rectification, every chart read as the birth on record on the
    /// request's own clock.
    #[cfg(feature = "rectification")]
    pub rectification: Option<crate::RectificationRequest>,
    /// The annual charts every birth founds, on the request's own clock.
    #[cfg(feature = "tajika")]
    pub varsha: Option<crate::VarshaRequest>,
    /// The Western tables.
    #[cfg(feature = "western")]
    pub western: WesternRecords,
}

/// The Western records a chart request carries, each optional.
#[cfg(feature = "western")]
#[derive(Clone, Debug, Default)]
pub struct WesternRecords {
    /// Progressions and directions, read against the request's own place,
    /// clock and kind.
    pub progressions: Option<crate::ProgressionsRequest>,
    /// Each chart's own Western aspects.
    pub aspects: Option<crate::AspectRequest>,
    /// A partner every chart is read against: the synastry, its composite
    /// and its Davison chart.
    pub synastry: Option<crate::PartnerSynastry>,
    /// The declinations and the parallels among the planets.
    pub parallels: Option<crate::ParallelRequest>,
    /// The antiscia.
    pub antiscia: Option<crate::AntisciaRequest>,
    /// The midpoints.
    pub midpoints: Option<crate::MidpointRequest>,
    /// The Western houses.
    pub houses: Option<crate::HouseRequest>,
    /// The harmonic chart.
    pub harmonic: Option<crate::HarmonicRequest>,
}

/// The Western tables a batch was asked for, a row a chart, each empty
/// when its record was not sent.
#[cfg(feature = "western")]
#[derive(Clone, Debug, Default, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WesternTables {
    /// Every chart's progressions and directions.
    pub progressions: Vec<crate::Progressions>,
    /// Every chart's own aspects.
    pub aspects: Vec<Vec<crate::WesternAspectRow>>,
    /// Every chart read against the partner.
    pub synastry: Vec<crate::PartnerReading>,
    /// Every chart's declinations.
    pub declinations: Vec<crate::Declinations>,
    /// The parallels among each chart's planets.
    pub parallels: Vec<Vec<crate::ParallelRow>>,
    /// Every chart's antiscia.
    pub antiscia: Vec<crate::Antiscia>,
    /// Every chart's midpoints.
    pub midpoints: Vec<Vec<crate::MidpointRow>>,
    /// Every chart's Davison chart with the partner.
    pub davisons: Vec<crate::Partner>,
    /// Every chart's Western houses.
    pub houses: Vec<crate::WesternHouses>,
    /// Every chart's harmonic chart.
    pub harmonics: Vec<crate::HarmonicChart>,
}

impl ChartRecords {
    /// The records a chart request names, as every binding writes them:
    /// the key in a JSON chart request ([`FoundRequest`](crate::FoundRequest))
    /// and the name [`ChartRecords::read`] takes.
    pub const NAMES: [&'static str; 26] = [
        "theme",
        "rules",
        "interpret",
        "varsha",
        "gochar",
        "hits",
        "sadeSati",
        "kp",
        "prashna",
        "remedies",
        "lalkitab",
        "rectification",
        "dignities",
        "fortitudes",
        "lots",
        "considerations",
        "perfection",
        "progressions",
        "westernAspects",
        "synastry",
        "parallels",
        "antiscia",
        "midpoints",
        "westernHouses",
        "harmonic",
        "matching",
    ];

    /// The records with `name`'s read from `json` by its own reader, which
    /// names a refusal from the record's root (`kp.clock`): the one place
    /// a record's name meets its reader, for the boundary's C fields and a
    /// JSON chart request alike.
    ///
    /// ```
    /// use teistro::ChartRecords;
    ///
    /// let mut records = ChartRecords::default();
    /// records.read("lots", "{}")?;
    /// assert!(records.lots.is_some());
    /// let typo = records.read("lot", "{}").unwrap_err();
    /// assert_eq!(typo.field(), Some("lot"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a name not in [`ChartRecords::NAMES`] or a record
    /// its reader refuses; `CAPABILITY` for a record of a family this
    /// build leaves out.
    pub fn read(&mut self, name: &str, json: &str) -> Result<(), Error> {
        /// The record of a family, read in a build with it and refused in
        /// one without it.
        macro_rules! family {
            ($family:literal, $($slot:ident).+, $reader:path) => {{
                #[cfg(feature = $family)]
                {
                    self.$($slot).+ = Some($reader(json)?);
                }
                #[cfg(not(feature = $family))]
                return Err(Error::left_out($family).with_field(name));
            }};
        }
        match name {
            "rules" => {
                self.rules = Some(
                    RuleRequest::from_json(json)
                        .and_then(|request| request.rule_set())
                        .map_err(|error| error.under(name))?,
                );
            }
            "interpret" => {
                self.plans = PlanRequest::from_json(json).map_err(|error| error.under(name))?;
            }
            "sadeSati" => self.sade_sati = Some(SadeSatiRequest::from_json(json)?),
            "gochar" => self.gochar = Some(GocharRequest::from_json(json)?),
            "hits" => self.hits = Some(HitRequest::from_json(json)?),
            "dignities" => self.dignities = Some(DignityRequest::from_json(json)?),
            "fortitudes" => self.fortitudes = Some(FortitudeRequest::from_json(json)?),
            "lots" => self.lots = Some(LotRequest::from_json(json)?),
            "considerations" => self.considerations = Some(ConsiderationRules::from_json(json)?),
            "perfection" => self.perfection = Some(PerfectionRequest::from_json(json)?),
            "matching" => self.matching = Some(PartnerMatching::from_json(json)?),
            "theme" => family!("svg", theme, teistro_render_svg::Theme::from_json),
            "kp" => family!("kp", kp, crate::KpRequest::from_json),
            "prashna" => family!("prashna", prashna, crate::PrashnaRequest::from_json),
            "remedies" => family!("remedies", remedies, crate::RemedyRequest::from_json),
            "lalkitab" => family!("lalkitab", lalkitab, crate::LalKitabRequest::from_json),
            "rectification" => family!(
                "rectification",
                rectification,
                crate::RectificationRequest::from_json
            ),
            "varsha" => family!("tajika", varsha, crate::VarshaRequest::from_json),
            "progressions" => family!(
                "western",
                western.progressions,
                crate::ProgressionsRequest::from_json
            ),
            "westernAspects" => {
                family!("western", western.aspects, crate::AspectRequest::from_json);
            }
            "synastry" => family!(
                "western",
                western.synastry,
                crate::PartnerSynastry::from_json
            ),
            "parallels" => family!(
                "western",
                western.parallels,
                crate::ParallelRequest::from_json
            ),
            "antiscia" => family!(
                "western",
                western.antiscia,
                crate::AntisciaRequest::from_json
            ),
            "midpoints" => family!(
                "western",
                western.midpoints,
                crate::MidpointRequest::from_json
            ),
            "westernHouses" => family!("western", western.houses, crate::HouseRequest::from_json),
            "harmonic" => family!(
                "western",
                western.harmonic,
                crate::HarmonicRequest::from_json
            ),
            _ => {
                return Err(
                    Error::invalid_arg(format!("no chart record is named `{name}`"))
                        .with_field(name)
                        .with_hint(format!(
                            "the records are {}",
                            ChartRecords::NAMES.join(", ")
                        )),
                );
            }
        }
        Ok(())
    }

    /// The record `name`'s JSON Schema, from the type its reader in
    /// [`ChartRecords::read`] reads, so never stricter than it; `None` for
    /// a record whose reader reads no single type yet, or whose family
    /// this build leaves out. The agent server's tests list each `None`.
    #[cfg(feature = "schema")]
    pub(crate) fn schema(
        name: &str,
        generator: &mut schemars::SchemaGenerator,
    ) -> Option<schemars::Schema> {
        /// The schema of a record a family reads, `None` without it.
        macro_rules! family {
            ($family:literal, $wire:ty) => {{
                #[cfg(feature = $family)]
                {
                    Some(generator.subschema_for::<$wire>())
                }
                #[cfg(not(feature = $family))]
                {
                    None
                }
            }};
        }
        match name {
            "rules" => Some(generator.subschema_for::<RuleRequest>()),
            "interpret" => Some(generator.subschema_for::<PlanRequest>()),
            "theme" => family!("svg", crate::records::ThemePatch),
            "varsha" => family!("tajika", crate::VarshaRequest),
            "synastry" => {
                #[cfg(feature = "western")]
                {
                    Some(crate::western_aspects::synastry_schema(generator))
                }
                #[cfg(not(feature = "western"))]
                {
                    None
                }
            }
            "sadeSati" => Some(generator.subschema_for::<crate::sade_sati_request::Asked>()),
            "gochar" => Some(generator.subschema_for::<crate::gochar_request::Asked>()),
            "hits" => Some(generator.subschema_for::<crate::hit_request::Asked>()),
            "dignities" => Some(generator.subschema_for::<DignityRequest>()),
            "fortitudes" => Some(generator.subschema_for::<FortitudeRequest>()),
            "lots" => Some(generator.subschema_for::<LotRequest>()),
            "considerations" => Some(generator.subschema_for::<ConsiderationRules>()),
            "perfection" => Some(generator.subschema_for::<PerfectionRequest>()),
            "matching" => Some(generator.subschema_for::<PartnerMatching>()),
            "kp" => family!("kp", crate::kp_request::Asked),
            "prashna" => family!("prashna", crate::PrashnaRequest),
            "remedies" => family!("remedies", crate::RemedyRequest),
            "lalkitab" => family!("lalkitab", crate::LalKitabRequest),
            "rectification" => family!("rectification", crate::RectificationRequest),
            "progressions" => family!("western", crate::progressions_request::Asked),
            "westernAspects" => family!("western", crate::AspectRequest),
            "parallels" => family!("western", crate::ParallelRequest),
            "antiscia" => family!("western", crate::AntisciaRequest),
            "midpoints" => family!("western", crate::MidpointRequest),
            "westernHouses" => family!("western", crate::HouseRequest),
            "harmonic" => family!("western", crate::HarmonicRequest),
            _ => None,
        }
    }

    /// The records, refused where two of them ask for one table: the
    /// fortitudes carry their own essential dignities, so a `dignities`
    /// record beside them is refused rather than one of the two silently
    /// winning.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `dignities` when both are asked for.
    pub fn checked(self) -> Result<ChartRecords, Error> {
        if self.dignities.is_some() && self.fortitudes.is_some() {
            return Err(Error::invalid_arg(
                "the essential dignities asked for twice, by `dignities` and by `fortitudes`",
            )
            .with_field("dignities")
            .with_hint(
                "the fortitudes answer the essential dignities too: put this record under `fortitudes.dignities` and drop `dignities`",
            ));
        }
        Ok(self)
    }

    /// The chart request with what these records need of the charts
    /// themselves: the lots a chart reports are the lots its time lords
    /// release from, so one `lots` record sets both; a prashna weighs the
    /// seven by their Shadbala, so one `prashna` record asks for it; and a
    /// remedy read at an instant reads the Vimśottarī daśā running then.
    #[must_use]
    pub fn widen(&self, request: ChartRequest) -> ChartRequest {
        let request = match self.lots {
            Some(rules) => request.with_lot_rules(rules),
            None => request,
        };
        #[cfg(feature = "prashna")]
        let request = if self.prashna.is_some() {
            request.with_shadbala()
        } else {
            request
        };
        #[cfg(feature = "remedies")]
        let request = {
            let vimshottari =
                teistro_core::key::KeyId::from(teistro_core::catalogue::DashaSystem::Vimshottari);
            let at_an_instant = self
                .remedies
                .as_ref()
                .is_some_and(|asked| asked.at.is_some());
            if at_an_instant && !request.dashas().contains(&vimshottari) {
                let dashas: Vec<_> = request
                    .dashas()
                    .iter()
                    .copied()
                    .chain([vimshottari])
                    .collect();
                request.with_dashas(dashas)
            } else {
                request
            }
        };
        request
    }

    /// What the founding reads beside the charts: the rules and the Sade
    /// Sati window.
    fn plan_inputs(&self) -> PlanInputs<'_> {
        let inputs = PlanInputs::from(self.rules.as_ref());
        match &self.sade_sati {
            Some(window) => inputs.with_sade_sati(window),
            None => inputs,
        }
    }
}

/// A chart request composed: the charts, what they answer by rule and
/// say, and each section its records asked for, a row a chart in the
/// batch's order, empty where none was asked.
#[derive(Clone, Debug, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Composed<'r> {
    /// The documents, the batch's provenance sealed over the list. Its
    /// JSON is the documents alone, as `charts`: the provenance is the
    /// whole answer's.
    #[serde(rename = "charts", serialize_with = "documents")]
    #[cfg_attr(feature = "schema", schemars(with = "Vec<Document>"))]
    pub founded: Envelope<Vec<Document>>,
    /// Each chart's own content hash, in the batch's order.
    pub hashes: Vec<Hash>,
    /// What every chart answers by rule; empty when no rules were asked
    /// for.
    pub readings: Vec<RulesReading<'r>>,
    /// What every chart says; empty when no plan was asked for.
    pub plans: Vec<Plans>,
    /// Every chart's Sade Sati report; empty when no window was asked for.
    pub sade_sati: Vec<Report>,
    /// Every chart's transits at each of the record's instants.
    pub gochar: Vec<Vec<GocharReading>>,
    /// Every chart's hit list.
    pub hits: Vec<Vec<Hit>>,
    /// Every chart's essential dignities.
    pub dignities: Vec<Dignities>,
    /// Every chart's accidental fortitudes.
    pub fortitudes: Vec<Fortitudes>,
    /// Every chart's fourteen lots.
    pub lots: Vec<LotReading>,
    /// Every chart's considerations.
    pub considerations: Vec<Considerations>,
    /// Every chart's perfection, with the rules it was weighed under.
    pub perfections: Vec<(Matter, PerfectionRules)>,
    /// Every chart matched with the partner.
    pub matchings: Vec<Matched>,
    /// Every chart's drawings as SVG, in the order they were asked for.
    #[cfg(feature = "svg")]
    pub svgs: Vec<Vec<String>>,
    /// Every chart's KP reading.
    #[cfg(feature = "kp")]
    pub kp: Vec<crate::KpReading>,
    /// Every chart's prashna.
    #[cfg(feature = "prashna")]
    pub prashna: Vec<teistro_prashna::Prashna>,
    /// Every chart's remedies.
    #[cfg(feature = "remedies")]
    pub remedies: Vec<teistro_remedies::Remedies>,
    /// Every chart's Lal Kitab reading.
    #[cfg(feature = "lalkitab")]
    pub lalkitab: Vec<teistro_lalkitab::Life>,
    /// Every chart's rectification.
    #[cfg(feature = "rectification")]
    pub rectification: Vec<crate::Rectification>,
    /// Every chart's annual charts.
    #[cfg(feature = "tajika")]
    pub varsha: Vec<crate::Varsha>,
    /// The Western tables.
    #[cfg(feature = "western")]
    pub western: WesternTables,
}

impl ChartArea<'_> {
    /// The charts `request` founds at `instants`, widened by and read
    /// under `records`, with every section the records ask for.
    ///
    /// The reading, the searching and the composing are the façade's
    /// ([`ChartArea::interpreted`]), so Saturn is scanned once for the
    /// reports and the plan that says them. A considerations or perfection
    /// record reads the fortitudes asked for, and Lilly's when none were,
    /// so no chart's fortitudes are read twice.
    ///
    /// # Errors
    ///
    /// Whatever founding the charts refuses, and each section's refusal,
    /// named under its record (`matching`) or with the chart it was
    /// refused for (a transit's hint).
    pub fn compose<'r>(
        self,
        instants: &[JulianDay<Utc>],
        request: &ChartRequest,
        records: &'r ChartRecords,
    ) -> Result<Composed<'r>, Error> {
        let read = self.interpreted(
            instants,
            &records.widen(request.clone()),
            records.plan_inputs(),
            records.plans,
        )?;
        let mut founded = Vec::with_capacity(read.value.len());
        let mut hashes = Vec::with_capacity(read.value.len());
        let mut readings = Vec::new();
        let mut plans = Vec::new();
        let mut sade_sati = Vec::new();
        for chart in read.value {
            hashes.push(chart.content_hash);
            readings.extend(chart.reading);
            if records.plans.asks_for_something() {
                plans.push(chart.plans);
            }
            sade_sati.extend(chart.sade_sati);
            founded.push(chart.document);
        }
        let documents = founded.as_slice();
        let fortitudes = each(documents, records.fortitudes.as_ref(), |document, asked| {
            self.fortitudes(document, asked)
        })?;
        Ok(Composed {
            hashes,
            readings,
            plans,
            sade_sati,
            gochar: self.transits_of(documents, records.gochar.as_ref())?,
            hits: self.hits_of(documents, records.hits.as_ref())?,
            dignities: each(documents, records.dignities.as_ref(), |document, asked| {
                self.dignities(document, asked)
            })?,
            lots: each(documents, records.lots.as_ref(), |document, asked| {
                self.lots_with_request(document, &Lot::ALL, *asked)
            })?,
            considerations: self.considerations_of(
                documents,
                records.considerations,
                &fortitudes,
            )?,
            perfections: self.perfections_of(
                documents,
                records.perfection.as_ref(),
                &fortitudes,
            )?,
            matchings: self.matchings_of(documents, records.matching.as_ref())?,
            #[cfg(feature = "svg")]
            svgs: each(documents, records.theme.as_ref(), |document, theme| {
                (0..document.drawings.len())
                    .map(|index| self.svg(document, index, theme))
                    .collect()
            })?,
            #[cfg(feature = "kp")]
            kp: {
                let asked = records.kp.map(|asked| match asked.clock() {
                    Some(_) => asked,
                    None => asked.on_clock(request.offset()),
                });
                each(documents, asked.as_ref(), |document, asked| {
                    self.kp_reading(document, asked)
                })?
            },
            #[cfg(feature = "prashna")]
            prashna: each(documents, records.prashna.as_ref(), |document, asked| {
                self.prashna(document, asked)
            })?,
            #[cfg(feature = "remedies")]
            remedies: each(documents, records.remedies.as_ref(), |document, asked| {
                self.remedies(document, asked)
            })?,
            #[cfg(feature = "lalkitab")]
            lalkitab: each(documents, records.lalkitab.as_ref(), |document, asked| {
                self.lalkitab(document, asked)
            })?,
            #[cfg(feature = "rectification")]
            rectification: each(
                documents,
                records.rectification.as_ref(),
                |document, asked| self.rectification(document, request.offset(), asked),
            )?,
            #[cfg(feature = "tajika")]
            varsha: chart_by_chart(
                documents,
                records.varsha.as_ref(),
                None,
                |document, asked| self.varsha(document, request.offset(), asked),
            )?,
            #[cfg(feature = "western")]
            western: self.western_of(documents, &records.western, request)?,
            fortitudes,
            founded: Envelope::new(founded, read.provenance),
        })
    }

    /// Every Western table `records` asks of `documents`; a refusal is
    /// named under its record's root, and one read chart by chart says
    /// which chart.
    #[cfg(feature = "western")]
    fn western_of(
        self,
        documents: &[Document],
        records: &WesternRecords,
        request: &ChartRequest,
    ) -> Result<WesternTables, Error> {
        // Progressions read against the request's own place, clock and
        // kind, not the sections its records widened it by.
        let base = ChartRequest::at(*request.place(), request.offset()).with_kind(request.kind());
        let (declinations, parallels) = match &records.parallels {
            None => (Vec::new(), Vec::new()),
            Some(asked) => documents
                .iter()
                .enumerate()
                .map(|(at, document)| {
                    let refused =
                        |error: Error| error.under("parallels").with_hint(format!("chart {at}"));
                    let declined = self.declinations(document).map_err(refused)?;
                    let parallels =
                        crate::western::parallels(&declined.grahas, asked).map_err(refused)?;
                    Ok((declined, parallels))
                })
                .collect::<Result<Vec<_>, Error>>()?
                .into_iter()
                .unzip(),
        };
        Ok(WesternTables {
            progressions: chart_by_chart(
                documents,
                records.progressions.as_ref(),
                None,
                |document, asked| self.progressions(document, asked, &base),
            )?,
            aspects: chart_by_chart(
                documents,
                records.aspects.as_ref(),
                Some("westernAspects"),
                |document, asked| self.western_aspects(document, asked),
            )?,
            synastry: match &records.synastry {
                Some(asked) => self
                    .synastry_with(documents, asked)
                    .map_err(|error| error.under("synastry"))?,
                None => Vec::new(),
            },
            declinations,
            parallels,
            antiscia: chart_by_chart(
                documents,
                records.antiscia.as_ref(),
                Some("antiscia"),
                |document, asked| self.antiscia(document, asked),
            )?,
            midpoints: chart_by_chart(
                documents,
                records.midpoints.as_ref(),
                Some("midpoints"),
                |document, asked| self.midpoints(document, asked),
            )?,
            davisons: records
                .synastry
                .as_ref()
                .map(|asked| asked.davisons(documents, request.offset()))
                .transpose()
                .map_err(|error| error.under("synastry"))?
                .flatten()
                .unwrap_or_default(),
            houses: chart_by_chart(
                documents,
                records.houses.as_ref(),
                Some("westernHouses"),
                |document, asked| self.western_houses(document, asked),
            )?,
            harmonics: chart_by_chart(
                documents,
                records.harmonic.as_ref(),
                Some("harmonic"),
                |document, asked| self.harmonic(document, asked),
            )?,
        })
    }

    /// Every chart's hit list in **one batch**, which scans the sky once
    /// for every chart; a batch of none asks nothing, which the search
    /// would refuse by `natals`.
    fn hits_of(
        self,
        documents: &[Document],
        asked: Option<&HitRequest>,
    ) -> Result<Vec<Vec<Hit>>, Error> {
        match asked {
            Some(asked) if !documents.is_empty() => Ok(self.hits_many(documents, asked)?.value),
            _ => Ok(Vec::new()),
        }
    }

    /// Every chart matched with the partner, the partner founded once; a
    /// refusal is named under `matching`.
    fn matchings_of(
        self,
        documents: &[Document],
        asked: Option<&PartnerMatching>,
    ) -> Result<Vec<Matched>, Error> {
        asked.map_or_else(
            || Ok(Vec::new()),
            |asked| {
                self.matching_with(documents, asked)
                    .map_err(|error| error.under("matching"))
            },
        )
    }

    /// Every chart's transits: one batch a chart, which places the grahas
    /// at every instant in one request; a refusal says which chart of the
    /// batch it was refused for.
    fn transits_of(
        self,
        documents: &[Document],
        asked: Option<&GocharRequest>,
    ) -> Result<Vec<Vec<GocharReading>>, Error> {
        let Some(asked) = asked else {
            return Ok(Vec::new());
        };
        documents
            .iter()
            .enumerate()
            .map(|(at, document)| {
                self.gochar(document, asked)
                    .map(|read| read.value)
                    .map_err(|error| error.with_hint(format!("chart {at}")))
            })
            .collect()
    }

    /// Every chart's considerations, read from the fortitudes asked for
    /// and Lilly's when none were.
    fn considerations_of(
        self,
        documents: &[Document],
        asked: Option<ConsiderationRules>,
        fortitudes: &[Fortitudes],
    ) -> Result<Vec<Considerations>, Error> {
        let Some(rules) = asked else {
            return Ok(Vec::new());
        };
        if fortitudes.is_empty() {
            let lilly = FortitudeRequest::default();
            return documents
                .iter()
                .map(|document| self.considerations(document, &lilly, rules))
                .collect();
        }
        documents
            .iter()
            .zip(fortitudes)
            .map(|(document, read)| {
                crate::hellenistic::considerations(
                    read,
                    document.foundation.timing.hora.lord,
                    rules,
                )
            })
            .collect()
    }

    /// Every chart's perfection, weighed on the fortitudes asked for and
    /// Lilly's when none were.
    fn perfections_of(
        self,
        documents: &[Document],
        asked: Option<&PerfectionRequest>,
        fortitudes: &[Fortitudes],
    ) -> Result<Vec<(Matter, PerfectionRules)>, Error> {
        let Some(asked) = asked else {
            return Ok(Vec::new());
        };
        let read = |document: &Document, fortitudes: &Fortitudes| {
            self.perfection_in(document, fortitudes, asked)
                .map(|matter| (matter, asked.rules))
        };
        if fortitudes.is_empty() {
            let lilly = FortitudeRequest::default();
            return documents
                .iter()
                .map(|document| read(document, &self.fortitudes(document, &lilly)?))
                .collect();
        }
        documents
            .iter()
            .zip(fortitudes)
            .map(|(document, fortitudes)| read(document, fortitudes))
            .collect()
    }
}

/// `read` over every document, or nothing when no record asked; a
/// refusal is named under the record's `root`, where one is given, and
/// says which chart it was refused for.
/// The documents of a sealed batch, without the seal.
fn documents<S: serde::Serializer>(
    founded: &Envelope<Vec<Document>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(&founded.value, serializer)
}

#[cfg(any(feature = "tajika", feature = "western"))]
fn chart_by_chart<A, T>(
    documents: &[Document],
    asked: Option<&A>,
    root: Option<&'static str>,
    read: impl Fn(&Document, &A) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            read(document, asked).map_err(|error| {
                let error = match root {
                    Some(root) => error.under(root),
                    None => error,
                };
                error.with_hint(format!("chart {at}"))
            })
        })
        .collect()
}

/// `read` over every document, or nothing when no record asked.
fn each<A, T>(
    documents: &[Document],
    asked: Option<&A>,
    read: impl Fn(&Document, &A) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    asked.map_or_else(
        || Ok(Vec::new()),
        |asked| {
            documents
                .iter()
                .map(|document| read(document, asked))
                .collect()
        },
    )
}
