//! The sankalpa said in words (`03-design/sankalpa.md` §5, SR-29): a
//! tradition's template, a message in the locale packs, filled with the
//! time [`Sankalpa`] reckons and the performer, gotra, place, purpose and
//! pitris the caller gives.
//!
//! A tradition is a message key. The SDK ships one, `HIMAVATKHANDE`, the
//! Nepal form, from the Mahendra Sanskrit University's *Karmakanda
//! Bhaskara*; a consumer registers another by loading a pack that carries
//! its own key with the same parameters, and asks for it by that key.

use serde::{Deserialize, Serialize};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::error::Error;
use teistro_intl::{Params, Value};
use teistro_panchanga::sankalpa::Sankalpa;

use super::AlmanacArea;

/// The field a sankalpa's words are refused under.
const SANKALPA: &str = "sankalpa";

/// The Nepal form: the place spoken as *Himavatparvataikadeśe
/// Nepāladeśe*, on the Himalaya, in Nepal (*Karmakanda Bhaskara*, VS 2054,
/// pp. 164, 477 and 500; `03-design/sankalpa.md` §5).
pub const HIMAVATKHANDE: &str = "sdk.sankalpa.himavatkhande";

/// The traditions the SDK ships, by their message key.
pub const SANKALPA_TRADITIONS: [&str; 1] = [HIMAVATKHANDE];

/// The three generations of pitris a rite for the forebears names, the
/// father first, each as the stem the template compounds (*Rāma*, not
/// *Rāmaśarmā*).
///
/// ```
/// use teistro::SankalpaFor;
///
/// let asked = SankalpaFor::from_json(
///     r#"{"performer": "हरिनाथशर्मा", "gotra": "कौण्डिन्य", "purpose": "तर्पणम्",
///         "pitris": {"father": "राम", "grandfather": "हरि", "greatGrandfather": "शिव"}}"#,
/// )?;
/// assert_eq!(asked.pitris.map(|p| p.great_grandfather), Some("शिव".to_owned()));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pitris {
    /// The father (*pitṛ*).
    pub father: String,
    /// The father's father (*pitāmaha*).
    pub grandfather: String,
    /// The father's father's father (*prapitāmaha*).
    pub great_grandfather: String,
}

/// Who resolves, where, and to what: the slots a tradition's template
/// leaves to the caller, written as they are to be spoken.
///
/// ```
/// use teistro::SankalpaFor;
///
/// let asked = SankalpaFor::from_json(
///     r#"{"performer": "हरिनाथशर्मा", "gotra": "कौण्डिन्य", "purpose": "गणेशपूजनम्",
///         "river": "वाग्मती", "kshetra": "पाशुपत"}"#,
/// )?;
/// assert_eq!(asked.pitris, None);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SankalpaFor {
    /// The performer's name with its title, in the nominative, as spoken
    /// before *aham* (`हरिनाथशर्मा`).
    pub performer: String,
    /// The gotra's stem (`कौण्डिन्य`), which the template compounds.
    pub gotra: String,
    /// The rite resolved on, in the accusative (`गणेशपूजनम्`), as spoken
    /// before *kariṣye*.
    pub purpose: String,
    /// The river the place is near, as a stem (`वाग्मती`); left out, the
    /// template says none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub river: Option<String>,
    /// The holy ground, as a stem (`पाशुपत`); left out, the template says
    /// none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kshetra: Option<String>,
    /// The three generations, for a rite for the forebears (a shraddha, a
    /// tarpana); left out, the template names none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitris: Option<Pitris>,
}

impl SankalpaFor {
    /// The record as JSON: `performer`, `gotra`, `purpose`, and
    /// optionally `river`, `kshetra` and `pitris` (`father`,
    /// `grandfather`, `greatGrandfather`).
    ///
    /// ```
    /// use teistro::SankalpaFor;
    ///
    /// let typo = SankalpaFor::from_json(r#"{"performer": "x", "gotra": "y", "purpos": "z"}"#).unwrap_err();
    /// assert!(typo.field().is_some_and(|field| field.starts_with("sankalpa")));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `sankalpa`.
    pub fn from_json(text: &str) -> Result<SankalpaFor, Error> {
        teistro_core::strict::read(text, SANKALPA)
    }

