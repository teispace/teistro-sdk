//! Which span of life the strongest of the seven grahas and the lagna names
//! (*Jataka Parijata* ch. 5 v. 33, C308).
//!
//! "Pindaja, Nisargaja, Rasmija, Bhinnashtakavargaja, Kalachakraja,
//! Nakshatraja, Samudayashtakavargaja or Amsaja Ayus is to be reckoned
//! according as the Sun, the Moon, Mercury, Mars, Venus, Jupiter, Saturn or
//! the Lagna possesses the greatest strength" (p. 259). Every candidate is
//! reported with its strength and the span it names, and the span's years
//! where the SDK computes that span; which is strongest is a clause of the
//! strengths, never a verdict, and a tie or a missing strength says so
//! rather than falling through to an answer.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;

use super::ayurdaya::Ayurdaya;
use super::chakrayus::Chakrayus;
use super::dasayus::Dasayus;
use super::rasmi::Rasmi;
use crate::chart::Strengths;
use crate::language::Body;

/// The eight spans of life *Jataka Parijata* ch. 5 v. 1 names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ayus {
    /// Pindaja, from each graha's distance from exaltation (vv. 2 to 16).
    Pinda,
    /// Nisargaja, the grahas' natural years (vv. 2 to 16).
    Nisarga,
    /// Rasmija, from the rays (vv. 22 to 25).
    Rasmi,
    /// From each graha's own ashtakavarga.
    Bhinnashtakavarga,
    /// Kalachakraja, through the Kalachakra years (v. 26).
    Kalachakra,
    /// Nakshatraja, the dasas from the Moon's nakshatra (v. 27).
    Nakshatra,
    /// From the gathered ashtakavarga.
    Samudaya,
    /// Amsaja, from the navamshas (vv. 17 to 21).
    Amsa,
}

/// The candidates in the verse's order: the Sun to Saturn as it lists them,
/// then the lagna, each with the span it names.
const NAMES: [(Body, Ayus); 8] = [
    (Body::Graha(Graha::Sun), Ayus::Pinda),
    (Body::Graha(Graha::Moon), Ayus::Nisarga),
    (Body::Graha(Graha::Mercury), Ayus::Rasmi),
    (Body::Graha(Graha::Mars), Ayus::Bhinnashtakavarga),
    (Body::Graha(Graha::Venus), Ayus::Kalachakra),
    (Body::Graha(Graha::Jupiter), Ayus::Nakshatra),
    (Body::Graha(Graha::Saturn), Ayus::Samudaya),
    (Body::Lagna, Ayus::Amsa),
];

impl Ayus {
    /// The span a body names when it is strongest (v. 33); none for the
    /// nodes, which the verse does not weigh.
    #[must_use]
    pub fn named_by(body: Body) -> Option<Ayus> {
        NAMES
            .iter()
            .find(|(by, _)| *by == body)
            .map(|(_, ayus)| *ayus)
    }
}

/// One candidate of v. 33.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Candidate {
    /// The graha or the lagna.
    pub by: Body,
    /// Its strength in the chart's measure, or none where the measure does
    /// not reach it.
    pub strength: Option<f64>,
    /// The span it names.
    pub ayus: Ayus,
    /// That span's years where it is computed: Pinda, Nisarga and Amsa as
    /// the three spans read them, Rasmi as the rays' years, Nakshatra as
    /// the dashas' (v. 27) and Kalachakra as the wheel of time's (v. 26);
    /// none for the two ashtakavarga spans, which the SDK does not yet
    /// compute.
    pub years: Option<f64>,
}

/// The spans already computed, whose years a candidate carries; each left
/// out is a span whose candidate carries none.
#[derive(Clone, Copy, Debug, Default)]
pub struct Computed<'a> {
    /// Pinda, Nisarga and Amsa.
    pub ayurdaya: Option<&'a Ayurdaya>,
    /// The rays and their years.
    pub rasmi: Option<&'a Rasmi>,
    /// The dashas' span.
    pub dasayus: Option<&'a Dasayus>,
    /// The wheel of time's span.
    pub chakrayus: Option<&'a Chakrayus>,
}

