//! The Teistro SDK for Rust: the surface a Rust consumer reads.
//!
//! Node, Dart and Python read `sdk.<area>.<operation>` off a context.
//! This crate is that, in Rust, and it is designed rather than improvised
//! — `docs/03-design/rust-consumer-surface.md` decides its shape, from
//! the measurement in `rust-consumer-surface-measured.md`.
//!
//! What the measurement found, and why this crate exists: a Rust consumer
//! could build a context before this — `TsContext::build` is `pub` — and
//! could not *use* one, because converting a date meant
//! `ts_calendar_convert` with three raw pointers. So Rust had a context
//! it could build and could not use, and two of the nine crates such a
//! context needs were held by the C boundary and by nothing else.
//!
//! ```
//! # // The built-in ephemeris is a feature, and a doctest sees the
//! # // crate's features, so this one compiles away with it rather than
//! # // failing a `--no-default-features` run.
//! # #[cfg(feature = "builtin-ephemeris")] fn run() -> Result<(), teistro::Error> {
//! use teistro::catalogue::Calendar;
//! use teistro::{CalendarDate, Context, Ephemeris};
//!
//! let sdk = Context::builder()
//!     .profile("nepali-default")
//!     .locale("ne-Deva-NP")
//!     .ephemeris([Ephemeris::Builtin])
//!     .build()?;
//!
//! // 14 April 2015 is 1 Baisakh 2072 BS.
//! let day = CalendarDate::defined(Calendar::Gregorian, 2015, 4, 14);
//! let bs = sdk.calendar().convert(&day, Calendar::BikramSambat)?;
//! assert_eq!((bs.year, bs.month, bs.day), (2072, 1, 1));
//! # Ok(()) }
//! # #[cfg(feature = "builtin-ephemeris")] run().unwrap();
//! ```
//!
//! **It composes the crates; it does not call the SDK's own C ABI.** A
//! Rust consumer reaching itself through a C boundary would write a
//! request struct so the boundary could decode it, and decode a result
//! blob to read numbers the crates had already returned as `JulianDay`
//! and `Longitude`. Eight of the boundary's forty-six entry points are
//! that marshalling and nothing else.

mod area;
mod context;
mod ephemeris;
mod reading;
mod render;
mod scale;

pub use area::{
    AlmanacArea, CalendarArea, ChartArea, EngineArea, FrameArea, InterpretArea, Interpreted,
    IntlArea, KeysArea, Plans, TimeArea,
};
mod gochar_request;
mod plan_request;
mod rule_request;
mod rules_bridge;
mod varsha;

pub use context::{Context, ContextBuilder};
pub use ephemeris::Ephemeris;
pub use reading::ChartRequest;
pub use scale::{Conversion, Scale};

