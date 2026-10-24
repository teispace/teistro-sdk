//! What a consumer asks a chart reading to answer by rule
//! (`03-design/rules-at-the-boundary.md`).
//!
//! A [`RuleRequest`] names the kernel's shipped sets and a consumer's own
//! rules, the readings to evaluate them under, and whether to add the house
//! and longevity readings. It is the record the C boundary's `rules_json`
//! carries, so Rust and every binding read one type. [`RuleRequest::rule_set`]
//! validates it into a [`RuleSet`], which a reading evaluates.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_rules::ThreePairsRules;
use teistro_rules::longevity::{
    AshtakavargaAyus, AshtakavargaAyusRules, Ayurdaya, AyurdayaRules, Chakrayus, ChakrayusRules,
    Dasayus, Marakas, Rasmi, RasmiRules, SpanChoice, ThreePairs,
};
use teistro_rules::{HouseReading, Readings, Rule, RuleResult, Tables, check_references, shipped};

use crate::reading::RuleNeeds;

/// A set of rules the kernel ships.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShippedRules {
    /// The recording engine's doshas the kernel computes.
    Doshas,
    /// The recording engine's yogas the kernel computes.
    Yogas,
    /// The gandantas of BPHS ch. 92.
    Gandantas,
    /// The evils at birth and their cancellations.
    Arishtas,
    /// The generated readings of a graha, a pair and a rising part.
    Readings,
    /// The yogas, doshas and classes of life written from the texts.
    Nabhasas,
}

impl ShippedRules {
    /// Every shipped set.
    pub const ALL: [ShippedRules; 6] = [
        ShippedRules::Doshas,
        ShippedRules::Yogas,
        ShippedRules::Gandantas,
        ShippedRules::Arishtas,
        ShippedRules::Readings,
        ShippedRules::Nabhasas,
    ];

    /// Its key, as serde writes it and a request names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ShippedRules::Doshas => "DOSHAS",
            ShippedRules::Yogas => "YOGAS",
            ShippedRules::Gandantas => "GANDANTAS",
            ShippedRules::Arishtas => "ARISHTAS",
            ShippedRules::Readings => "READINGS",
            ShippedRules::Nabhasas => "NABHASAS",
        }
    }

    /// Its rules.
    #[must_use]
    pub fn rules(self) -> &'static [Rule] {
        match self {
            ShippedRules::Doshas => shipped::computed_doshas(),
            ShippedRules::Yogas => shipped::computed_yogas(),
            ShippedRules::Gandantas => shipped::gandantas(),
            ShippedRules::Arishtas => shipped::arishtas(),
            ShippedRules::Readings => shipped::readings(),
            ShippedRules::Nabhasas => shipped::nabhasas(),
        }
    }
}

/// The two named readings a request evaluates rules under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleReadings {
    /// The texts' readings wherever a text settles one.
    #[default]
    Texts,
    /// The recording engine's.
    RecordingEngine,
}

impl RuleReadings {
    /// The kernel's readings.
    #[must_use]
    pub const fn readings(self) -> Readings {
        match self {
            RuleReadings::Texts => Readings::TEXTS,
            RuleReadings::RecordingEngine => Readings::RECORDING_ENGINE,
        }
    }
}

