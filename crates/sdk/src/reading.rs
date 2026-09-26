//! What a chart reading asks for: where, what kind, and which of the
//! document's sections.
//!
//! `03-design/chart-reading.md` §4 decides the shape. What a consumer
//! writes is a **builder**, not a bit set, because ADR-0023's
//! type-safety rule applies to a request as much as to an answer and
//! because one of the fields is a list — "which divisional charts" is a
//! real question with twenty-one answers, and a flag cannot hold one.
//! What the builder *stores* for the five yes-or-no sections is a bit
//! set, which is the same split the frame already has: `ts_frame_pack`
//! packs, and no consumer writes bits.
//!
//! One naming rule, and it is the whole of the API: **every builder
//! method is `with_*` and every reader is the bare field name.** Without
//! it a setter and its getter want the same word, and the codebase ends
//! up with `kind` beside `chart_kind` for no reason a reader can see.

#[cfg(doc)]
use teistro_core::catalogue::ChartLayout;
use teistro_core::catalogue::{Catalogued, ChartKind, Varga};
use teistro_core::key::KeyId;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;

/// Which sections of the document to compute, beside the foundation, as
/// a set.
///
/// **A bit set and not five flags**, which clippy's `excessive_bools`
/// asked for and which is right for a second reason: this is the shape
/// the C boundary will carry (`03-design/chart-reading.md` §5 —
/// `TsChartRequest` gains a `sections: u32`), so the internal
/// representation and the wire one are the same thing and the crossing
/// is a cast rather than a translation. It is the same shape as the
/// port's own `Overrides`.
///
/// Private, because a consumer names its members one at a time through
/// [`ChartRequest`]'s builder and never as a set: the names are the
/// documentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Sections(u32);

impl Sections {
    /// The almanac of the day the chart belongs to.
    pub(crate) const PANCHANGA: Sections = Sections(1);
    /// What each graha is, as opposed to where it is.
    pub(crate) const STATE: Sections = Sections(1 << 1);
    /// Which bodies reach which.
    pub(crate) const ASPECTS: Sections = Sections(1 << 2);
    /// The upagrahas and the special lagnas.
    pub(crate) const POINTS: Sections = Sections(1 << 3);
    /// The twelve bhavas under both readings.
    pub(crate) const HOUSES: Sections = Sections(1 << 4);
    /// Each graha's Ashtakavarga, their sum, reductions and pindas.
    pub(crate) const ASHTAKAVARGA: Sections = Sections(1 << 5);
    /// Each graha's Vimshopaka under the four schemes.
    pub(crate) const VIMSHOPAKA: Sections = Sections(1 << 6);
    /// Each graha's six strengths.
    pub(crate) const SHADBALA: Sections = Sections(1 << 7);
    /// Each house's strength.
    pub(crate) const BHAVA_BALA: Sections = Sections(1 << 8);
    /// The names each graha earns by its good vargas.
    pub(crate) const VAISESHIKAMSA: Sections = Sections(1 << 9);
    /// What each graha's placement says of its dasha.
    pub(crate) const DASHA_PHALA: Sections = Sections(1 << 10);
    /// Jaimini's significators.
    pub(crate) const JAIMINI: Sections = Sections(1 << 11);

    /// The union.
    const fn with(self, other: Sections) -> Sections {
        Sections(self.0 | other.0)
    }

    /// Whether one section is in the set.
    pub(crate) const fn has(self, one: Sections) -> bool {
        self.0 & one.0 == one.0
    }
}

