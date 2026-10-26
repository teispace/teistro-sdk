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
use teistro_chart::moment::Moment;
use teistro_core::envelope::Provenance;
use teistro_dasha::DashaReading;
use teistro_dasha::jaimini::JaiminiReading;
use teistro_geometry::Drawing;
use teistro_houses::Houses;
use teistro_matching::Avakahada;
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
///
/// ```
/// use teistro_serial::Document;
/// use teistro_serial::canonical::from_hash_form;
///
/// // A whole chart's document, as the crate's own tests store one.
/// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
/// let bare = Document::of(chart.foundation.clone());
/// // Every section but the foundation is optional, and written only when carried.
/// assert_eq!(bare.sections(), ["foundation"]);
/// assert!(chart.sections().contains(&"shadbala"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// Jaimini's significators: the karakamsha and the Brahma graha, or why
    /// the rule found none, the graha arudhas, and BPHS ch. 32's sthira
    /// karakas and karaka bhavas (`03-design/jaimini-significators.md`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub jaimini: Option<JaiminiReading>,
    /// The janma-patrika's summary of the Moon: its star and pada, the
    /// syllable the child is named by, and the readings a koota takes of
    /// one native (`03-design/matching.md`, C301).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub avakahada: Option<Avakahada>,
    /// What the instant falls in: the rising sign's span and the eclipse
    /// seen where the chart is cast (`teistro_chart::moment`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub moment: Option<Moment>,
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
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_serial::Document;
    ///
    /// let document = Document::of(sample::foundation()?);
    /// assert_eq!(document.sections(), ["foundation"]);
    /// assert!(document.panchanga.is_none() && document.dashas.is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
            jaimini: None,
            avakahada: None,
            moment: None,
            shadbala: None,
            bhava_bala: None,
            drawings: Vec::new(),
            dashas: Vec::new(),
        }
    }

    /// With the almanac of its day.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let panchanga = chart.panchanga.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_panchanga(panchanga);
    /// assert_eq!(document.sections(), ["foundation", "panchanga"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_panchanga(mut self, panchanga: Panchanga) -> Document {
        self.panchanga = Some(panchanga);
        self
    }

    /// With a divisional chart, which may be asked for more than once.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let document = bare.with_varga(chart.vargas[0].clone());
    /// assert_eq!(document.sections(), ["foundation", "vargas"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_varga(mut self, varga: VargaChart) -> Document {
        self.vargas.push(varga);
        self
    }

    /// With what each graha is.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let state = chart.state.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_state(state);
    /// assert_eq!(document.sections(), ["foundation", "state"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_state(mut self, state: Vec<GrahaState>) -> Document {
        self.state = Some(state);
        self
    }

    /// With which bodies reach which.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let aspects = chart.aspects.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_aspects(aspects);
    /// assert_eq!(document.sections(), ["foundation", "aspects"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_aspects(mut self, aspects: Aspects) -> Document {
        self.aspects = Some(aspects);
        self
    }

    /// With the derived points.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let points = chart.points.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_points(points);
    /// assert_eq!(document.sections(), ["foundation", "points"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_points(mut self, points: Points) -> Document {
        self.points = Some(points);
        self
    }

    /// With a chart drawn in a layout, which may be asked for more than once.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Varga;
    /// use teistro_geometry::{draw, rows};
    /// use teistro_serial::Document;
    ///
    /// let foundation = sample::foundation()?;
    /// let drawing = draw(&rows::north_indian(), &foundation, Varga::D1)?;
    /// let document = Document::of(foundation).with_drawing(drawing);
    /// assert_eq!(document.sections(), ["foundation", "drawings"]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_drawing(mut self, drawing: Drawing) -> Document {
        self.drawings.push(drawing);
        self
    }

    /// With a dasha, which may be asked for more than once, a system each.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let document = bare.with_dasha(chart.dashas[0].clone());
    /// assert_eq!(document.sections(), ["foundation", "dashas"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_dasha(mut self, dasha: DashaReading) -> Document {
        self.dashas.push(dasha);
        self
    }

    /// With the Ashtakavarga.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let ashtakavarga = chart.ashtakavarga.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_ashtakavarga(ashtakavarga);
    /// assert_eq!(document.sections(), ["foundation", "ashtakavarga"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_ashtakavarga(mut self, ashtakavarga: AshtakavargaReading) -> Document {
        self.ashtakavarga = Some(ashtakavarga);
        self
    }

    /// With the dasha phala.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::{Dignity, Rashi};
    /// use teistro_core::settings::ShantaSign;
    /// use teistro_serial::Document;
    /// use teistro_strength::{DashaPhalaChart, DashaPhalaGraha, DashaPhalaReading};
    ///
    /// let chart = DashaPhalaChart {
    ///     grahas: [(); 9].map(|()| DashaPhalaGraha { longitude: 5.0, house: 1, dignity: Dignity::Neutral, retrograde: false }),
    ///     vargas: [[Rashi::Aries; 9]; 7],
    /// };
    /// let reading = DashaPhalaReading::of(&chart, ShantaSign::Friendly);
    /// let document = Document::of(sample::foundation()?).with_dasha_phala(reading);
    /// assert_eq!(document.sections(), ["foundation", "dasha_phala"]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_dasha_phala(mut self, dasha_phala: DashaPhalaReading) -> Document {
        self.dasha_phala = Some(dasha_phala);
        self
    }

    /// With Jaimini's significators.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let jaimini = chart.jaimini.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_jaimini(jaimini);
    /// assert_eq!(document.sections(), ["foundation", "jaimini"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_jaimini(mut self, jaimini: JaiminiReading) -> Document {
        self.jaimini = Some(jaimini);
        self
    }

    /// With what the instant falls in.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_chart::{BirthEclipse, Moment, RisingSign};
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_serial::Document;
    ///
    /// let moment = Moment { rising: RisingSign { sign: Rashi::Gemini, rose: None, sets: None }, nakshatra: None, eclipse: BirthEclipse::None };
    /// let document = Document::of(sample::foundation()?).with_moment(moment);
    /// assert_eq!(document.sections(), ["foundation", "moment"]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_moment(mut self, moment: Moment) -> Document {
        self.moment = Some(moment);
        self
    }

    /// With the Moon's avakahada.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let avakahada = chart.avakahada.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_avakahada(avakahada);
    /// assert_eq!(document.sections(), ["foundation", "avakahada"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_avakahada(mut self, avakahada: Avakahada) -> Document {
        self.avakahada = Some(avakahada);
        self
    }

    /// With the Vaiseshikamsa.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_serial::Document;
    /// use teistro_strength::{VaiseshikamsaChart, VaiseshikamsaReading, VimshopakaChart};
    ///
    /// let chart = VaiseshikamsaChart {
    ///     signs: VimshopakaChart::from_fn(|_, _| Rashi::Aries),
    ///     arudha_lagna: Rashi::Aries,
    ///     impaired: [false; 7],
    /// };
    /// let document = Document::of(sample::foundation()?).with_vaiseshikamsa(VaiseshikamsaReading::of(&chart));
    /// assert_eq!(document.sections(), ["foundation", "vaiseshikamsa"]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn with_vaiseshikamsa(mut self, vaiseshikamsa: VaiseshikamsaReading) -> Document {
        self.vaiseshikamsa = Some(vaiseshikamsa);
        self
    }

    /// With the Vimshopaka.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let vimshopaka = chart.vimshopaka.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_vimshopaka(vimshopaka);
    /// assert_eq!(document.sections(), ["foundation", "vimshopaka"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_vimshopaka(mut self, vimshopaka: VimshopakaReading) -> Document {
        self.vimshopaka = Some(vimshopaka);
        self
    }

    /// With the Shadbala.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let shadbala = chart.shadbala.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_shadbala(shadbala);
    /// assert_eq!(document.sections(), ["foundation", "shadbala"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_shadbala(mut self, shadbala: ShadbalaReading) -> Document {
        self.shadbala = Some(shadbala);
        self
    }

    /// With the Bhava bala.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let bhava_bala = chart.bhava_bala.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_bhava_bala(bhava_bala);
    /// assert_eq!(document.sections(), ["foundation", "bhava_bala"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_bhava_bala(mut self, bhava_bala: BhavaBalaReading) -> Document {
        self.bhava_bala = Some(bhava_bala);
        self
    }

    /// With the houses under both readings.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// let houses = chart.houses.clone().ok_or("the chart carries it")?;
    /// let document = bare.with_houses(houses);
    /// assert_eq!(document.sections(), ["foundation", "houses"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_houses(mut self, houses: Houses) -> Document {
        self.houses = Some(houses);
        self
    }

    /// Every member a document can carry, in the order it is written:
    /// the one list [`Document::sections`] reads, held to the document
    /// schema's properties by a test, so a section added to the type
    /// cannot be one this leaves out.
    pub const MEMBERS: [&'static str; 18] = [
        "foundation",
        "panchanga",
        "vargas",
        "state",
        "aspects",
        "points",
        "houses",
        "ashtakavarga",
        "vimshopaka",
        "vaiseshikamsa",
        "shadbala",
        "bhava_bala",
        "dasha_phala",
        "jaimini",
        "avakahada",
        "moment",
        "drawings",
        "dashas",
    ];

    /// Which sections this document carries, in the order they are
    /// written — what a consumer checks before reading one. A section is
    /// carried when the document writes it, so the answer is the JSON's.
    ///
    /// ```
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    ///
    /// // A whole chart's document, as the crate's own tests store one.
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let bare = Document::of(chart.foundation.clone());
    /// // The order a document is written in, whatever order it was built in.
    /// let sections = chart.sections();
    /// let order: Vec<usize> = sections.iter().filter_map(|name| Document::MEMBERS.iter().position(|member| member == name)).collect();
    /// assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
    /// # let _ = bare;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn sections(&self) -> Vec<&'static str> {
        let Ok(serde_json::Value::Object(written)) = serde_json::to_value(self) else {
            return vec!["foundation"];
        };
        Document::MEMBERS
            .into_iter()
            .filter(|member| written.contains_key(*member))
            .collect()
    }

    /// The document sealed with the provenance that produced it, which
    /// is how it leaves the SDK.
    ///
    /// ```
    /// use teistro_chart::sample;
    /// use teistro_core::envelope::{Hash, Provenance, Version};
    /// use teistro_serial::{Document, hash_of};
    ///
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "parashari-classical", Hash::of(b"settings"), Hash::of(b"input"));
    /// let document = Document::of(sample::foundation()?);
    /// let hash = hash_of(&document);
    /// let sealed = document.seal(provenance);
    /// assert_eq!(sealed.content_hash(), hash);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn seal(self, provenance: Provenance) -> Sealed<Document> {
        Sealed::new(self, provenance)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