/// What a chart reading is asked to answer by rule.
///
/// ```
/// use teistro::{RuleRequest, ShippedRules};
///
/// let request = RuleRequest::from_json(r#"{"shipped": ["NABHASAS"], "houses": true}"#)?;
/// assert_eq!(request, RuleRequest::shipped([ShippedRules::Nabhasas]).with_houses());
/// let set = request.rule_set()?;
/// assert_eq!(set.rules(), ShippedRules::Nabhasas.rules());
/// assert!(set.houses() && !set.longevity());
///
/// let wrong = RuleRequest::from_json(r#"{"rules": [{"key": "X", "category": "raja"}]}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("rules[0]"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RuleRequest {
    /// The shipped sets to evaluate.
    pub shipped: Vec<ShippedRules>,
    /// A consumer's own rules, which may name shipped rules by key.
    pub rules: Vec<Rule>,
    /// The readings to evaluate under.
    pub readings: RuleReadings,
    /// Whether to add the twelve house readings.
    pub houses: bool,
    /// Whether to add the three pairs, the three spans, the rays, the
    /// dashas', the wheel of time's and the two ashtakavarga spans, the
    /// span the strongest names and the marakas. Asks the chart for its
    /// Shadbala and Bhava bala, which they weigh, its Vimshottari, whose
    /// balance the dashas' span reads, and its ashtakavarga.
    pub longevity: bool,
    /// The choices the three spans are read under, when `longevity` asks
    /// for them; BPHS's by default (cruxes C104, C302 to C304).
    pub ayurdaya: Option<AyurdayaRules>,
    /// The choices the three pairs are read under, when `longevity` asks
    /// for them; the verses' by default (crux C103).
    pub three_pairs: Option<ThreePairsRules>,
    /// The choices the rays and Rasmija years are read under, when
    /// `longevity` asks for them;
    /// the translator's note's by default (cruxes C305 to C307).
    pub rasmi: Option<RasmiRules>,
    /// The choices the wheel of time's span is read under, when
    /// `longevity` asks for it; the note's figure's by default (crux C310).
    pub chakrayus: Option<ChakrayusRules>,
    /// The choices the two ashtakavarga spans are read under, when
    /// `longevity` asks for them; *Jataka Parijata*'s by default (cruxes
    /// C311 and C312).
    pub ashtakavarga: Option<AshtakavargaAyusRules>,
}

impl RuleRequest {
    /// A request for shipped sets.
    #[must_use]
    pub fn shipped(sets: impl IntoIterator<Item = ShippedRules>) -> RuleRequest {
        RuleRequest {
            shipped: sets.into_iter().collect(),
            ..RuleRequest::default()
        }
    }

    /// The same request with a consumer's own rules added.
    #[must_use]
    pub fn with_rules(mut self, rules: impl IntoIterator<Item = Rule>) -> RuleRequest {
        self.rules.extend(rules);
        self
    }

    /// The same request under other readings.
    #[must_use]
    pub const fn with_readings(mut self, readings: RuleReadings) -> RuleRequest {
        self.readings = readings;
        self
    }

    /// The same request with the house readings.
    #[must_use]
    pub const fn with_houses(mut self) -> RuleRequest {
        self.houses = true;
        self
    }

    /// The same request with the longevity readings.
    #[must_use]
    pub const fn with_longevity(mut self) -> RuleRequest {
        self.longevity = true;
        self
    }