/// What a chart reading asks for: the record it is cast for, and what to
/// read from it.
///
/// Named for the **request** and not the answer, as `PositionRequest`
/// is, and renamed from `Reading` when it met `teistro_chart::Reading`
/// in the boundary's own `use` list — which is a bhava's reading, sandhi
/// or madhya, and a word this domain had already spent. The answer is a
/// `Document` and the operation is `reading`.
///
/// **Nothing but the foundation is on by default**, which is the "pay
/// for what you ask for" half of the maintainer's no-dead-ends rule. A
/// caller who wants a birth chart should not pay for twenty-one
/// divisional charts, and the sections are cheap to leave out because
/// every one of them is a pure function of the foundation and the
/// settings — an unasked section costs its producer not being called
/// (`03-design/chart-reading.md` §2).
///
/// The foundation itself is not optional and has no flag: every other
/// section is computed from it, and a document without one cannot be
/// checked against anything.
///
/// ```
/// use teistro::catalogue::{ChartKind, Varga};
/// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
/// use teistro::{ChartRequest, UtcOffset};
///
/// let place = Place::new(
///     Latitude::try_new(27.7172)?,
///     Longitude::try_new(85.324)?,
///     Altitude::try_new(1400.0)?,
/// );
/// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20700)?)
///     .with_kind(ChartKind::Natal)
///     .with_vargas([Varga::D9, Varga::D10])
///     .with_state()
///     .with_houses();
/// assert_eq!(request.vargas().len(), 2);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartRequest {
    place: Place,
    offset: UtcOffset,
    kind: ChartKind,
    vargas: Vec<Varga>,
    drawings: Vec<(KeyId, Varga)>,
    dashas: Vec<KeyId>,
    pub(crate) sections: Sections,
}

impl ChartRequest {
    /// A reading at a place, under a local clock: the foundation and
    /// nothing else.
    #[must_use]
    pub fn at(place: Place, offset: UtcOffset) -> ChartRequest {
        ChartRequest {
            place,
            offset,
            kind: ChartKind::Natal,
            vargas: Vec::new(),
            drawings: Vec::new(),
            dashas: Vec::new(),
            sections: Sections::default(),
        }
    }

    /// The kind of chart to found; `Natal` unless said otherwise.
    #[must_use]
    pub const fn with_kind(mut self, kind: ChartKind) -> ChartRequest {
        self.kind = kind;
        self
    }

    /// The divisional charts to compute, in the order given.
    ///
    /// **Replaces rather than accumulates**, as every setter here does:
    /// a builder that accumulated would make `with_vargas([])` mean
    /// nothing at all, and "none" has to be sayable.
    #[must_use]
    pub fn with_vargas(mut self, vargas: impl IntoIterator<Item = Varga>) -> ChartRequest {
        self.vargas = vargas.into_iter().collect();
        self
    }

    /// Every divisional chart the catalogue ships.
    #[must_use]
    pub fn with_every_varga(self) -> ChartRequest {
        self.with_vargas(Varga::all().iter().copied())
    }

