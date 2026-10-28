//! The gochar sidecar: a chart's natal facts, kept beside its date-free
//! dossier so its transits can be read on any later day
//! (`03-design/gochar-sidecar.md`).
//!
//! A stored chart is text that never changes, and a transit is a question
//! about *now*. The sidecar is what every transit reading asks of the natal
//! chart — its foundation, and its panchanga when it has one — cut from a
//! sealed document and tied by hash to that document and to the settings
//! it was computed under. It holds no instant but the birth's, so the same
//! chart gives the same bytes on any day.
//!
//! ```
//! use teistro_core::envelope::{Hash, Provenance, Version};
//! use teistro_serial::Document;
//! use teistro_serial::canonical::from_hash_form;
//! use teistro_serial::sidecar::GocharSidecar;
//!
//! let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
//! let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
//! let sealed = chart.seal(provenance);
//!
//! // Cut it when the chart is computed, and store its text.
//! let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
//! let stored = sidecar.to_text();
//!
//! // Read it back on any later day, check it is this chart's, and open it.
//! let read = GocharSidecar::read(&stored)?;
//! read.check_chart(sealed.content_hash())?;
//! assert_eq!(read.natal().sections(), ["foundation", "panchanga"]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::envelope::{Hash, Provenance};
use teistro_core::error::{Error, Status};
use teistro_panchanga::almanac::Panchanga;

use crate::Document;
use crate::canonical::{hash_of, to_hash_form};

/// The record a sidecar's fields are named under when one is refused.
const SIDECAR: &str = "sidecar";

/// A chart's natal facts for its transits: its foundation and, when the
/// chart carries one, its panchanga, with the hashes that tie them to the
/// chart they were cut from.
///
/// Built only by [`GocharSidecar::cut`] and read only by
/// [`GocharSidecar::read`], so one whose `natal_hash` is not its own
/// sections' is not representable.
///
/// ```
/// use teistro_serial::sidecar::GocharSidecar;
///
/// // The shape a stored sidecar has; another format is refused by name.
/// assert_eq!(GocharSidecar::FORMAT, 1);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GocharSidecar {
    /// The sidecar's own shape, `1`.
    format: u32,
    /// The content hash of the sealed chart document it was cut from.
    chart: Hash,
    /// The settings hash that document was computed under.
    settings: Hash,
    /// The content hash of the natal document it opens to: the foundation,
    /// and the panchanga when carried.
    natal_hash: Hash,
    /// The chart's foundation, in the document's own shape.
    foundation: ChartFoundation,
    /// The chart's panchanga, which the Sarvatobhadra reads the birth's
    /// tithi and weekday from; absent when the chart was founded without
    /// one.
    #[serde(skip_serializing_if = "Option::is_none")]
    panchanga: Option<Panchanga>,
}

/// A sidecar as it is read, before its hash is checked.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct Unchecked {
    format: u32,
    chart: Hash,
    settings: Hash,
    natal_hash: Hash,
    foundation: ChartFoundation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    panchanga: Option<Panchanga>,
}

impl GocharSidecar {
    /// The sidecar shape this build writes and reads.
    pub const FORMAT: u32 = 1;