// The types an operation takes and answers with are the crates' own, and
// are re-exported so a consumer needs one dependency rather than five.
// Re-exported and **not** wrapped: a newtype over `JulianDay` would be a
// second type with the same invariant and no new one (ADR-0023).
//
// The rule this list keeps is checkable rather than a judgement:
// **every type an area's signature names is reachable from this crate
// root.** An operation answering a `CalendarResolution` a consumer
// cannot name is an operation whose answer cannot be matched on, and
// the examples found four of those before this list did —
// `CalendarResolution`, `Envelope`, `ChartFoundation` and `Panchanga`.
// `rust-consumer-surface-measured.md`'s *every type an area's signature
// names is reachable from the crate root* holds it now, born red on
// exactly those four.
pub use teistro_astro::DeltaTModel;
pub use teistro_astro::delta_t::DeltaT;
pub use teistro_calendar::{CalendarDate, FixedDay, Weekday};
pub use teistro_chart::day::ChartDay;
pub use teistro_chart::foundation::{ChartAngles, ChartFoundation, GrahaPosition};
pub use teistro_core::catalogue;
// `canonical_json` and `content_hash` with them: a stored chart keeps
// the bytes the provenance was hashed from, and without these a
// consumer could hold an `Envelope` and not reproduce its own hash.
pub use teistro_core::envelope::{
    CalendarResolution, Envelope, Hash, Provenance, canonical_json, content_hash,
};
pub use teistro_core::error::{Error, Status};
// A house, 1 to 12: the rule language reads one, a placement names one,
// and a Tajika yoga is asked about one, so it lives in `core` and is
// named here once for all three.
pub use teistro_core::house::House;
pub use teistro_core::interval::Interval;
pub use teistro_core::key::KeyId;
pub use teistro_core::quantity;
pub use teistro_core::settings;
pub use teistro_panchanga::almanac::Panchanga;
// What a reading answers with, and the sections it holds: the document
// is `teistro-serial`'s, and an operation that answers one must let a
// consumer name it and every section of it.
pub use teistro_aspect::{Aspects, Drishti};
pub use teistro_houses::Houses;
pub use teistro_points::Points;
// The derived points' own functions — the padas and the arudha lagna among
// them, which a consumer counting a pada to another lord needs
// (`points::arudha::arudha_by`, crux C135).
pub use teistro_points as points;
pub use teistro_port_ephemeris::native::{NativeFunction, NativeManifest};
pub use teistro_serial::{Document, Sealed};
// The document's JSON Schema, for a consumer who stores one and wants to
// check it before reading it back (`03-design/document-schema.md`).
pub use teistro_serial::schema;
pub use teistro_state::{Anka, GrahaState, Sayanadi};
pub use teistro_vargas::chart::{Axis, VargaChart};
// Chart geometry: the layouts a chart is drawn in, a consumer's own, and a
// chart placed in one (`03-design/chart-geometry.md`).
pub use teistro_geometry as geometry;
pub use teistro_geometry::{Drawing, Layout, Layouts, Placed};
// The first-party renderer: a drawing as SVG, themed as data
// (`03-design/render-svg.md`). Not behind a feature: it is pure Rust with
// no dependency the façade lacks, and a consumer who never calls it has
// its code removed by the linker.
pub use teistro_render_svg as render_svg;
// Dashas: a system as a row, the balance at birth, and the period tree read
// without building it (`03-design/dasha-kernels.md`).
pub use crate::plan_request::PlanRequest;
// The annual charts a birth is asked for, in one call
// (`03-design/annual-chart.md`).
pub use crate::rule_request::{
    Longevity, Present, RuleReadings, RuleRequest, RuleSet, RulesReading, ShippedRules,
};
pub use crate::rules_bridge::{
    MarakaWindow, RuleInputs, maraka_windows, rule_chart, rule_periods, rule_vargas,
};
pub use crate::varsha::{
    AnnualChart, AnnualPlace, Askable, Asked, SahamStrengthReadings, Varsha, VarshaRequest,
    VarshaYear,
};
pub use teistro_dasha as dasha;
// Gochar: the transits read from the natal Moon (`03-design/gochar.md`).
pub use crate::gochar_request::GocharRequest;
pub use teistro_dasha::{
    DashaCursor, DashaDefinition, DashaReading, PeriodRow, RashiDefinition, Share, Timeline,
    UduDefinition, YearDasha, YearRing,
};
pub use teistro_gochar as gochar;
pub use teistro_gochar::GocharFrom;
pub use teistro_interpret as interpret;
pub use teistro_interpret::{Item, Plan};
pub use teistro_rules as rules;
pub use teistro_rules::{HouseReading, RuleChart, RuleResult, Strengths};
pub use teistro_tajika as tajika;
pub use teistro_tajika::{
    Affliction, AnnualDasha, AnnualDashaRules, AnnualStates, Bala, Between, Chosen, Claim,
    Drishti as TajikaDrishti, DrishtiRules, Favour, Held, MoonBenefic, MoonMayRule, MoonPartner,
    MuddaBalance, Muntha, MunthaDegree, Natal, NoneAspects, Office, OfficeBearers, Panchavargiya,
    Pravesha, Qualification, Reading as VarshaReading, Relation as TajikaRelation, Strength,
    SubDegree, TambiraMover, Tied as VarsheshaTied, Varshesha, VarsheshaRules, YearClock, YearYoga,
    YearYogas, Yoga as TajikaYoga, YogaRules,
};
// The sahams: a formula over a chart's points, the source's forty-one as a
// table of them, and the readings the tradition divides over
// (`03-design/tajika-sahams.md`).
pub use teistro_tajika::{
    AddSign, HousePoints, RogaReading, Saham, SahamFormula, SahamPlace, SahamPoint, SahamReading,
    SahamRules, SahamTerm, SahamTriple,
};
// The Harsha bala: four places a planet of the annual chart is happy in
// (`03-design/tajika-harsha.md`).
pub use teistro_tajika::{Harsha, HarshaGrade, HarshaRules, VenusPlace};
// A saham's strength, clause by clause, and the readings it is judged under
// (`03-design/tajika-saham-strength.md`).
pub use teistro_tajika::{
    Friendship, SahamNatures, SahamStrength, SahamStrengthRules, StrongClause, WeakClause,
};
// Strength measures: the Ashtakavarga, the Vimshopaka and the Shadbala, each
// with the rules it was read under (`03-design/strength-schemes.md`).
pub use teistro_strength as strength;
// The typed accessor tree: every message of the SDK's locale as a value
// of its own parameters. A **module** tree, because that is what a
// namespace is in Rust — where Node writes
// `ctx.intl.messages.sdk.reason.grahaInBhava({ … })`.
pub use teistro_intl::messages;
// What the locale area takes and answers with.
pub use teistro_intl::source::{Entity, Tree};
pub use teistro_intl::translit::Script;
pub use teistro_intl::{Intl, Loaded, Params, Rendered, TypedMessage, Value, params};
// Making a pack, which is the other half of loading one. `Loaded` is what
// `load_pack` answers with and was published already; this is what a
// consumer with locale sources of its own — or with the SDK's readings
// corpus, which is loaded rather than embedded — builds one from
// (`03-design/interpretation-records.md` §3).
pub use teistro_intl::pack;
// What `positions` takes and answers with, and what an ephemeris of
// your own implements. Re-exported because a consumer needing five
// dependencies to call one operation is the thing this crate exists to
// stop -- and the test for `positions` reached past it before these
// were here, which is how the gap was noticed.
pub use teistro_astro::completion::{Completed, Completion};
pub use teistro_core::time::UtcOffset;
// And what an ephemeris of your own needs to *implement* the port, not
// merely to call it: writing a provider is a first-class use of this
// crate (ADR-0029 hands a Rust consumer the adapters as `rlib`s), so a
// consumer writing one should need this dependency and no other.
pub use teistro_port_ephemeris::capabilities::{Astronomy, DistanceUnit, SpeedModel};
pub use teistro_port_ephemeris::columns::{EphemerisKind, Source};
pub use teistro_port_ephemeris::provider::validate;
pub use teistro_port_ephemeris::{
    Body, Capabilities, Cell, CellStatus, Centre, Coordinates, Corrections, EphemerisProvider,
    Equinox, Frame, Identity, Overrides, PositionColumns, PositionRequest, ProviderError,
    TimeScale, Zodiac,
};
// A chart's day and an almanac's are one record, `LocalDay`, and a
// consumer reading either names its type from here rather than from the
// time crate underneath.
pub use teistro_time::{
    CivilDateTime, CivilTime, DayState, LocalDay, PolarKind, Resolved, ZoneResolution, ZoneSpec,
};

/// The Julian day at the UTC midnight that begins a fixed day.
///
/// A **free function** and not an area's operation, as it is in the
/// other three bindings: it takes no context, because a fixed day and a
/// Julian day are two spellings of the same integer and nothing about a
/// profile or a locale can change the arithmetic.
#[must_use]
pub fn jd_of_fixed(fixed: FixedDay) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a fixed day within any calendar's range is exact in an f64"
    )]
    let day = fixed.get() as f64;
    day + FixedDay::JD_EPOCH
}

/// The fixed day a Julian day falls in, and the fraction of that day
/// elapsed since its midnight.
#[must_use]
pub fn fixed_of_jd(jd: f64) -> (FixedDay, f64) {
    FixedDay::from_local_jd(jd)
}

// `BUNDLES`: the SDK's locales, built from `i18n/` by this crate's build
// script so a consumer needs no files to render its messages (ADR-0010).
include!(concat!(env!("OUT_DIR"), "/bundles.rs"));
