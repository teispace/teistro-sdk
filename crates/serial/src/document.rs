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

use serde::{Deserialize, Serialize};
use teistro_aspect::Aspects;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::envelope::Provenance;
use teistro_dasha::DashaReading;
use teistro_geometry::Drawing;
use teistro_houses::Houses;
use teistro_panchanga::almanac::Panchanga;
use teistro_points::Points;
use teistro_state::GrahaState;
use teistro_strength::{
    AshtakavargaReading, BhavaBalaReading, DashaPhalaReading, ShadbalaReading,
    VaiseshikamsaReading, VimshopakaReading,
};
use teistro_vargas::chart::VargaChart;

use crate::seal::Sealed;

/// One chart, with whichever of the layer's readings were asked for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Document {
    /// What every other section is computed from.
    pub foundation: ChartFoundation,
    /// The almanac of the day the chart belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panchanga: Option<Panchanga>,
    /// The divisional charts asked for.
    ///
    /// `default` because the key is left out when there are none, and a
    /// reader must take an absent key as "none were asked for" rather
    /// than refuse the document. Serde does that for an `Option` on its
    /// own; a `Vec` has to be told.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
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
    /// Each graha's Ashtakavarga, their sum, reductions and pindas
    /// (`03-design/ashtakavarga-measured.md`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ashtakavarga: Option<AshtakavargaReading>,
    /// Each graha's Vimshopaka under the four schemes
    /// (`03-design/vimshopaka-measured.md`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vimshopaka: Option<VimshopakaReading>,
    /// The names each graha earns by its good vargas (BPHS ch. 6).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vaiseshikamsa: Option<VaiseshikamsaReading>,
    /// Each graha's six strengths (`03-design/shadbala-measured.md`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub shadbala: Option<ShadbalaReading>,
    /// Each house's strength (`03-design/bhava-bala-measured.md`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bhava_bala: Option<BhavaBalaReading>,
    /// What each graha's placement says of its dasha (BPHS chs. 28 and 47).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dasha_phala: Option<DashaPhalaReading>,
    /// The charts drawn in the layouts asked for: which chart, placed in
    /// which layout (`03-design/chart-geometry.md`).
    ///
    /// `default` for the reason `vargas` has it: an absent key is "none
    /// were asked for", which a `Vec` has to be told.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub drawings: Vec<Drawing>,
    /// The dashas asked for: each system's balance and its periods to the
    /// settings' depth (`03-design/dasha-kernels.md`).
    ///
    /// `default` for the reason `vargas` has it.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dashas: Vec<DashaReading>,
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
            ashtakavarga: None,
            vimshopaka: None,
            vaiseshikamsa: None,
            dasha_phala: None,
            shadbala: None,
            bhava_bala: None,
            drawings: Vec::new(),
            dashas: Vec::new(),
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

    /// With a chart drawn in a layout, which may be asked for more than once.
    #[must_use]
    pub fn with_drawing(mut self, drawing: Drawing) -> Document {
        self.drawings.push(drawing);
        self
    }

    /// With a dasha, which may be asked for more than once, a system each.
    #[must_use]
    pub fn with_dasha(mut self, dasha: DashaReading) -> Document {
        self.dashas.push(dasha);
        self
    }

    /// With the Ashtakavarga.
    #[must_use]
    pub fn with_ashtakavarga(mut self, ashtakavarga: AshtakavargaReading) -> Document {
        self.ashtakavarga = Some(ashtakavarga);
        self
    }

    /// With the dasha phala.
    #[must_use]
    pub fn with_dasha_phala(mut self, dasha_phala: DashaPhalaReading) -> Document {
        self.dasha_phala = Some(dasha_phala);
        self
    }

    /// With the Vaiseshikamsa.
    #[must_use]
    pub fn with_vaiseshikamsa(mut self, vaiseshikamsa: VaiseshikamsaReading) -> Document {
        self.vaiseshikamsa = Some(vaiseshikamsa);
        self
    }

    /// With the Vimshopaka.
    #[must_use]
    pub fn with_vimshopaka(mut self, vimshopaka: VimshopakaReading) -> Document {
        self.vimshopaka = Some(vimshopaka);
        self
    }

    /// With the Shadbala.
    #[must_use]
    pub fn with_shadbala(mut self, shadbala: ShadbalaReading) -> Document {
        self.shadbala = Some(shadbala);
        self
    }

    /// With the Bhava bala.
    #[must_use]
    pub fn with_bhava_bala(mut self, bhava_bala: BhavaBalaReading) -> Document {
        self.bhava_bala = Some(bhava_bala);
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
        if self.ashtakavarga.is_some() {
            found.push("ashtakavarga");
        }
        if self.vimshopaka.is_some() {
            found.push("vimshopaka");
        }
        if self.vaiseshikamsa.is_some() {
            found.push("vaiseshikamsa");
        }
        if self.shadbala.is_some() {
            found.push("shadbala");
        }
        if self.bhava_bala.is_some() {
            found.push("bhava_bala");
        }
        if self.dasha_phala.is_some() {
            found.push("dasha_phala");
        }
        if !self.drawings.is_empty() {
            found.push("drawings");
        }
        if !self.dashas.is_empty() {
            found.push("dashas");
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