/// The span the strongest names, with every candidate weighed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SpanChoice {
    /// The eight in the verse's order.
    pub candidates: [Candidate; 8],
    /// The strongest, when one is: none when two share the greatest strength
    /// or no candidate has one.
    pub strongest: Option<Body>,
    /// The span the strongest names, when one is.
    pub ayus: Option<Ayus>,
    /// Whether every candidate was weighed. False when the measure reached
    /// only some of them, as Shadbala does not reach the lagna; the
    /// strongest is then the strongest of those weighed.
    pub all_weighed: bool,
}

/// v. 33's choice over a chart's strengths, `lagna` filling the lagna's
/// where the chart's measure does not reach it, with each span's years from
/// those `computed`.
///
/// ```
/// use teistro_rules::longevity::{Ayus, Computed, span_choice};
/// # use teistro_rules::{Body, StrengthMeasure, Strengths};
/// # use teistro_core::catalogue::Graha;
/// // The rays' figure (p. 251): the Sun's 8.154 rupas are the greatest of
/// // the seven and the lagna's 7.345, so it names Pindaja.
/// let strengths = Strengths {
///     measure: StrengthMeasure::Shadbala,
///     of: [Some(8.154), Some(7.289), Some(7.354), Some(7.550), Some(5.678),
///          Some(7.719), Some(5.053), None, None, None],
///     required: [None; 10],
/// };
/// let choice = span_choice(&strengths, Some(7.345), Computed::default());
/// assert_eq!(choice.strongest, Some(Body::Graha(Graha::Sun)));
/// assert_eq!(choice.ayus, Some(Ayus::Pinda));
/// ```
#[must_use]
pub fn span_choice(
    strengths: &Strengths,
    lagna: Option<f64>,
    computed: Computed<'_>,
) -> SpanChoice {
    let spans = computed.ayurdaya;
    let candidates = NAMES.map(|(by, ayus)| {
        let measured = strengths.of.get(by.index()).copied().flatten();
        Candidate {
            by,
            strength: measured.or(if by == Body::Lagna { lagna } else { None }),
            ayus,
            years: match ayus {
                Ayus::Pinda => spans.map(|spans| spans.pindayu.years),
                Ayus::Nisarga => spans.map(|spans| spans.nisargayu.years),
                Ayus::Amsa => spans.map(|spans| spans.amsayu.years),
                Ayus::Rasmi => computed.rasmi.map(|rays| rays.years),
                Ayus::Nakshatra => computed.dasayus.map(|span| span.years),
                Ayus::Kalachakra => computed.chakrayus.map(|span| span.years),
                Ayus::Bhinnashtakavarga | Ayus::Samudaya => None,
            },
        }
    });
    let greatest = candidates
        .iter()
        .filter_map(|candidate| candidate.strength)
        .fold(None, |most: Option<f64>, strength| {
            Some(most.map_or(strength, |most| most.max(strength)))
        });
    let mut at_greatest = candidates
        .iter()
        .filter(|candidate| candidate.strength.is_some() && candidate.strength == greatest);
    let strongest = match (at_greatest.next(), at_greatest.next()) {
        (Some(only), None) => Some(only.by),
        _ => None,
    };
    SpanChoice {
        candidates,
        strongest,
        ayus: strongest.and_then(Ayus::named_by),
        all_weighed: candidates
            .iter()
            .all(|candidate| candidate.strength.is_some()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::StrengthMeasure;

    fn strengths(of: [Option<f64>; 10]) -> Strengths {
        Strengths {
            measure: StrengthMeasure::Caller,
            of,
            required: [None; 10],
        }
    }

    #[test]
    fn a_tie_or_no_strength_names_no_span_and_the_nodes_name_none() {
        let tied = span_choice(
            &strengths([
                Some(5.0),
                Some(5.0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]),
            None,
            Computed::default(),
        );
        assert_eq!(
            (tied.strongest, tied.ayus, tied.all_weighed),
            (None, None, false)
        );
        let none = span_choice(&strengths([None; 10]), None, Computed::default());
        assert_eq!(none.strongest, None);
        // The lagna's own strength stands where the measure reaches it.
        let mut of = [Some(1.0); 10];
        of[9] = Some(9.0);
        let lagna = span_choice(&strengths(of), Some(0.0), Computed::default());
        assert_eq!((lagna.ayus, lagna.all_weighed), (Some(Ayus::Amsa), true));
        assert_eq!(Ayus::named_by(Body::Graha(Graha::Rahu)), None);
        assert_eq!(
            Ayus::named_by(Body::Graha(Graha::Mars)),
            Some(Ayus::Bhinnashtakavarga)
        );
    }
}
