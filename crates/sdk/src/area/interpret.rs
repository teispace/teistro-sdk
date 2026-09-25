//! `sdk.interpret`: what a reading says, as a narrative plan.

use teistro_core::error::Error;
use teistro_interpret::{
    Plan, ashtakavarga as say_ashtakavarga, aspects, bhava_bala as say_bhava_bala,
    chalit as say_chalit, conditions, dasha_phala, houses, karakas, panchanga as say_panchanga,
    phala, placements, positions, readings, states as say_states, strength,
    vimshopaka as say_vimshopaka,
};
use teistro_serial::document::Document;

use crate::context::Context;
use crate::plan_request::PlanRequest;
use crate::rule_request::RulesReading;
use crate::rules_bridge::RuleInputs;

/// `sdk.interpret`: a chart's answers turned into a plan of message keys
/// and slots, which [`IntlArea`](crate::IntlArea) renders in any locale.
///
/// A plan holds no words. That is the point: one plan renders in every
/// locale the engine carries, a consumer may reorder or drop items before
/// rendering, and a golden plan tests the composer rather than the
/// translator (`03-design/interpret-composers.md`).
///
/// ```no_run
/// # use teistro::{Context, Document, Plan};
/// # fn main() -> Result<(), teistro::Error> {
/// # let sdk = Context::builder().build()?;
/// # let document: Document = unimplemented!("a reading, as `chart_reading.rs` asks for one");
/// let plan: Plan = sdk.interpret().placements(&document)?;
/// for item in &plan {
///     println!("{}", sdk.intl().render(&item.key, &item.params).text);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct InterpretArea<'a> {
    context: &'a Context,
}

