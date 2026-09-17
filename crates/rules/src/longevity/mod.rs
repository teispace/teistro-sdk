//! How long a life the texts give a chart, read from the kernel: the three
//! pairs' class of life (BPHS ch. 43 vv. 33 to 50), the three arithmetic
//! spans, Pindayu, Nisargayu and Amsayu (vv. 4 to 32), and the marakas whose
//! periods the span is read against (ch. 44 vv. 2 to 24).
//!
//! Both are readings of an [`Evaluator`](crate::Evaluator) and not rules: they
//! need a chart's degrees and a strength comparison, and they answer with
//! numbers, where a rule answers present or not.

mod ayurdaya;
mod maraka;
mod pairs;

pub use ayurdaya::{
    Ayurdaya, AyurdayaRules, Combine, Contribution, Giver, Method, Nisarga, Reductions, Span,
    by_exaltation, by_navamsha, full_years, visible_half_share,
};
pub use maraka::{Brings, Marakas, Presentation, Reason, Reasons, Vulnerability, age_span};
pub use pairs::{
    Basis, Decided, Pair, PairReading, Rectification, SaturnAmongThem, Shift, ThreePairs,
    ThreePairsRules, class_of, rectify, years_of,
};