    /// The document sections a sidecar carries, when the chart has them:
    /// what every transit operation reads of a natal chart.
    pub const SECTIONS: [&'static str; 2] = ["foundation", "panchanga"];

    /// Cuts the sidecar of a chart document, from the document and the
    /// provenance it was published with: an [`Envelope`]'s two fields, or a
    /// [`Sealed`](crate::Sealed)'s `value()` and `provenance()`.
    ///
    /// [`Envelope`]: teistro_core::envelope::Envelope
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Provenance, Version};
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    /// use teistro_serial::sidecar::GocharSidecar;
    ///
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert_eq!(sidecar.chart(), sealed.content_hash());
    /// assert_eq!(sidecar.settings(), Hash::of(b"settings"));
    ///
    /// // A provenance that is not this document's is refused: the sidecar
    /// // would claim a chart it was not cut from.
    /// let other = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let refused = GocharSidecar::cut(sealed.value(), &other).unwrap_err();
    /// assert_eq!(refused.field(), Some("provenance.content_hash"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `provenance.content_hash` when the provenance's
    /// content hash is not the document's.
    pub fn cut(document: &Document, provenance: &Provenance) -> Result<GocharSidecar, Error> {
        let chart = hash_of(document);
        if provenance.content_hash != chart {
            return Err(Error::invalid_arg(format!(
                "the provenance carries the content hash {}, and the document's is {chart}",
                provenance.content_hash
            ))
            .with_field("provenance.content_hash")
            .with_hint("cut the sidecar with the provenance the document was published with"));
        }
        let foundation = document.foundation.clone();
        let panchanga = document.panchanga.clone();
        let natal_hash = hash_of(&natal_of(&foundation, panchanga.as_ref()));
        Ok(GocharSidecar {
            format: GocharSidecar::FORMAT,
            chart,
            settings: provenance.settings_hash,
            natal_hash,
            foundation,
            panchanga,
        })
    }

