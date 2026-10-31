//! A day in sentences (`03-design/day-in-sentences.md`, SR-13): the
//! almanac's answer for one date, said by the `sdk.day` templates in a
//! locale that carries them.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_intl::{Intl, Params};

use super::AlmanacArea;
use super::day_plan::{DAY_KEYS, DayPlan, day_sentences};
use crate::asked::{DayAsked, offset_of, place_of};

/// The field a sentence request's refusals are named under.
const SENTENCES: &str = "sentences";

/// The shipped locales whose templates are drafts no native reviewer has
/// signed yet (Q42): the answer marks their sentences so, and the design
/// page lists every string a reviewer reads.
const DRAFTS: [&str; 1] = ["ne-Deva-NP"];

/// One day to be said: the date, the place, the clock and the locale.
///
/// ```
/// use teistro::SentencesRequest;
///
/// let asked = SentencesRequest::from_json(
///     r#"{"date": {"year": 2026, "month": 3, "day": 18}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "locale": "ne-Deva-NP"}"#,
/// )?;
/// assert_eq!(asked.locale.as_deref(), Some("ne-Deva-NP"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SentencesRequest {
    /// The date, in its calendar.
    pub date: CalendarDate,
    /// The place.
    pub place: Place,
    /// The offset the civil day and its times run at.
    pub offset: UtcOffset,
    /// The locale to say the day in; the context's when left out.
    pub locale: Option<String>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SentencesRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SentencesAsked {
    /// The calendar `date` is in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    date: DayAsked,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    /// The locale to say the day in; the context's when left out.
    #[serde(default)]
    locale: Option<String>,
}

impl SentencesRequest {
    /// The record a binding sends, as JSON: `date` as `{"year", "month",
    /// "day"}` in `calendar` (Gregorian when left out), `latitudeDeg`,
    /// `longitudeDeg`, `altitudeM`, `utcOffsetSeconds` and optionally
    /// `locale`.
    ///
    /// ```
    /// use teistro::SentencesRequest;
    ///
    /// let refused = SentencesRequest::from_json(r#"{"date": {"year": 2026, "month": 3, "day": 18}}"#)
    ///     .unwrap_err();
    /// assert_eq!(refused.field(), Some("sentences"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, or a place or offset
    /// out of range, each named under `sentences`.
    pub fn from_json(text: &str) -> Result<SentencesRequest, Error> {
        let asked: SentencesAsked = teistro_core::strict::read(text, SENTENCES)?;
        asked.request().map_err(|why| why.under(SENTENCES))
    }
}

impl SentencesAsked {
    fn request(self) -> Result<SentencesRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(SentencesRequest {
            date: self.date.in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            locale: self.locale,
        })
    }
}

/// One sentence of a day.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DaySentence {
    /// The template, as `sdk.day.tithi`.
    pub key: String,
    /// Its slots, each a value of the day's answer, as `ts_intl_render`
    /// takes them.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
    /// The sentence.
    pub text: String,
}

/// A day said in a locale.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DaySentences {
    /// The locale the sentences are in.
    pub locale: String,
    /// Whether its templates still wait for a native reviewer's sign-off
    /// (Q42).
    pub draft: bool,
    /// The sentences, in the order a printed day runs.
    pub sentences: Vec<DaySentence>,
    /// The facts a polar day holds elsewhere and so does not say here:
    /// empty on any other day (`03-design/day-in-sentences.md` §4).
    pub withheld: Vec<String>,
}

/// Refuses a locale that is not loaded or that does not itself carry every
/// template. A fallback would answer another language under the asked
/// tag's name, so a locale's own messages are what is checked.
fn templates_in(intl: &Intl, tag: &str) -> Result<(), Error> {
    if !intl.locales().any(|locale| locale.tag == tag) {
        let known: Vec<&str> = intl.locales().map(|l| l.tag.as_str()).collect();
        return Err(
            Error::unsupported(format!("the locale `{tag}` is not loaded"))
                .with_field("locale")
                .with_hint(format!("the locales are {}", known.join(", "))),
        );
    }
    let carried = |key: &str| {
        intl.resolve_from(tag, key)
            .is_some_and(|(locale, _)| locale.tag == tag)
    };
    if DAY_KEYS.iter().all(|key| carried(key)) {
        return Ok(());
    }
    Err(Error::unsupported(format!(
        "`{tag}` has no templates for a day in sentences: Hindi and Sanskrit wait on \
         their reviewers (Q42), and no sentence is machine-written"
    ))
    .with_field("locale")
    .with_hint(String::from(
        "ask `en-Latn` or `ne-Deva-NP`, or load a pack that carries `sdk.day`",
    )))
}

impl AlmanacArea<'_> {
    /// **A day in sentences**: the facts of one date as short declarative
    /// sentences from templates, every value taken from the day's answer
    /// (`03-design/day-in-sentences.md`).
    ///
    /// ```no_run
    /// # fn build_context() -> teistro::Context { unimplemented!() }
    /// # let sdk = build_context();
    /// use teistro::SentencesRequest;
    ///
    /// let asked = SentencesRequest::from_json(
    ///     r#"{"date": {"year": 2026, "month": 3, "day": 18}, "latitudeDeg": 27.7172,
    ///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
    /// )?;
    /// for sentence in sdk.almanac().sentences(&asked)?.value.sentences {
    ///     println!("{}", sentence.text);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` on `locale` for a locale not loaded or one without
    /// the templates, which a fallback does not answer; otherwise as
    /// [`AlmanacArea::day`].
    pub fn sentences(self, asked: &SentencesRequest) -> Result<Envelope<DaySentences>, Error> {
        let tag = {
            let intl = self.context.locale_engine();
            let tag = asked
                .locale
                .clone()
                .unwrap_or_else(|| intl.locale().to_owned());
            templates_in(&intl, &tag)?;
            tag
        };
        let day = self.day(&asked.date, &asked.place, asked.offset)?;
        let DayPlan { plan, withheld } = day_sentences(&day.value, &asked.offset);
        let intl = self.context.locale_engine();
        let sentences = plan
            .items
            .into_iter()
            .map(|item| {
                let text = intl.render_in(&tag, &item.key, &item.params).text;
                DaySentence {
                    key: item.key,
                    params: item.params,
                    text,
                }
            })
            .collect();
        let draft = DRAFTS.contains(&tag.as_str());
        Ok(Envelope::sealing(
            DaySentences {
                locale: tag,
                draft,
                sentences,
                withheld,
            },
            day.provenance,
        ))
    }
}