    /// Refuses a slot left empty, which the template would speak as
    /// nothing, naming it under `root`.
    pub(crate) fn check_under(&self, root: &str) -> Result<(), Error> {
        let mut named = vec![
            ("performer", self.performer.as_str()),
            ("gotra", self.gotra.as_str()),
            ("purpose", self.purpose.as_str()),
        ];
        if let Some(river) = &self.river {
            named.push(("river", river));
        }
        if let Some(kshetra) = &self.kshetra {
            named.push(("kshetra", kshetra));
        }
        if let Some(pitris) = &self.pitris {
            named.push(("pitris.father", &pitris.father));
            named.push(("pitris.grandfather", &pitris.grandfather));
            named.push(("pitris.greatGrandfather", &pitris.great_grandfather));
        }
        match named.into_iter().find(|(_, text)| text.trim().is_empty()) {
            Some((field, _)) => Err(Error::invalid_arg(format!(
                "`{field}` is empty, and a sankalpa does not speak a blank"
            ))
            .with_field(format!("{root}.{field}"))),
            None => Ok(()),
        }
    }
}

/// A sankalpa said by a tradition's template.
///
/// ```no_run
/// use teistro::{AyanaReckoning, Context, HIMAVATKHANDE, SankalpaFor, UtcOffset};
/// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
///
/// let sdk = Context::builder().profile("nepali-drik").build()?;
/// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
/// let at = JulianDay::try_new(2_461_055.593_75)?;
/// let time = sdk.almanac().sankalpa(at, &kathmandu, UtcOffset::literal(5, 45, 0), AyanaReckoning::Sidereal)?;
/// let asked = SankalpaFor { performer: "x".into(), gotra: "y".into(), purpose: "z".into(), ..SankalpaFor::default() };
/// let said = sdk.almanac().sankalpa_words(&time.value, &asked, HIMAVATKHANDE, Some("en-Latn"))?;
/// assert!(said.text.starts_with("hariḥ oṃ tatsat"));
/// assert_eq!(said.tradition, HIMAVATKHANDE);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SankalpaWords {
    /// The tradition: the template's message key.
    pub tradition: String,
    /// The locale it was said in: the script it is written in.
    pub locale: String,
    /// The template's slots, as `ts_intl_render` takes them, to say it
    /// again in another locale.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
    /// The words.
    pub text: String,
}

/// The slots every tradition's template is given: the time `sankalpa`
/// reckons and the caller's `asked`, each under the name the shipped
/// template reads.
///
/// | slot | value |
/// |---|---|
/// | `samvatsara`, `ayana`, `ritu`, `masa`, `paksha`, `vara`, `tithi`, `nakshatra`, `yoga`, `karana` | the member, as an entity |
/// | `sun`, `moon`, `jupiter` | their signs, as entities |
/// | `vikrama` | the Vikrama year, an integer |
/// | `varaKey` | the weekday's catalogue key (`RAVIVARA`), for a template that says its stem |
/// | `adhika` | `yes` in an adhika month, else `no` |
/// | `performer`, `gotra`, `purpose` | the caller's text |
/// | `river`, `kshetra`, with `hasRiver`, `hasKshetra` | the caller's text, or empty and `no` |
/// | `father`, `grandfather`, `greatGrandfather`, with `hasPitris` | likewise |
///
/// ```no_run
/// use teistro::{AyanaReckoning, Context, SankalpaFor, UtcOffset, Value, sankalpa_params};
/// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
///
/// let sdk = Context::builder().profile("nepali-drik").build()?;
/// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
/// let at = JulianDay::try_new(2_461_055.593_75)?;
/// let time = sdk.almanac().sankalpa(at, &kathmandu, UtcOffset::literal(5, 45, 0), AyanaReckoning::Sidereal)?;
/// let asked = SankalpaFor { performer: "x".into(), gotra: "y".into(), purpose: "z".into(), ..SankalpaFor::default() };
/// let params = sankalpa_params(&time.value, &asked);
/// assert_eq!(params.get("hasPitris"), Some(&Value::Str("no".into())));
/// # Ok::<(), teistro::Error>(())
/// ```
#[must_use]
pub fn sankalpa_params(sankalpa: &Sankalpa, asked: &SankalpaFor) -> Params {
    let yes = |holds: bool| Value::Str(String::from(if holds { "yes" } else { "no" }));
    let text = |slot: Option<&String>| Value::Str(slot.cloned().unwrap_or_default());
    let pitri = |pick: fn(&Pitris) -> &String| text(asked.pitris.as_ref().map(pick));
    let mut params = Params::new();
    let mut put = |name: &str, value: Value| {
        params.insert(name.to_owned(), value);
    };
    put("samvatsara", Value::catalogued(sankalpa.year.samvatsara));
    put("vikrama", Value::Int(i64::from(sankalpa.year.vikrama)));
    put("ayana", Value::catalogued(sankalpa.ayana));
    put("ritu", Value::catalogued(sankalpa.ritu));
    put("masa", Value::catalogued(sankalpa.month.month));
    put("adhika", yes(sankalpa.month.kind == MonthKind::Adhika));
    put("paksha", Value::catalogued(sankalpa.paksha));
    put("vara", Value::catalogued(sankalpa.vara));
    put("varaKey", Value::Str(sankalpa.vara.key().to_owned()));
    put("tithi", Value::catalogued(sankalpa.tithi));
    put("nakshatra", Value::catalogued(sankalpa.nakshatra));
    put("yoga", Value::catalogued(sankalpa.yoga));
    put("karana", Value::catalogued(sankalpa.karana));
    put("sun", Value::catalogued(sankalpa.sun));
    put("moon", Value::catalogued(sankalpa.moon));
    put("jupiter", Value::catalogued(sankalpa.jupiter));
    put("performer", Value::Str(asked.performer.clone()));
    put("gotra", Value::Str(asked.gotra.clone()));
    put("purpose", Value::Str(asked.purpose.clone()));
    put("hasRiver", yes(asked.river.is_some()));
    put("river", text(asked.river.as_ref()));
    put("hasKshetra", yes(asked.kshetra.is_some()));
    put("kshetra", text(asked.kshetra.as_ref()));
    put("hasPitris", yes(asked.pitris.is_some()));
    put("father", pitri(|p| &p.father));
    put("grandfather", pitri(|p| &p.grandfather));
    put("greatGrandfather", pitri(|p| &p.great_grandfather));
    params
}