    /// The almanac of the day the chart belongs to.
    ///
    /// The one section that is a **second crossing**: an almanac
    /// searches for sunrise, the Moon's rises and the limbs' boundaries,
    /// where every other section is arithmetic on the foundation.
    #[must_use]
    pub const fn with_panchanga(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::PANCHANGA);
        self
    }

    /// What each graha is, as opposed to where it is: the avasthas, the
    /// combustion, the planetary war.
    #[must_use]
    pub const fn with_state(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::STATE);
        self
    }

    /// Which bodies reach which, under the drishti table the settings
    /// name.
    #[must_use]
    pub const fn with_aspects(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::ASPECTS);
        self
    }

    /// What `rules` read of a chart, and nothing they do not: the graha
    /// states always, the points when a rule names one, the Shadbala when a
    /// rule reads strength, and every divisional chart a rule steps into —
    /// derived from the rules rather than listed, so a consumer's own rules
    /// ask for their own. Read the document back with
    /// [`RuleInputs::of`](crate::RuleInputs::of).
    ///
    /// A birth on a day without a sunrise has no special lagnas, which count
    /// from sunrise, so a reading asking for points there is refused.
    #[must_use]
    pub fn with_rule_inputs<'r>(
        self,
        rules: impl IntoIterator<Item = &'r teistro_rules::Rule>,
    ) -> ChartRequest {
        self.rule_inputs(rules, true)
    }

    /// What `rules` read, the points only where `points` allows: a birth with
    /// no sunrise has no special lagnas, and a reading for rules alone is read
    /// without them rather than refused (`03-design/rules-at-the-boundary.md`
    /// §5).
    pub(crate) fn rule_inputs<'r>(
        self,
        rules: impl IntoIterator<Item = &'r teistro_rules::Rule>,
        points: bool,
    ) -> ChartRequest {
        let (mut named_points, mut strength, mut limbs) = (false, false, false);
        let mut asked = self.vargas.clone();
        for rule in rules {
            named_points |= rule.reads_points();
            strength |= rule.reads_strength();
            limbs |= rule.reads_panchanga();
            for varga in rule.vargas() {
                if !asked.contains(&varga) {
                    asked.push(varga);
                }
            }
        }
        let mut request = self.with_state().with_vargas(asked);
        if named_points && points {
            request = request.with_points();
        }
        if strength {
            request = request.with_shadbala();
        }
        // The limb a rule reads is the one running at the birth, which the
        // day's almanac answers at the instant: without this section the
        // 11 shipped rules that read a limb say "the chart has none" and a
        // consumer asking for exactly what the rules read has no way to
        // give them one (`rules_bridge::birth_limbs`).
        if limbs {
            request = request.with_panchanga();
        }
        request
    }

    /// Whether the request itself asks for the derived points.
    pub(crate) const fn asks_points(&self) -> bool {
        self.sections.has(Sections::POINTS)
    }

    /// The upagrahas and the special lagnas.
    ///
    /// The one section whose producer takes more than the foundation and
    /// the settings: Saturn's eighth divides the day's arc and asks for
    /// the ascendant at each division, so a lagna at an instant that is
    /// not the birth is what it needs and what `Founder::ascendant_at`
    /// answers.
    #[must_use]
    pub const fn with_points(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::POINTS);
        self
    }

    /// The twelve bhavas under both readings, with every body placed in
    /// each.
    #[must_use]
    pub const fn with_houses(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::HOUSES);
        self
    }

    /// Each graha's Ashtakavarga, the sarvashtakavarga, and their
    /// reductions and pindas under the settings' `strength.shodhana` and
    /// `strength.ekadhipatya` (`03-design/ashtakavarga-measured.md`).
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_ashtakavarga();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_ashtakavarga(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::ASHTAKAVARGA);
        self
    }

    /// Each graha's Vimshopaka, its strength out of 20 across the
    /// divisional charts under the shadvarga, saptavarga, dashavarga and
    /// shodashavarga, each varga scored under the settings'
    /// `strength.vimshopaka` (`03-design/vimshopaka-measured.md`).
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_vimshopaka();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_vimshopaka(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::VIMSHOPAKA);
        self
    }

    /// Each graha's Shadbala: its six strengths in virupas, their sum in
    /// rupas and whether it reaches what it must, under the settings'
    /// `strength.*` readings of BPHS ch. 27 (`03-design/shadbala-measured.md`).
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_shadbala();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_shadbala(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::SHADBALA);
        self
    }

    /// Each bhava's Bhava bala: its lord's Shadbala, its Dig and drishti
    /// balas and, under BPHS's reading, its special rules, under the settings'
    /// `strength.bhava_*` readings (`03-design/bhava-bala-measured.md`). The
    /// Shadbala it reads is computed alongside when the request does not ask
    /// for it too.
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_bhava_bala();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_bhava_bala(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::BHAVA_BALA);
        self
    }

    /// Each graha's Vaiseshikamsa: how many of each scheme's vargas are good
    /// for it, the name that count earns, and whether it is combust or
    /// defeated in war and so earns it without its auspiciousness (BPHS ch. 6
    /// vv. 42 to 53).
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_vaiseshikamsa();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_vaiseshikamsa(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::VAISESHIKAMSA);
        self
    }

    /// What each graha's placement says of its dasha: its benefic and
    /// malefic points in the seven vargas and whether its rasi place is
    /// auspicious (BPHS ch. 28 vv. 7 to 10), where in its dasha its effects
    /// come, and whether its placement makes the dasha favourable or
    /// unfavourable (ch. 47 vv. 3 to 6), read under `dasha.shanta_sign`.
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_dasha_phala();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_dasha_phala(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::DASHA_PHALA);
        self
    }

    /// Jaimini's significators: the karakamsha, the Atmakaraka's navamsha
    /// sign under `jaimini.chara_karakas`, with every graha's house from it
    /// in the rasi chart and the navamsha; and the Brahma graha under
    /// `jaimini.brahma` and `jaimini.node_co_lordship`, or why the rule
    /// found none (BPHS ch. 33 v. 1, ch. 46 vv. 170 to 173).
    ///
    /// ```
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20_700)?).with_jaimini();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_jaimini(mut self) -> ChartRequest {
        self.sections = self.sections.with(Sections::JAIMINI);
        self
    }

    /// The charts to draw, each a layout and which chart to place in it,
    /// in the order given: `D1` for the founded chart, or a divisional one.
    ///
    /// A layout is a shipped [`ChartLayout`] or the id of one the context was
    /// built with (`ContextBuilder::layout`). Pairs rather than a list of
    /// layouts, so drawing the D9 in North Indian does not also draw every
    /// other chart in it, and a pair that cannot be drawn (a Western wheel of
    /// the D9, which has no degrees) is refused by name when the reading runs.
    ///
    /// Replaces rather than accumulates, as every setter here does.
    ///
    /// ```
    /// use teistro::catalogue::{ChartLayout, Varga};
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20700)?)
    ///     .with_drawings([(ChartLayout::NorthIndian, Varga::D1), (ChartLayout::NorthIndian, Varga::D9)]);
    /// assert_eq!(request.drawings().len(), 2);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn with_drawings<L: Into<KeyId>>(
        mut self,
        drawings: impl IntoIterator<Item = (L, Varga)>,
    ) -> ChartRequest {
        self.drawings = drawings
            .into_iter()
            .map(|(layout, varga)| (layout.into(), varga))
            .collect();
        self
    }

    /// The dashas to compute, a system each, in the order given: each one's
    /// balance at birth and its periods to the depth the settings give it
    /// (`dasha.depth`, three levels by default).
    ///
    /// A system is a catalogued member or the id of one the context was built
    /// with (`ContextBuilder::dasha_system`). A system the catalogue names and
    /// no row implements yet, or an id the context never registered, is
    /// refused by its place in the request when the reading runs. Replaces
    /// rather than accumulates, as every setter here does.
    ///
    /// ```
    /// use teistro::catalogue::DashaSystem;
    /// use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro::{ChartRequest, UtcOffset};
    ///
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let request = ChartRequest::at(place, UtcOffset::try_from_seconds(20700)?)
    ///     .with_dashas([DashaSystem::Vimshottari]);
    /// assert_eq!(request.dashas(), [DashaSystem::Vimshottari.into()]);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn with_dashas<S: Into<KeyId>>(
        mut self,
        dashas: impl IntoIterator<Item = S>,
    ) -> ChartRequest {
        self.dashas = dashas.into_iter().map(Into::into).collect();
        self
    }

    /// Every section, and every divisional chart.
    ///
    /// What a consumer storing a chart for later wants, and what the
    /// parity runner asks for: the widest document the SDK can produce for
    /// any chart. Every dasha system this build computes on every chart
    /// ([`teistro_dasha::systems_every_chart_gives`]): the Sthira dasa,
    /// which a chart with no Brahma graha refuses, is asked for by name.
    /// **No drawings**:
    /// those are named pairs, and every layout times every
    /// chart is a hundred and twenty-six placements nobody asked for.
    #[must_use]
    pub fn with_everything(self) -> ChartRequest {
        self.with_every_varga()
            .with_dashas(teistro_dasha::systems_every_chart_gives())
            .with_panchanga()
            .with_state()
            .with_aspects()
            .with_points()
            .with_houses()
            .with_ashtakavarga()
            .with_vimshopaka()
            .with_vaiseshikamsa()
            .with_shadbala()
            .with_bhava_bala()
            .with_dasha_phala()
            .with_jaimini()
    }

    /// The place the chart is cast for.
    #[must_use]
    pub const fn place(&self) -> &Place {
        &self.place
    }

    /// The local clock the day's date is read in.
    #[must_use]
    pub const fn offset(&self) -> UtcOffset {
        self.offset
    }

    /// What kind of chart this is.
    #[must_use]
    pub const fn kind(&self) -> ChartKind {
        self.kind
    }

    /// The divisional charts asked for.
    #[must_use]
    pub fn vargas(&self) -> &[Varga] {
        &self.vargas
    }

    /// The dashas asked for, as catalogue or registered ids.
    #[must_use]
    pub fn dashas(&self) -> &[KeyId] {
        &self.dashas
    }

    /// The charts to draw, as layout ids and the chart drawn in each.
    #[must_use]
    pub fn drawings(&self) -> &[(KeyId, Varga)] {
        &self.drawings
    }
}