impl<'a> InterpretArea<'a> {
    pub(crate) const fn of(context: &'a Context) -> InterpretArea<'a> {
        InterpretArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub const fn context(self) -> &'a Context {
        self.context
    }

    /// Where each of the nine grahas stands and who shares a sign.
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]).
    pub fn placements(self, document: &Document) -> Result<Plan, Error> {
        Ok(placements(&RuleInputs::of(document)?.chart))
    }

    /// Where each of the nine grahas stands, to the degree.
    ///
    /// `placements` says the sign; this says the sign and the degree, so a
    /// consumer picks the precision its page wants rather than being given
    /// both (`03-design/interpret-composers.md` §4).
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]).
    pub fn positions(self, document: &Document) -> Result<Plan, Error> {
        Ok(positions(&RuleInputs::of(document)?.chart))
    }

    /// What each of the nine grahas **is** where it stands: its dignity,
    /// its navamsha sign and whether that makes it vargottama, whether it
    /// is retrograde and whether the Sun burns it.
    ///
    /// `placements` and `positions` say where a graha stands; this says the
    /// four facts of the same placement that neither of them says
    /// (`03-design/interpret-composers.md` §4).
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]).
    pub fn conditions(self, document: &Document) -> Result<Plan, Error> {
        let engine = self.context.locale_engine();
        Ok(conditions(&RuleInputs::of(document)?.chart, &*engine))
    }

    /// Which chara karaka each of the nine grahas holds, under the
    /// seven-karaka scheme and the eight-karaka one.
    ///
    /// Both, because they disagree: over the recorded corpus they give a
    /// graha the same karaka about as often as a different one, so emitting
    /// one would be choosing for the consumer. Which ordering the eight are
    /// ranked in is the chart's, not this composer's
    /// (`RuleChart::with_chara_karakas`).
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]).
    pub fn karakas(self, document: &Document) -> Result<Plan, Error> {
        Ok(karakas(&RuleInputs::of(document)?.chart))
    }

    /// Where the placement system and the chalit put a graha in different
    /// bhavas.
    ///
    /// A chart places every graha twice and keeps both readings; this says
    /// where they disagree, and **nothing at all** where they do not,
    /// because the disagreement is the fact. It reads the chart's own
    /// grahas rather than the rules' chart, which carries one house a
    /// graha (`03-design/interpret-composers.md` §8).
    #[must_use]
    pub fn chalit(self, document: &Document) -> Plan {
        say_chalit(&document.foundation.grahas)
    }

    /// What a loaded corpus of state readings says of this chart's
    /// subjects: a graha in a bhava, the lagna's sign, and each limb of the
    /// panchanga.
    ///
    /// **It says nothing until a pack carrying those readings is loaded**,
    /// which is not a failure but the composer's whole shape: every other
    /// composer says what the SDK computed, and this one says what a corpus
    /// carries, so it asks the context's base locale for each subject and
    /// is silent where the answer is no
    /// (`03-design/state-readings.md` §5).
    ///
    /// # Errors
    ///
    /// A document a rule cannot read: one without its graha states, naming
    /// the section to ask for ([`RuleInputs::of`]); and one without its
    /// panchanga, whose limbs this says — refused rather than said without
    /// them, since a plan missing half its subjects reads as a chart the
    /// corpus has nothing to say of.
    pub fn phala(self, document: &Document) -> Result<Plan, Error> {
        // The states first, as every composer over `RuleInputs` refuses a
        // bare document for those.
        let inputs = RuleInputs::of(document)?;
        if document.panchanga.is_none() {
            return Err(Error::invalid_arg(
                "the document carries no panchanga, whose limbs the phala composer says; ask \
                 for it with `ChartRequest::with_panchanga`",
            )
            .with_field("panchanga"));
        }
        let engine = self.context.locale_engine();
        Ok(phala(&inputs.chart, &*engine))
    }

    /// Each bhava's strength in virupas, the first house first.
    ///
    /// # Errors
    ///
    /// A document without its Bhava bala, naming the section to ask for.
    pub fn bhava_bala(self, document: &Document) -> Result<Plan, Error> {
        document
            .bhava_bala
            .as_ref()
            .map(say_bhava_bala)
            .ok_or_else(|| {
                Error::invalid_arg(
                "the document carries no Bhava bala, which the bhava bala composer says; ask for \
                 it with `ChartRequest::with_bhava_bala`",
            )
            .with_field("bhavaBala")
            })
    }

    /// Each graha's Vimshopaka under all four schemes.
    ///
    /// # Errors
    ///
    /// A document without its Vimshopaka, naming the section to ask for.
    pub fn vimshopaka(self, document: &Document) -> Result<Plan, Error> {
        document
            .vimshopaka
            .as_ref()
            .map(say_vimshopaka)
            .ok_or_else(|| {
                Error::invalid_arg(
                "the document carries no Vimshopaka, which the vimshopaka composer says; ask for \
                 it with `ChartRequest::with_vimshopaka`",
            )
            .with_field("vimshopaka")
            })
    }

    /// What the Ashtakavarga says: each graha's bindus in the sign it
    /// stands in, and each sign's sarvashtakavarga.
    ///
    /// # Errors
    ///
    /// A document without its Ashtakavarga, naming the section to ask
    /// for; or one whose chart cannot be read, which the reading has to
    /// stand beside because it is indexed by sign and knows nothing of
    /// where the grahas are.
    pub fn ashtakavarga(self, document: &Document) -> Result<Plan, Error> {
        let Some(reading) = document.ashtakavarga.as_ref() else {
            return Err(Error::invalid_arg(
                "the document carries no Ashtakavarga, which the ashtakavarga composer says; ask \
                 for it with `ChartRequest::with_ashtakavarga`",
            )
            .with_field("ashtakavarga"));
        };
        let chart = RuleInputs::of(document)?.chart;
        Ok(say_ashtakavarga(reading, &chart))
    }

    /// The almanac of the chart's day: its five limbs, the Moon's pada
    /// and whether the birth fell by day.
    ///
    /// # Errors
    ///
    /// A document without its panchanga, naming the section to ask for.
    pub fn panchanga(self, document: &Document) -> Result<Plan, Error> {
        let chart = RuleInputs::of(document)?.chart;
        if chart.panchanga.is_none() {
            return Err(Error::invalid_arg(
                "the document carries no panchanga, which the panchanga composer says; ask for \
                 it with `ChartRequest::with_panchanga`",
            )
            .with_field("panchanga"));
        }
        Ok(say_panchanga(&chart))
    }

    /// The other half of each graha's state: its three friendships and
    /// its four avasthas, with what a loaded corpus says of each avastha.
    ///
    /// # Errors
    ///
    /// A document without its graha states, naming the section to ask for.
    pub fn states(self, document: &Document) -> Result<Plan, Error> {
        let carried = document.state.as_ref().ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no graha states, which the states composer says; ask for \
                 them with `ChartRequest::with_state`",
            )
            .with_field("state")
        })?;
        let engine = self.context.locale_engine();
        Ok(say_states(carried, &*engine))
    }

    /// What each graha's placement says of its dasha, with the reading a
    /// loaded corpus carries of that graha as a dasha lord.
    ///
    /// # Errors
    ///
    /// A document without its dasha phala, naming the section to ask for.
    pub fn dasha_phala(self, document: &Document) -> Result<Plan, Error> {
        let reading = document.dasha_phala.as_ref().ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no dasha phala, which the dasha phala composer says; ask \
                 for it with `ChartRequest::with_dasha_phala`",
            )
            .with_field("dashaPhala")
        })?;
        let engine = self.context.locale_engine();
        Ok(dasha_phala(reading, &*engine))
    }

    /// Each graha's Shadbala in rupas, the strongest first.
    ///
    /// # Errors
    ///
    /// A document without its Shadbala, naming the section to ask for: the
    /// six strengths are computed only when a request asks, so a composer
    /// says which knob was not turned rather than answering an empty plan.
    pub fn strength(self, document: &Document) -> Result<Plan, Error> {
        document.shadbala.as_ref().map(strength).ok_or_else(|| {
            Error::invalid_arg(
                "the document carries no Shadbala, which the strength composer says; ask for it \
                 with `ChartRequest::with_shadbala`",
            )
            .with_field("shadbala")
        })
    }

    /// The lord of each of the twelve bhavas, first house first.
    ///
    /// The relation the rest of the tradition is read through: `placements`
    /// says where a graha stands, and this says what it rules.
    ///
    /// # Errors
    ///
    /// A document without its houses, naming the section to ask for: the
    /// twelve bhavas are computed only when a request asks, so a composer
    /// says which knob was not turned rather than answering an empty plan.
    pub fn houses(self, document: &Document) -> Result<Plan, Error> {
        document
            .houses
            .as_ref()
            .map(|read| houses(read.all()))
            .ok_or_else(|| {
                Error::invalid_arg(
                    "the document carries no houses, whose lords the houses composer says; ask \
                     for them with `ChartRequest::with_houses`",
                )
                .with_field("houses")
            })
    }

    /// Which graha looks at which, and how strongly, with every pair that
    /// looks back said once more as the pair it is.
    ///
    /// The first composer whose messages were written for it: no locale
    /// carried a word for a drishti, so `sdk.aspect` was added in English
    /// and Nepali from the tradition's own vocabulary
    /// (`03-design/interpret-composers.md` §4).
    ///
    /// # Errors
    ///
    /// A document without its aspects, naming the section to ask for: the
    /// drishtis are computed only when a request asks, so a composer says
    /// which knob was not turned rather than answering an empty plan.
    pub fn aspects(self, document: &Document) -> Result<Plan, Error> {
        document
            .aspects
            .as_ref()
            .map(|read| aspects(read.all()))
            .ok_or_else(|| {
                Error::invalid_arg(
                    "the document carries no aspects, which the aspects composer says; ask for \
                     them with `ChartRequest::with_aspects`",
                )
                .with_field("aspects")
            })
    }

    /// What each rule the chart held says: its verse's statement, who took
    /// part, whether a cancellation moved it and how grave it is.
    ///
    /// It takes a reading rather than a document, because a reading is what
    /// carries the rules' answers — `sdk.chart().readings_with_rules` gives
    /// one — and composing from the answers rather than evaluating again is
    /// what keeps this a rename of work already done.
    ///
    /// **What a rule's verse says has two forms**, and the context's locale
    /// engine decides which: where the base locale carries a reading of the
    /// rule — loaded from a readings pack — the item names that reading and
    /// each locale renders its own words; where it does not, the item
    /// carries the verse's cited words untranslated. A context with no
    /// readings pack loaded composes exactly as it did before
    /// (`03-design/interpretation-records.md` §5).
    #[must_use]
    pub fn readings(self, reading: &RulesReading<'_>) -> Plan {
        let engine = self.context.locale_engine();
        readings(
            reading
                .present
                .iter()
                .map(|present| (present.rule, &present.result)),
            &*engine,
        )
    }

    /// Every plan `asked` names, for one chart: a composer not asked for
    /// is absent rather than empty, and one asked for that has nothing to
    /// say is present and empty, which is an answer
    /// (`03-design/plans-at-the-boundary.md` §4).
    ///
    /// `reading` is what the same chart's rules answered, which
    /// `readings` says and never evaluates again; without one, `readings`
    /// is an empty plan. [`ChartArea::interpreted`](crate::ChartArea::interpreted)
    /// founds the charts, computes the sections these composers read and
    /// calls this, which is the one call a consumer usually wants.
    ///
    /// # Errors
    ///
    /// A document without a section a composer asked for reads, naming
    /// the section.
    pub fn plans(
        self,
        document: &Document,
        reading: Option<&RulesReading<'_>>,
        asked: PlanRequest,
    ) -> Result<Plans, Error> {
        Ok(Plans {
            placements: asked
                .placements
                .then(|| self.placements(document))
                .transpose()?,
            readings: asked
                .readings
                .then(|| reading.map_or_else(Plan::default, |reading| self.readings(reading))),
            strength: asked
                .strength
                .then(|| self.strength(document))
                .transpose()?,
            houses: asked.houses.then(|| self.houses(document)).transpose()?,
            positions: asked
                .positions
                .then(|| self.positions(document))
                .transpose()?,
            aspects: asked.aspects.then(|| self.aspects(document)).transpose()?,
            conditions: asked
                .conditions
                .then(|| self.conditions(document))
                .transpose()?,
            karakas: asked.karakas.then(|| self.karakas(document)).transpose()?,
            chalit: asked.chalit.then(|| self.chalit(document)),
            phala: asked.phala.then(|| self.phala(document)).transpose()?,
            bhava_bala: asked
                .bhava_bala
                .then(|| self.bhava_bala(document))
                .transpose()?,
            vimshopaka: asked
                .vimshopaka
                .then(|| self.vimshopaka(document))
                .transpose()?,
            panchanga: asked
                .panchanga
                .then(|| self.panchanga(document))
                .transpose()?,
            states: asked.states.then(|| self.states(document)).transpose()?,
            dasha_phala: asked
                .dasha_phala
                .then(|| self.dasha_phala(document))
                .transpose()?,
            ashtakavarga: asked
                .ashtakavarga
                .then(|| self.ashtakavarga(document))
                .transpose()?,
        })
    }
}