    /// The same request with the longevity readings, the three spans read
    /// under `rules`.
    ///
    /// ```
    /// use teistro::RuleRequest;
    /// use teistro::rules::longevity::AyurdayaRules;
    ///
    /// let request = RuleRequest::default().with_ayurdaya(AyurdayaRules::PARIJATA);
    /// let read = RuleRequest::from_json(
    ///     r#"{"longevity": true, "ayurdaya": {"enemy_exempt": "mars", "enmity": "compound", "rising": "every"}}"#,
    /// )?;
    /// assert_eq!(read, request);
    /// assert_eq!(request.rule_set()?.ayurdaya(), Some(AyurdayaRules::PARIJATA));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub const fn with_ayurdaya(mut self, rules: AyurdayaRules) -> RuleRequest {
        self.longevity = true;
        self.ayurdaya = Some(rules);
        self
    }

    /// The same request with the longevity readings, the three pairs read
    /// under `rules`.
    #[must_use]
    pub const fn with_three_pairs(mut self, rules: ThreePairsRules) -> RuleRequest {
        self.longevity = true;
        self.three_pairs = Some(rules);
        self
    }

    /// The same request with the longevity readings, the rays read under
    /// `rules`.
    ///
    /// ```
    /// use teistro::RuleRequest;
    /// use teistro::rules::longevity::RasmiRules;
    ///
    /// let request = RuleRequest::default().with_rasmi(RasmiRules::VERSE);
    /// let read = RuleRequest::from_json(
    ///     r#"{"longevity": true, "rasmi": {"place": "sign"}}"#,
    /// )?;
    /// assert_eq!(read, request);
    /// assert_eq!(request.rule_set()?.rasmi(), Some(RasmiRules::VERSE));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub const fn with_rasmi(mut self, rules: RasmiRules) -> RuleRequest {
        self.longevity = true;
        self.rasmi = Some(rules);
        self
    }

    /// The same request with the longevity readings, the wheel of time's
    /// span read under `rules`.
    ///
    /// ```
    /// use teistro::RuleRequest;
    /// use teistro::rules::longevity::{ChakraPortion, ChakrayusRules};
    ///
    /// let by_pada = ChakrayusRules { portion: ChakraPortion::Pada };
    /// let read = RuleRequest::from_json(r#"{"longevity": true, "chakrayus": {"portion": "pada"}}"#)?;
    /// assert_eq!(read, RuleRequest::default().with_chakrayus(by_pada));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub const fn with_chakrayus(mut self, rules: ChakrayusRules) -> RuleRequest {
        self.longevity = true;
        self.chakrayus = Some(rules);
        self
    }

    /// The same request with the longevity readings, the two ashtakavarga
    /// spans read under `rules`.
    ///
    /// ```
    /// use teistro::RuleRequest;
    /// use teistro::rules::longevity::{
    ///     AshtakaReductions, AshtakavargaAyusRules, Divisor, SignMeasures,
    /// };
    ///
    /// let balabhadra = AshtakavargaAyusRules {
    ///     measures: SignMeasures::Bphs,
    ///     divisor: Divisor::SevenOverTwentySeven,
    ///     reductions: AshtakaReductions::Balabhadra,
    ///     ..AshtakavargaAyusRules::default()
    /// };
    /// let read = RuleRequest::from_json(
    ///     r#"{"longevity": true, "ashtakavarga": {"measures": "bphs", "divisor": "seven-over-twenty-seven", "reductions": "balabhadra"}}"#,
    /// )?;
    /// assert_eq!(read, RuleRequest::default().with_ashtakavarga(balabhadra));
    /// assert_eq!(read.rule_set()?.ashtakavarga(), Some(balabhadra));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub const fn with_ashtakavarga(mut self, rules: AshtakavargaAyusRules) -> RuleRequest {
        self.longevity = true;
        self.ashtakavarga = Some(rules);
        self
    }

    /// A request read from JSON. A value that does not read is refused by
    /// where it stands — `rules[3].when`, `houses` — with what was wrong.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for JSON that is not a request, naming the field.
    pub fn from_json(text: &str) -> Result<RuleRequest, Error> {
        // Parsed strictly, as every record is, so a key given twice is
        // refused by its path rather than the last one silently kept.
        let mut value = teistro_core::strict::parse(text, "")?;
        // Each rule is read on its own, so a refusal names which one.
        let rules = match value
            .as_object_mut()
            .and_then(|fields| fields.remove("rules"))
        {
            None => Vec::new(),
            Some(serde_json::Value::Array(rules)) => rules
                .into_iter()
                .enumerate()
                .map(|(at, rule)| {
                    teistro_core::strict::deserialize::<Rule>(&rule, &format!("rules[{at}]"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => {
                return Err(
                    Error::invalid_arg("`rules` is not an array of rules").with_field("rules")
                );
            }
        };
        // Named from the request's own root, which its caller prefixes.
        let request: RuleRequest =
            teistro_core::strict::deserialize(&value, "").map_err(|err| {
                err.with_hint(
                    "an object of `shipped`, `rules`, `readings`, `houses`, `longevity`, `ayurdaya`, `threePairs`, `rasmi`, `chakrayus` and `ashtakavarga`",
                )
            })?;
        Ok(request.with_rules(rules))
    }

    /// The request validated into a set: the shipped sets and the consumer's
    /// rules together, each key once, every key a rule names in the set and
    /// no rule reaching itself.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a key given twice or a reference the set cannot
    /// resolve, naming it.
    pub fn rule_set(&self) -> Result<RuleSet, Error> {
        let mut rules: Vec<Rule> = self
            .shipped
            .iter()
            .flat_map(|set| set.rules().iter().cloned())
            .collect();
        rules.extend(self.rules.iter().cloned());
        let mut keys = std::collections::BTreeSet::new();
        for rule in &rules {
            if !keys.insert(rule.key.as_str()) {
                return Err(Error::invalid_arg(format!(
                    "the rule `{}` is given twice, once in a shipped set or twice among the rules",
                    rule.key
                ))
                .with_field("rules"));
            }
        }
        check_references(&rules)
            .map_err(|reason| Error::invalid_arg(reason).with_field("rules"))?;
        // A table no evaluator holds would leave its predicate silently
        // false, so a rule naming one is refused here instead.
        for rule in &rules {
            Tables::classical()
                .check(rule)
                .map_err(|reason| Error::invalid_arg(reason).with_field("rules"))?;
        }
        // A longevity choice without the longevity readings would be read
        // by nothing, so it is refused rather than ignored.
        for (field, given) in [
            ("ayurdaya", self.ayurdaya.is_some()),
            ("threePairs", self.three_pairs.is_some()),
            ("rasmi", self.rasmi.is_some()),
            ("chakrayus", self.chakrayus.is_some()),
            ("ashtakavarga", self.ashtakavarga.is_some()),
        ] {
            if given && !self.longevity {
                return Err(Error::invalid_arg(format!(
                    "`{field}` chooses how the longevity readings are read, which `longevity` does not ask for"
                ))
                .with_field(field)
                .with_hint("set `longevity` to true, or leave the choice out"));
            }
        }
        Ok(RuleSet {
            needs: RuleNeeds::of(&rules),
            rules,
            readings: self.readings,
            houses: self.houses,
            longevity: self.longevity.then(|| LongevityRules {
                ayurdaya: self.ayurdaya.unwrap_or_default(),
                three_pairs: self.three_pairs.unwrap_or_default(),
                rasmi: self.rasmi.unwrap_or_default(),
                chakrayus: self.chakrayus.unwrap_or_default(),
                ashtakavarga: self.ashtakavarga.unwrap_or_default(),
            }),
        })
    }
}

/// The choices the longevity readings are read under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LongevityRules {
    pub(crate) ayurdaya: AyurdayaRules,
    pub(crate) three_pairs: ThreePairsRules,
    pub(crate) rasmi: RasmiRules,
    pub(crate) chakrayus: ChakrayusRules,
    pub(crate) ashtakavarga: AshtakavargaAyusRules,
}

/// A validated set of rules with what else a reading should answer.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleSet {
    rules: Vec<Rule>,
    /// What its rules read of a chart, decided once when the set is built.
    needs: RuleNeeds,
    readings: RuleReadings,
    houses: bool,
    longevity: Option<LongevityRules>,
}

impl RuleSet {
    /// Refuses the set when a rule reads the sky in transit, which `reader`
    /// reads at no instant of: a transit is read at the instants a timed
    /// study asks for ([`ResearchArea::timed`](crate::ResearchArea::timed)),
    /// and would never hold on a natal chart alone.
    pub(crate) fn refuse_transits(&self, reader: &str) -> Result<(), Error> {
        match self.rules.iter().position(Rule::reads_transits) {
            None => Ok(()),
            Some(at) => Err(Error::invalid_arg(format!(
                "the rule `{}` reads the sky in transit, which {reader} reads at no instant",
                self.rules.get(at).map_or("", |rule| rule.key.as_str())
            ))
            .with_field("rules")
            .with_hint(
                "a transit is read at an event's instant: ask for it in a timed study (research)",
            )),
        }
    }

    /// Its rules, the shipped sets' first in the order asked, then the
    /// consumer's.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// What its rules read of a chart.
    pub(crate) const fn needs(&self) -> &RuleNeeds {
        &self.needs
    }

    /// The readings it is evaluated under.
    #[must_use]
    pub const fn readings(&self) -> RuleReadings {
        self.readings
    }

    /// Whether the house readings are asked for.
    #[must_use]
    pub const fn houses(&self) -> bool {
        self.houses
    }

    /// Whether the longevity readings are asked for.
    #[must_use]
    pub const fn longevity(&self) -> bool {
        self.longevity.is_some()
    }

    /// The choices the three spans are read under, when the longevity
    /// readings are asked for.
    #[must_use]
    pub fn ayurdaya(&self) -> Option<AyurdayaRules> {
        self.longevity.map(|rules| rules.ayurdaya)
    }

    /// The choices the three pairs are read under, when the longevity
    /// readings are asked for.
    #[must_use]
    pub fn three_pairs(&self) -> Option<ThreePairsRules> {
        self.longevity.map(|rules| rules.three_pairs)
    }

    /// The choices the rays are read under, when the longevity readings
    /// are asked for.
    #[must_use]
    pub fn rasmi(&self) -> Option<RasmiRules> {
        self.longevity.map(|rules| rules.rasmi)
    }

    /// The choices the wheel of time's span is read under, when the
    /// longevity readings are asked for.
    #[must_use]
    pub fn chakrayus(&self) -> Option<ChakrayusRules> {
        self.longevity.map(|rules| rules.chakrayus)
    }

    /// The choices the two ashtakavarga spans are read under, when the
    /// longevity readings are asked for.
    #[must_use]
    pub fn ashtakavarga(&self) -> Option<AshtakavargaAyusRules> {
        self.longevity.map(|rules| rules.ashtakavarga)
    }

    /// Every longevity choice together, when the readings are asked for.
    pub(crate) const fn longevity_rules(&self) -> Option<LongevityRules> {
        self.longevity
    }
}

/// A rule that held on a chart, and what it answered.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Present<'r> {
    /// Which rule.
    #[serde(serialize_with = "teistro_rules::key_of")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub rule: &'r Rule,
    /// What it answered.
    pub result: RuleResult,
}

