//! The time a sankalpa recites, as a binding asks it
//! (`03-design/sankalpa.md`): an instant, a place, the clock its civil
//! days run on, and how the ayana is reckoned.

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_panchanga::sankalpa::AyanaReckoning;
use teistro_time::ZoneSpec;

use crate::asked::{instant, place_of};
use crate::{AlmanacArea, DayClock, HIMAVATKHANDE, SankalpaFor, SankalpaWords};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const SANKALPA: &str = "sankalpa";

/// A sankalpa's instant and place, as a binding asks it.
#[derive(Clone, Debug, PartialEq)]
pub struct SankalpaRequest {
    /// The instant of the rite.
    pub at: JulianDay<Utc>,
    /// The place.
    pub place: Place,
    /// The clock its civil days run on: an offset stated, or a zone read
    /// at the instant.
    pub clock: DayClock,
    /// How the ayana is reckoned.
    pub ayana: AyanaReckoning,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SankalpaRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SankalpaAsked {
    /// The instant of the rite, a UTC Julian day.
    at: f64,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds,
    /// east positive; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone each instant is read on, at its offset then; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// How the ayana is reckoned: `SIDEREAL`, at Makara and Karka
    /// sankranti, when left out.
    #[serde(default)]
    ayana: AyanaReckoning,
}

impl SankalpaRequest {
    /// The record a binding sends, as JSON: `at`, a UTC Julian day;
    /// `latitudeDeg`, `longitudeDeg`, `altitudeM` (0 when left out) and
    /// `utcOffsetSeconds` or a `zone`; and `ayana`, `SIDEREAL` or `TROPICAL`,
    /// `SIDEREAL` when left out.
    ///
    /// ```
    /// use teistro::{AyanaReckoning, SankalpaRequest};
    ///
    /// let asked = SankalpaRequest::from_json(
    ///     r#"{"at": 2461055.59375, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///         "altitudeM": 1400, "utcOffsetSeconds": 20700, "ayana": "TROPICAL"}"#,
    /// )?;
    /// assert_eq!(asked.ayana, AyanaReckoning::Tropical);
    /// let nowhere = SankalpaRequest::from_json(
    ///     r#"{"at": 2461055.59375, "latitudeDeg": 97, "longitudeDeg": 85.324,
    ///         "utcOffsetSeconds": 20700}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(nowhere.field(), Some("sankalpa.latitudeDeg"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an instant, place or offset out of range, or both or neither
    /// of `utcOffsetSeconds` and `zone`, each named under `sankalpa`.
    pub fn from_json(text: &str) -> Result<SankalpaRequest, Error> {
        let asked: SankalpaAsked = teistro_core::strict::read(text, SANKALPA)?;
        asked.request().map_err(|why| why.under(SANKALPA))
    }
}

impl SankalpaAsked {
    fn request(self) -> Result<SankalpaRequest, Error> {
        Ok(SankalpaRequest {
            at: instant(self.at, "at")?,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            ayana: self.ayana,
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::sankalpa`] for a request a binding sent, every
    /// catalogue member written as its full key: what the boundary
    /// carries and seals.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::sankalpa`]; a zone the database does not have.
    pub fn sankalpa_request(
        self,
        asked: &SankalpaRequest,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self.clocked_sealed(
            &asked.clock,
            || Ok(asked.at),
            |almanac, offset| almanac.sankalpa(asked.at, &asked.place, offset, asked.ayana),
        )?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}

/// The record's name where a binding sends the words, which a refusal is
/// named under.
const SANKALPA_WORDS: &str = "sankalpaWords";

/// A sankalpa in words, as a binding asks it (`03-design/sankalpa.md` §5,
/// SR-29): the instant and place [`SankalpaRequest`] reads, the slots the
/// template leaves to the caller, the tradition and the locale.
///
/// ```
/// use teistro::{HIMAVATKHANDE, SankalpaWordsRequest};
///
/// let asked = SankalpaWordsRequest::from_json(
///     r#"{"at": 2461055.59375, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "slots": {"performer": "हरिनाथशर्मा", "gotra": "कौण्डिन्य", "purpose": "गणेशपूजनम्"}}"#,
/// )?;
/// assert_eq!((asked.tradition.as_str(), asked.locale), (HIMAVATKHANDE, None));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SankalpaWordsRequest {
    /// The time and place.
    pub time: SankalpaRequest,
    /// The performer, gotra, purpose, place and pitris.
    pub slots: SankalpaFor,
    /// The tradition: its template's message key.
    pub tradition: String,
    /// The locale; the context's when `None`.
    pub locale: Option<String>,
}

/// The words' request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SankalpaWordsRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SankalpaWordsAsked {
    /// The instant of the rite, a UTC Julian day.
    at: f64,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds,
    /// east positive; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone each instant is read on, at its offset then; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// How the ayana is reckoned: `SIDEREAL` when left out.
    #[serde(default)]
    ayana: AyanaReckoning,
    /// The performer, gotra, purpose, and optionally the river, the holy
    /// ground and the three generations of pitris.
    slots: SankalpaFor,
    /// The tradition, its template's message key:
    /// `sdk.sankalpa.himavatkhande` when left out, or a key a loaded pack
    /// carries.
    #[serde(default)]
    tradition: Option<String>,
    /// The locale the words are in; the context's when left out.
    #[serde(default)]
    locale: Option<String>,
}

impl SankalpaWordsRequest {
    /// The record a binding sends, as JSON: [`SankalpaRequest`]'s keys,
    /// `slots` (a [`SankalpaFor`]), and optionally `tradition` and
    /// `locale`.
    ///
    /// ```
    /// use teistro::SankalpaWordsRequest;
    ///
    /// let blank = SankalpaWordsRequest::from_json(
    ///     r#"{"at": 2461055.59375, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "slots": {"performer": "हरिनाथशर्मा", "gotra": " ", "purpose": "गणेशपूजनम्"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(blank.field(), Some("sankalpaWords.slots.gotra"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` as [`SankalpaRequest::from_json`], and on a slot left
    /// blank, each named under `sankalpaWords`.
    pub fn from_json(text: &str) -> Result<SankalpaWordsRequest, Error> {
        let asked: SankalpaWordsAsked = teistro_core::strict::read(text, SANKALPA_WORDS)?;
        asked
            .slots
            .check_under("slots")
            .map_err(|why| why.under(SANKALPA_WORDS))?;
        let time = SankalpaAsked {
            at: asked.at,
            latitude_deg: asked.latitude_deg,
            longitude_deg: asked.longitude_deg,
            altitude_m: asked.altitude_m,
            utc_offset_seconds: asked.utc_offset_seconds,
            zone: asked.zone,
            ayana: asked.ayana,
        }
        .request()
        .map_err(|why| why.under(SANKALPA_WORDS))?;
        Ok(SankalpaWordsRequest {
            time,
            slots: asked.slots,
            tradition: asked.tradition.unwrap_or_else(|| HIMAVATKHANDE.to_owned()),
            locale: asked.locale,
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::sankalpa_words`] for a request a binding sent:
    /// the time reckoned at the request's instant and place on its clock,
    /// then said by the tradition's template, sealed with the time's
    /// provenance.
    ///
    /// ```no_run
    /// use teistro::{Context, SankalpaWordsRequest};
    ///
    /// let sdk = Context::builder().profile("nepali-drik").build()?;
    /// let asked = SankalpaWordsRequest::from_json(
    ///     r#"{"at": 2461055.59375, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "slots": {"performer": "हरिनाथशर्मा", "gotra": "कौण्डिन्य", "purpose": "गणेशपूजनम्"}, "locale": "sa-Deva"}"#,
    /// )?;
    /// let said = sdk.almanac().sankalpa_words_request(&asked)?;
    /// assert!(said.value.text.starts_with("हरिः ॐ तत्सत्"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::sankalpa`]; as [`AlmanacArea::sankalpa_words`],
    /// its fields named under `sankalpaWords`.
    pub fn sankalpa_words_request(
        self,
        asked: &SankalpaWordsRequest,
    ) -> Result<Envelope<SankalpaWords>, Error> {
        let time = &asked.time;
        self.clocked_sealed(
            &time.clock,
            || Ok(time.at),
            |almanac, offset| {
                let Envelope { value, provenance } =
                    almanac.sankalpa(time.at, &time.place, offset, time.ayana)?;
                let words = almanac
                    .sankalpa_words(
                        &value,
                        &asked.slots,
                        &asked.tradition,
                        asked.locale.as_deref(),
                    )
                    .map_err(|why| why.under(SANKALPA_WORDS))?;
                Ok(Envelope::sealing(words, provenance))
            },
        )
    }
}
