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
    Plans, RuleSet, RulesReading, SadeSatiRequest,
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
}

impl ChartRecords {
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
#[derive(Clone, Debug)]
pub struct Composed<'r> {
    /// The documents, the batch's provenance sealed over the list.
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
            hits: match &records.hits {
                Some(asked) if !documents.is_empty() => self.hits_many(documents, asked)?.value,
                _ => Vec::new(),
            },
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
            matchings: match &records.matching {
                Some(asked) => self
                    .matching_with(documents, asked)
                    .map_err(|error| error.under("matching"))?,
                None => Vec::new(),
            },
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
            fortitudes,
            founded: Envelope::new(founded, read.provenance),
        })
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
