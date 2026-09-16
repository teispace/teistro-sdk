//! The Teistro SDK's rules kernel (`docs/03-design/rules-engine.md`).
//!
//! A rule is data: conditions in a small language over a chart — where a body
//! stands, its dignity, its company, the lords of houses, the chara karakas —
//! and cancellations. A condition can be about a body or a sign reached from
//! one ([`reference`]): the lord of the seventh from the upapada, the fourth
//! from the Karakamsha. The kernel reads a rule strictly ([`language`]),
//! evaluates it over a [`RuleChart`] under [`Readings`], one choice at each
//! place the language leaves a meaning open, and answers a [`RuleResult`]:
//! whether it is present, the bodies it consulted, their houses and the
//! cancellations that held.
//!
//! The language is the recording engine's, and its reading is the default:
//! over the conformance corpus's 93 charts the kernel reproduces all 605 of
//! its yoga rules' 55 521 decisions, their participants and their
//! cancellations (`tests/baseline.rs`, `03-design/yogas-measured.md`).
//!
//! ```
//! use teistro_rules::{Condition, Evaluator, Readings, Rule};
//!
//! let rule: Rule = serde_json::from_str(r#"{
//!     "key": "GURU_IN_KENDRA",
//!     "category": "example",
//!     "source": { "text": "an example" },
//!     "conditions": [{ "type": "planet-in-kendra", "planet": "JUPITER" }]
//! }"#)?;
//! assert!(matches!(rule.conditions[0], Condition::PlanetInKendra { .. }));
//! # let _ = Evaluator::new;
//! # let _ = Readings::RECORDING_ENGINE;
//! # Ok::<(), serde_json::Error>(())
//! ```

pub mod chart;
mod dwigraha;
pub mod eval;
pub mod house;
pub mod language;
pub mod reference;
pub mod rule;
pub mod shipped;
pub mod table;
pub mod trace;

pub use chart::{
    AspectGathering, Benefics, Bhaga, Conjunction, DignityMatch, Eclipse, EightKarakas, Gathering,
    Houses, Limb, NodeMotion, NodeSides, Panchanga, Placement, PointAt, Readings, RuleChart, Span,
    Spans, StrengthMeasure, Strengths, Upapada, VargaSigns,
};
pub use eval::{Evaluator, Found, Participants, RuleResult};
pub use house::{COMPOSITIONS, Composition, Held, HouseReading, Kind};
pub use language::{
    ArgalaPlace, Body, Condition, EclipseKind, EvidenceRank, House, Karaka, KarakaScheme, NodeSide,
    Pada, Source,
};
pub use reference::{BodyRef, BodySubject, Class, SignRef, Subject};
pub use rule::{
    Cancellation, Group, NetStatus, Outcome, Rule, Scope, Severity, Unit, check_references,
};
pub use table::{SignDegree, Table, TableKey, Tables};
pub use trace::{Explanation, Resolved, Step};