/// The narrative plans one chart was asked for, and only those: a composer
/// not asked for is absent rather than empty, and one asked for that has
/// nothing to say is present and empty, which is an answer
/// (`03-design/plans-at-the-boundary.md` §4).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct Plans {
    /// Where each graha stands and who shares a sign.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placements: Option<Plan>,
    /// What each rule the chart held says.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readings: Option<Plan>,
    /// Each graha's Shadbala, the strongest first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<Plan>,
    /// The lord of each bhava.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub houses: Option<Plan>,
    /// Where each graha stands, to the degree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Plan>,
    /// Which graha looks at which, and how strongly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspects: Option<Plan>,
    /// What each graha is where it stands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Plan>,
    /// Which chara karaka each graha holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub karakas: Option<Plan>,
    /// Where the two house readings disagree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chalit: Option<Plan>,
    /// The placements' effects from the readings corpus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phala: Option<Plan>,
    /// Each bhava's strength.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bhavaBala")]
    pub bhava_bala: Option<Plan>,
    /// Each graha's vimshopaka strength.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vimshopaka: Option<Plan>,
    /// The day's five limbs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panchanga: Option<Plan>,
    /// Each graha's states.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub states: Option<Plan>,
    /// What the running dashas promise.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dashaPhala")]
    pub dasha_phala: Option<Plan>,
    /// The ashtakavarga's bindus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ashtakavarga: Option<Plan>,
}