/// The longevity readings of a chart (BPHS chs. 43 and 44).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Longevity {
    /// The three pairs, when the chart carried its hora lagna.
    pub three_pairs: Option<ThreePairs>,
    /// The three spans and Jeevasarman's, which is worked as Pindayu.
    pub ayurdaya: Ayurdaya,
    /// The seven grahas' rays, the class of life their sum gives, and
    /// Rasmija ayurdaya.
    pub rasmi: Rasmi,
    /// The span the dashas give from birth (*Jataka Parijata* ch. 5 v. 27):
    /// Vimshottari's balance and the eight dashas after it; none where the
    /// chart's Vimshottari has no balance.
    pub dasayus: Option<Dasayus>,
    /// The span the wheel of time gives (*Jataka Parijata* ch. 5 v. 26):
    /// each graha's ch. 17 years in proportion to what of its nakshatra is
    /// still to run.
    pub chakrayus: Chakrayus,
    /// The two ashtakavarga spans (*Jataka Parijata* ch. 10 vv. 44 to 48,
    /// 70 and 71): each graha's years from its own reduced ashtakavarga,
    /// their sum, and the gathered ashtakavarga's; none where the chart
    /// carries no ashtakavarga.
    pub ashtakavarga: Option<AshtakavargaAyus>,
    /// Which span the strongest of the seven grahas and the lagna names
    /// (*Jataka Parijata* ch. 5 v. 33), the lagna weighed by its Bhava
    /// bala in rupas.
    pub choice: SpanChoice,
    /// The marakas.
    pub marakas: Marakas,
}