    /// Reads a stored sidecar back, in any JSON spelling of it: a database
    /// that keeps JSON in a binary column returns its keys reordered, and
    /// the check is on the canonical hash, not on the stored bytes.
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Provenance, Version};
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    /// use teistro_serial::sidecar::GocharSidecar;
    ///
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let text = GocharSidecar::cut(sealed.value(), sealed.provenance())?.to_text();
    /// // The same bytes again, a fixed point.
    /// assert_eq!(GocharSidecar::read(&text)?.to_text(), text);
    ///
    /// // A sidecar of another shape is refused by its format.
    /// let later = text.replacen(r#""format":1"#, r#""format":2"#, 1);
    /// let refused = GocharSidecar::read(&later).unwrap_err();
    /// assert_eq!(refused.field(), Some("sidecar.format"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `SCHEMA_VERSION` naming `sidecar.format` for a format other than
    /// [`GocharSidecar::FORMAT`]; `INVALID_ARG` for text that is not the
    /// record or names a key it does not have (by its path), and naming
    /// `sidecar.natal_hash` when the sections read are not the ones it was
    /// cut with.
    pub fn read(text: &str) -> Result<GocharSidecar, Error> {
        let given = teistro_core::strict::parse(text, SIDECAR)?;
        // A missing format is the record's own refusal, below; a format
        // given and not this build's is a sidecar of another shape.
        let format = given.get("format");
        if let Some(format) =
            format.filter(|one| one.as_u64() != Some(u64::from(GocharSidecar::FORMAT)))
        {
            return Err(Error::new(
                Status::SchemaVersion,
                format!(
                    "a gochar sidecar of format {}, and this build reads format {}",
                    format,
                    GocharSidecar::FORMAT
                ),
            )
            .with_field(format!("{SIDECAR}.format"))
            .with_hint("cut the sidecar again from the chart under this build"));
        }
        let read: Unchecked = teistro_core::strict::read_value(&given, SIDECAR)?;
        let natal_hash = hash_of(&natal_of(&read.foundation, read.panchanga.as_ref()));
        if natal_hash != read.natal_hash {
            return Err(Error::invalid_arg(format!(
                "the sidecar's sections hash to {natal_hash}, and it was cut with {}",
                read.natal_hash
            ))
            .with_field(format!("{SIDECAR}.natal_hash"))
            .with_hint(
                "the stored sidecar was edited or written by a build whose sections this one \
                 reads otherwise; cut it again from the chart",
            ));
        }
        Ok(GocharSidecar {
            format: read.format,
            chart: read.chart,
            settings: read.settings,
            natal_hash,
            foundation: read.foundation,
            panchanga: read.panchanga,
        })
    }

    /// The sidecar in the canonical form: keys in code-point order, no
    /// whitespace, no exponent. The same chart gives the same bytes.
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Provenance, Version};
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    /// use teistro_serial::sidecar::GocharSidecar;
    ///
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let once = GocharSidecar::cut(sealed.value(), sealed.provenance())?.to_text();
    /// let twice = GocharSidecar::cut(sealed.value(), sealed.provenance())?.to_text();
    /// assert_eq!(once, twice);
    /// assert!(once.starts_with(r#"{"chart":""#));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn to_text(&self) -> String {
        to_hash_form(self)
    }

    /// Refuses the sidecar unless it was cut from the chart whose content
    /// hash is `chart`: the hash an app stores with the chart's provenance.
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Provenance, Version};
    /// use teistro_serial::Document;
    /// use teistro_serial::canonical::from_hash_form;
    /// use teistro_serial::sidecar::GocharSidecar;
    ///
    /// let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert!(sidecar.check_chart(sealed.content_hash()).is_ok());
    /// // A chart recomputed since has another hash, and its old sidecar is stale.
    /// let stale = sidecar.check_chart(Hash::of(b"the chart recomputed")).unwrap_err();
    /// assert_eq!(stale.field(), Some("sidecar.chart"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `sidecar.chart` when it was cut from another
    /// chart.
    pub fn check_chart(&self, chart: Hash) -> Result<&GocharSidecar, Error> {
        if self.chart == chart {
            Ok(self)
        } else {
            Err(Error::invalid_arg(format!(
                "the sidecar was cut from the chart {}, not {chart}",
                self.chart
            ))
            .with_field(format!("{SIDECAR}.chart"))
            .with_hint("cut the sidecar again from the chart as it is now"))
        }
    }

    /// The content hash of the chart document it was cut from.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::{Document, hash_of};
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// # let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert_eq!(sidecar.chart(), hash_of(sealed.value()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn chart(&self) -> Hash {
        self.chart
    }

    /// The settings hash the chart was computed under, which a context
    /// opening the sidecar must have.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::Document;
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert_eq!(sidecar.settings(), sealed.provenance().settings_hash);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn settings(&self) -> Hash {
        self.settings
    }

    /// The content hash of the natal document it opens to.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::{Document, hash_of};
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// # let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert_eq!(sidecar.natal_hash(), hash_of(&sidecar.natal()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn natal_hash(&self) -> Hash {
        self.natal_hash
    }

    /// The chart's foundation.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::Document;
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// # let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert_eq!(sidecar.foundation(), &sealed.value().foundation);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn foundation(&self) -> &ChartFoundation {
        &self.foundation
    }

    /// The chart's panchanga, when it was founded with one.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::Document;
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// # let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// assert!(sidecar.panchanga().is_some());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn panchanga(&self) -> Option<&Panchanga> {
        self.panchanga.as_ref()
    }

    /// The natal document a transit operation reads: the foundation, with
    /// the panchanga when carried. Not checked against a context's
    /// settings; the façade's `chart().natal` is, and is the one an app
    /// calls.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # use teistro_serial::Document;
    /// # use teistro_serial::canonical::from_hash_form;
    /// # use teistro_serial::sidecar::GocharSidecar;
    /// # let chart: Document = from_hash_form(include_str!("../tests/dossier-chart.json"))?;
    /// # let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "conformance-baseline", Hash::of(b"settings"), Hash::of(b"input"));
    /// let sealed = chart.seal(provenance);
    /// let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance())?;
    /// let natal = sidecar.natal();
    /// assert_eq!(natal.foundation, sealed.value().foundation);
    /// assert_eq!(natal.sections(), GocharSidecar::SECTIONS);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn natal(&self) -> Document {
        natal_of(&self.foundation, self.panchanga.as_ref())
    }
}

/// The natal document of a foundation and its panchanga.
fn natal_of(foundation: &ChartFoundation, panchanga: Option<&Panchanga>) -> Document {
    let natal = Document::of(foundation.clone());
    match panchanga {
        Some(day) => natal.with_panchanga(day.clone()),
        None => natal,
    }
}