impl AlmanacArea<'_> {
    /// **The sankalpa in words**: a tradition's template, `tradition` its
    /// message key ([`HIMAVATKHANDE`] or one a loaded pack carries),
    /// filled with `sankalpa`'s time and the caller's slots, in `locale`
    /// (the context's when left out). The Sanskrit is in each Devanagari
    /// locale that carries the template, and in IAST in `en-Latn` and
    /// `sa-Latn`.
    ///
    /// ```no_run
    /// use teistro::{AyanaReckoning, Context, HIMAVATKHANDE, SankalpaFor, UtcOffset};
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    ///
    /// let sdk = Context::builder().profile("nepali-drik").build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let at = JulianDay::try_new(2_461_055.593_75)?;
    /// let time = sdk.almanac().sankalpa(at, &kathmandu, UtcOffset::literal(5, 45, 0), AyanaReckoning::Sidereal)?;
    /// let asked = SankalpaFor {
    ///     performer: "हरिनाथशर्मा".into(),
    ///     gotra: "कौण्डिन्य".into(),
    ///     purpose: "गणेशपूजनम्".into(),
    ///     ..SankalpaFor::default()
    /// };
    /// let said = sdk.almanac().sankalpa_words(&time.value, &asked, HIMAVATKHANDE, Some("sa-Deva"))?;
    /// println!("{}", said.text);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a slot left empty, named under `sankalpa`;
    /// `UNSUPPORTED` on `locale` for a locale not loaded, and on
    /// `tradition` for a key no loaded locale carries.
    pub fn sankalpa_words(
        self,
        sankalpa: &Sankalpa,
        asked: &SankalpaFor,
        tradition: &str,
        locale: Option<&str>,
    ) -> Result<SankalpaWords, Error> {
        asked.check_under(SANKALPA)?;
        let intl = self.context.locale_engine();
        let tag = locale.map_or_else(|| intl.locale().to_owned(), str::to_owned);
        if !intl.locales().any(|loaded| loaded.tag == tag) {
            let known: Vec<&str> = intl.locales().map(|l| l.tag.as_str()).collect();
            return Err(
                Error::unsupported(format!("the locale `{tag}` is not loaded"))
                    .with_field("locale")
                    .with_hint(format!("the locales are {}", known.join(", "))),
            );
        }
        if intl.resolve_from(&tag, tradition).is_none() {
            return Err(Error::unsupported(format!(
                "no loaded pack carries the sankalpa tradition `{tradition}`"
            ))
            .with_field("tradition")
            .with_hint(format!(
                "the SDK ships {}; load a pack that carries another",
                SANKALPA_TRADITIONS.join(", ")
            )));
        }
        let params = sankalpa_params(sankalpa, asked);
        let rendered = intl.render_in(&tag, tradition, &params);
        Ok(SankalpaWords {
            tradition: tradition.to_owned(),
            locale: rendered.resolved_from.unwrap_or(tag),
            params,
            text: rendered.text,
        })
    }
}
