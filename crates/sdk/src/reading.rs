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

use teistro_core::catalogue::{Catalogued, ChartKind, Varga};
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

    /// Every section, and every divisional chart.
    ///
    /// What a consumer storing a chart for later wants, and what the
    /// parity runner asks for: the widest document the SDK can produce.
    #[must_use]
    pub fn with_everything(self) -> ChartRequest {
        self.with_every_varga()
            .with_panchanga()
            .with_state()
            .with_aspects()
            .with_points()
            .with_houses()
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
}