/// What a chart answers by rule.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RulesReading<'r> {
    /// Every rule of the set that held, in the set's order.
    pub present: Vec<Present<'r>>,
    /// The twelve house readings, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub houses: Option<Vec<HouseReading<'r>>>,
    /// The longevity readings, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longevity: Option<Longevity>,
    /// The inputs a rule named that this chart could not have — `points` for
    /// a birth with no sunrise — every rule reading one having answered false.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreadable: Vec<&'static str>,
}

/// The `RULES` section of a chart's dossier: every rule that held on it,
/// a row a rule (`03-design/dossier.md`). A chart request writes it from
/// its `rules` record; a caller of [`crate::dossier::dossier`] passes it as
/// an extra section with the chart's own reading.
///
/// ```no_run
/// # fn read(document: &teistro::Document, reading: &teistro::RulesReading<'_>) -> Result<(), teistro::Error> {
/// use teistro::dossier::{DossierOptions, dossier};
///
/// let rules = teistro::RulesSection::new(Some(reading));
/// let text = dossier(document, &DossierOptions::default(), &[&rules])?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RulesSection<'a, 'r> {
    reading: Option<&'a RulesReading<'r>>,
}

impl<'a, 'r> RulesSection<'a, 'r> {
    /// The section's key, as a dossier record names it.
    pub const KEY: &'static str = "RULES";

