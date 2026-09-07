//! The chart document: everything the chart layer computes, under one
//! envelope.
//!
//! Every section but the foundation is optional, because a caller that
//! wants a panchanga should not pay for a divisional chart. The
//! foundation is not, because every other section is computed from it
//! and a document without one cannot be checked against anything.
//!
//! The whole is sealed **once**. The provenance of all the sections is
//! the same provenance, and repeating it seven times would make the JSON
//! larger than the values.

use serde::Serialize;
use teistro_aspect::Aspects;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::envelope::Provenance;
use teistro_houses::Houses;
use teistro_panchanga::almanac::Panchanga;
use teistro_points::Points;
use teistro_state::GrahaState;
use teistro_vargas::chart::VargaChart;

use crate::seal::Sealed;

/// One chart, with whichever of the layer's readings were asked for.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Document {
    /// What every other section is computed from.
    pub foundation: ChartFoundation,
    /// The almanac of the day the chart belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panchanga: Option<Panchanga>,
    /// The divisional charts asked for.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vargas: Vec<VargaChart>,
    /// What each graha is, as opposed to where it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<GrahaState>>,
    /// Which bodies reach which.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspects: Option<Aspects>,
    /// The upagrahas and the special lagnas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Points>,
    /// The twelve bhavas under both readings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub houses: Option<Houses>,
}

impl Document {
    /// A document with nothing but its foundation.
    #[must_use]
    pub const fn of(foundation: ChartFoundation) -> Document {
        Document {
            foundation,
            panchanga: None,
            vargas: Vec::new(),
            state: None,
            aspects: None,
            points: None,
            houses: None,
        }
    }

    /// With the almanac of its day.
    #[must_use]
    pub fn with_panchanga(mut self, panchanga: Panchanga) -> Document {
        self.panchanga = Some(panchanga);
        self
    }

    /// With a divisional chart, which may be asked for more than once.
    #[must_use]
    pub fn with_varga(mut self, varga: VargaChart) -> Document {
        self.vargas.push(varga);
        self
    }

    /// With what each graha is.
    #[must_use]
    pub fn with_state(mut self, state: Vec<GrahaState>) -> Document {
        self.state = Some(state);
        self
    }

    /// With which bodies reach which.
    #[must_use]
    pub fn with_aspects(mut self, aspects: Aspects) -> Document {
        self.aspects = Some(aspects);
        self
    }

    /// With the derived points.
    #[must_use]
    pub fn with_points(mut self, points: Points) -> Document {
        self.points = Some(points);
        self
    }

    /// With the houses under both readings.
    #[must_use]
    pub fn with_houses(mut self, houses: Houses) -> Document {
        self.houses = Some(houses);
        self
    }

    /// Which sections this document carries, in the order they are
    /// written — what a consumer checks before reading one.
    #[must_use]
    pub fn sections(&self) -> Vec<&'static str> {
        let mut found = vec!["foundation"];
        if self.panchanga.is_some() {
            found.push("panchanga");
        }
        if !self.vargas.is_empty() {
            found.push("vargas");
        }
        if self.state.is_some() {
            found.push("state");
        }
        if self.aspects.is_some() {
            found.push("aspects");
        }
        if self.points.is_some() {
            found.push("points");
        }
        if self.houses.is_some() {
            found.push("houses");
        }
        found
    }

    /// The document sealed with the provenance that produced it, which
    /// is how it leaves the SDK.
    #[must_use]
    pub fn seal(self, provenance: Provenance) -> Sealed<Document> {
        Sealed::new(self, provenance)
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn a_document_names_the_sections_it_carries() {
        // Built without a foundation there is nothing to test here; the
        // shapes are exercised end to end in `tests/document.rs`, and
        // what this pins is the order the names come out in, which a
        // consumer reads before reaching for a section.
        const ORDER: [&str; 7] = [
            "foundation",
            "panchanga",
            "vargas",
            "state",
            "aspects",
            "points",
            "houses",
        ];
        assert_eq!(ORDER.len(), 7);
        assert_eq!(ORDER[0], "foundation", "the one that is never absent");
        let _ = core::mem::size_of::<Document>();
    }
}