    /// The section of `reading`; `None`, a chart read under no rules,
    /// whose dossier has no `RULES` to write.
    #[must_use]
    pub const fn new(reading: Option<&'a RulesReading<'r>>) -> RulesSection<'a, 'r> {
        RulesSection { reading }
    }
}

impl crate::dossier::DossierSection for RulesSection<'_, '_> {
    fn key(&self) -> &'static str {
        RulesSection::KEY
    }
    fn columns(&self) -> &[crate::dossier::Column] {
        use crate::dossier::{Column, column};
        const COLUMNS: [Column; 7] = [
            column("rule", "the rule's key"),
            column(
                "category",
                "the rule's kind, as its pack names it (neecha-bhanga, positional, …)",
            ),
            column(
                "status",
                "active, partially-cancelled or fully-cancelled after its cancellations",
            ),
            column(
                "grahas",
                "the bodies its conditions consulted, `,`-separated",
            ),
            column("houses", "the houses they stand in, `,`-separated"),
            column("severity", "how grave, where the rule says"),
            column("source", "the text, chapter and verse it is read from"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "the `rules` record on the chart request"
    }
    fn rows(&self, _: &crate::Document, _: &crate::dossier::View) -> Option<Vec<Vec<String>>> {
        let joined = |items: Vec<String>| {
            if items.is_empty() {
                String::from("-")
            } else {
                items.join(",")
            }
        };
        Some(
            self.reading?
                .present
                .iter()
                .map(|one| {
                    let source = &one.rule.source;
                    let mut cited = source.text.clone();
                    if let Some(chapter) = &source.chapter {
                        cited.push_str(" ch. ");
                        cited.push_str(chapter);
                    }
                    if let Some(verse) = &source.verse {
                        cited.push_str(" v. ");
                        cited.push_str(verse);
                    }
                    vec![
                        one.rule.key.clone(),
                        one.rule.category.clone(),
                        one.result
                            .status
                            .map_or_else(|| String::from("-"), |status| status.key().to_owned()),
                        joined(
                            one.result
                                .participants
                                .iter()
                                .map(|body| body.key().to_owned())
                                .collect(),
                        ),
                        joined(
                            one.result
                                .houses
                                .iter()
                                .map(|house| house.get().to_string())
                                .collect(),
                        ),
                        one.result
                            .severity
                            .map_or_else(|| String::from("-"), |severity| severity.to_string()),
                        cited,
                    ]
                })
                .collect(),
        )
    }
}
