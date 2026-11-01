//! `sdk.verify`: what a text claims of the time, checked against what the
//! SDK computes for an instant and a place (`03-design/lexicon.md` §3,
//! C1095).
//!
//! The grammar is narrow on purpose. A sentence is read for the catalogue
//! members it names (`intl.lookup`), and a claim is a member of a day's
//! kind, a graha in a sign, or the Moon in a nakshatra. Everything else a
//! sentence says is not read, and a claim whose sentence negates, asks or
//! names another time is answered unread with its reason rather than
//! judged.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_intl::lexicon::{LookupMatch, LookupRequest, fold};
use teistro_panchanga::sankalpa::AyanaReckoning;

use crate::SankalpaRequest;
use crate::context::Context;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const CLAIMS: &str = "claims";

/// The kinds a claim of the day names, each with where the sankalpa's
/// answer holds the computed member (a JSON pointer).
const DAY_KINDS: [(&str, &str); 10] = [
    ("tithi", "/tithi"),
    ("paksha", "/paksha"),
    ("nakshatra", "/nakshatra"),
    ("yoga", "/yoga"),
    ("karana", "/karana"),
    ("vara", "/vara"),
    ("masa", "/month/month"),
    ("samvatsara", "/year/samvatsara"),
    ("ritu", "/ritu"),
    ("ayana", "/ayana"),
];

/// The grahas whose sidereal sign the sankalpa gives, and where.
const SIGNS: [(&str, &str); 3] = [
    ("graha.SUN", "/sun"),
    ("graha.MOON", "/moon"),
    ("graha.JUPITER", "/jupiter"),
];

/// The one graha a nakshatra is claimed of: the day's nakshatra is the
/// Moon's.
const STAR_OF: &str = "graha.MOON";

/// The kind of a graha, which is a claim's subject and not a claim.
const GRAHA: &str = "graha";

/// The kind of a sign, which is claimed of a graha.
const RASHI: &str = "rashi";

/// Every kind a sentence is looked up for.
const LOOKED_UP: [&str; 12] = [
    "tithi",
    "paksha",
    "nakshatra",
    "yoga",
    "karana",
    "vara",
    "masa",
    "samvatsara",
    "ritu",
    "ayana",
    RASHI,
    GRAHA,
];

/// What ends a sentence. A claim is read within one.
const ENDS: [char; 7] = ['.', '?', '!', ';', '।', '॥', '\n'];

/// The words that negate a sentence, compared lowercased with any
/// apostrophe removed.
const NEGATIONS: [&str; 17] = [
    "not",
    "no",
    "isnt",
    "arent",
    "wasnt",
    "never",
    "नहीं",
    "नहि",
    "नही",
    "होइन",
    "हैन",
    "छैन",
    "थिएन",
    "hoina",
    "haina",
    "chaina",
    "nahi",
];

/// The words that name a time other than the instant asked.
const OTHER_TIMES: [&str; 13] = [
    "tomorrow",
    "yesterday",
    "भोलि",
    "हिजो",
    "परसि",
    "पर्सि",
    "कल",
    "परसों",
    "bholi",
    "hijo",
    "parsi",
    "kal",
    "parson",
];

/// The words that name a kind, which narrow a span naming members of
/// several kinds to that one. Compared by their fold
/// ([`teistro_intl::lexicon::fold`]).
const KIND_WORDS: [(&str, &[&str]); 11] = [
    ("tithi", &["tithi", "तिथि"]),
    ("paksha", &["paksha", "पक्ष", "fortnight"]),
    ("nakshatra", &["nakshatra", "नक्षत्र", "star"]),
    ("yoga", &["yoga", "योग"]),
    ("karana", &["karana", "करण"]),
    ("vara", &["vara", "वार", "बार", "weekday"]),
    (
        "masa",
        &["month", "masa", "मास", "महिना", "महीना", "mahina"],
    ),
    ("samvatsara", &["samvatsara", "संवत्सर"]),
    ("ritu", &["ritu", "ऋतु", "season"]),
    ("ayana", &["ayana", "अयन"]),
    (RASHI, &["rashi", "राशि", "sign"]),
];

/// The words naming a fortnight before a tithi, which settle which of
/// its two fortnights' members the tithi is: folded, as `Shukla`,
/// `शुक्ल` and `sudi` are for the bright.
const BRIGHT: [&str; 3] = ["sukl", "sudi", "sud"];
/// As [`BRIGHT`], for the dark fortnight: `Krishna`, `कृष्ण`, `badi`.
const DARK: [&str; 3] = ["krisn", "badi", "bad"];

/// What a text claims, and the instant and place it is judged at.
///
/// ```
/// use teistro::verify::ClaimsRequest;
///
/// let asked = ClaimsRequest::from_json(
///     r#"{"text": "Today is Ekadashi.", "at": 2461055.59375, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
/// )?;
/// assert_eq!(asked.text, "Today is Ekadashi.");
/// assert!(asked.locale.is_none());
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ClaimsRequest {
    /// The text.
    pub text: String,
    /// The locale whose names are searched; every loaded locale's when
    /// left out.
    pub locale: Option<String>,
    /// The instant and place the claims are judged at.
    pub at: SankalpaRequest,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ClaimsRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ClaimsAsked {
    /// The text whose claims are checked.
    text: String,
    /// The instant the claims are judged at, a UTC Julian day.
    at: f64,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds,
    /// east positive.
    utc_offset_seconds: i32,
    /// How the ayana is reckoned: `SIDEREAL`, at Makara and Karka
    /// sankranti, when left out.
    #[serde(default)]
    ayana: AyanaReckoning,
    /// The locale whose names are searched (a BCP 47 tag, by RFC 4647
    /// lookup); every loaded locale's when left out.
    #[serde(default)]
    locale: Option<String>,
}

impl ClaimsRequest {
    /// The record a binding sends, as JSON: `text`; `at`, a UTC Julian
    /// day; `latitudeDeg`, `longitudeDeg`, `altitudeM` (0 when left out)
    /// and `utcOffsetSeconds`; `ayana`, `SIDEREAL` when left out; and
    /// `locale`, every loaded locale's names when left out.
    ///
    /// ```
    /// use teistro::verify::ClaimsRequest;
    ///
    /// let nowhere = ClaimsRequest::from_json(
    ///     r#"{"text": "x", "at": 2461055.59375, "latitudeDeg": 97, "longitudeDeg": 85.324,
    ///         "utcOffsetSeconds": 20700}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(nowhere.field(), Some("claims.latitudeDeg"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or an instant, place or offset out of range, each named
    /// under `claims`.
    pub fn from_json(text: &str) -> Result<ClaimsRequest, Error> {
        let asked: ClaimsAsked = teistro_core::strict::read(text, CLAIMS)?;
        asked.request().map_err(|why| why.under(CLAIMS))
    }
}

impl ClaimsAsked {
    fn request(self) -> Result<ClaimsRequest, Error> {
        use crate::asked::{instant, offset_of, place_of};
        Ok(ClaimsRequest {
            text: self.text,
            locale: self.locale,
            at: SankalpaRequest {
                at: instant(self.at, "at")?,
                place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
                offset: offset_of(self.utc_offset_seconds)?,
                ayana: self.ayana,
            },
        })
    }
}

/// What a claim was found to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ClaimVerdict"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    /// The SDK computes a member the claim names.
    Held,
    /// The SDK computes another member of the kind, which the claim
    /// carries as `computed`.
    Contradicted,
    /// The claim was not judged, for its `reason`.
    Unread,
}

/// Why a claim was not judged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "UnreadReason"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Unread {
    /// Its sentence is a question.
    Question,
    /// Its sentence holds a negation.
    Negated,
    /// Its sentence names another day: tomorrow, yesterday.
    OtherTime,
    /// It places a graha the sankalpa does not: Mars in a sign, the Sun
    /// in a nakshatra.
    NotComputed,
    /// It names members of several kinds that nothing in its sentence
    /// settles, a sign with no graha, or two grahas.
    Ambiguous,
}

/// One claim a text makes, and what it was found to be.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "CheckedClaim"))]
#[serde(rename_all = "camelCase")]
pub struct Claim {
    /// Where the span naming the member starts, in Unicode scalar values
    /// from the text's start.
    pub start: usize,
    /// Where it ends, exclusive.
    pub end: usize,
    /// The span.
    pub text: String,
    /// The kind claimed (`tithi`, `rashi`); none on a claim unread as
    /// ambiguous between kinds.
    pub kind: Option<String>,
    /// The graha the claim is of, when it is one's sign or nakshatra.
    pub graha: Option<String>,
    /// Every member the span may name, each a full key: the claim holds
    /// when the SDK computes any of them.
    pub claimed: Vec<String>,
    /// What the claim was found to be.
    pub verdict: Verdict,
    /// The member the SDK computes, a full key, on a claim judged.
    pub computed: Option<String>,
    /// Why the claim was not judged, on one unread.
    pub reason: Option<Unread>,
}

/// Every claim a text makes, in the text's order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Claims {
    /// The claims.
    pub claims: Vec<Claim>,
}

/// `sdk.verify`: what a text claims of the time, checked against what the
/// SDK computes.
#[derive(Clone, Copy, Debug)]
pub struct VerifyArea<'a> {
    context: &'a Context,
}

impl<'a> VerifyArea<'a> {
    pub(crate) fn of(context: &'a Context) -> VerifyArea<'a> {
        VerifyArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Every claim a text makes of the time, each held, contradicted (with
    /// the member computed) or unread (with why), judged at the request's
    /// instant and place against what `almanac.sankalpa` computes there
    /// (`03-design/lexicon.md` §3, C1095).
    ///
    /// ```no_run
    /// use teistro::verify::ClaimsRequest;
    ///
    /// let sdk = teistro::Context::builder().profile("nepali-drik").build()?;
    /// let asked = ClaimsRequest::from_json(
    ///     r#"{"text": "आज एकादशी हो। चन्द्रमा रोहिणीमा छ।", "at": 2461055.59375,
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
    /// )?;
    /// for claim in sdk.verify().claims(&asked)?.value.claims {
    ///     println!("{} {:?} {:?}", claim.text, claim.verdict, claim.computed);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A locale the context does not answer, named under
    /// `claims.locale`; otherwise as [`crate::AlmanacArea::sankalpa`].
    pub fn claims(self, request: &ClaimsRequest) -> Result<Envelope<Claims>, Error> {
        let Envelope { value, provenance } =
            self.context.almanac().sankalpa_request(&request.at)?;
        let mut claims = Vec::new();
        for sentence in sentences(&request.text) {
            let asked = LookupRequest {
                text: sentence.text.to_owned(),
                locale: request.locale.clone(),
                kinds: Some(LOOKED_UP.iter().map(|k| (*k).to_owned()).collect()),
            };
            let found = self
                .context
                .intl()
                .lookup(&asked)
                // The kinds are the grammar's own, so only the locale can
                // be refused.
                .map_err(|why| why.with_field("locale").under(CLAIMS))?;
            claims.extend(read(&sentence, &found.matches, &value));
        }
        Ok(Envelope::sealing(Claims { claims }, provenance))
    }
}

/// One sentence of the text, where it starts and whether it asks.
struct Sentence<'t> {
    text: &'t str,
    /// Where it starts, in scalar values.
    start: usize,
    /// Whether it ends in `?`.
    asks: bool,
}

/// The text cut at [`ENDS`].
fn sentences(text: &str) -> Vec<Sentence<'_>> {
    let mut found = Vec::new();
    let (mut from, mut from_chars) = (0, 0);
    for (count, (at, c)) in text.char_indices().enumerate() {
        if ENDS.contains(&c) {
            let said = text.get(from..at).unwrap_or_default();
            if !said.trim().is_empty() {
                found.push(Sentence {
                    text: said,
                    start: from_chars,
                    asks: c == '?',
                });
            }
            from = at + c.len_utf8();
            from_chars = count + 1;
        }
    }
    if text.get(from..).is_some_and(|rest| !rest.trim().is_empty()) {
        found.push(Sentence {
            text: text.get(from..).unwrap_or_default(),
            start: from_chars,
            asks: false,
        });
    }
    found
}

/// One word of a sentence: where it starts and ends in scalar values,
/// and its spelling lowercased with any apostrophe dropped.
struct Word {
    start: usize,
    end: usize,
    spelled: String,
}

/// The words of a sentence.
fn words(text: &str) -> Vec<Word> {
    let in_word = |c: char| {
        c.is_alphanumeric()
            || ('\u{0900}'..='\u{097F}').contains(&c)
            || matches!(c, '\u{200C}' | '\u{200D}' | '\'' | '\u{2019}')
    };
    let mut found: Vec<Word> = Vec::new();
    let mut current: Option<Word> = None;
    for (at, c) in text.chars().enumerate() {
        if in_word(c) {
            let word = current.get_or_insert_with(|| Word {
                start: at,
                end: at,
                spelled: String::new(),
            });
            word.end = at + 1;
            if !matches!(c, '\'' | '\u{2019}') {
                word.spelled.extend(c.to_lowercase());
            }
        } else if let Some(word) = current.take() {
            found.push(word);
        }
    }
    found.extend(current);
    found
}

/// The kind a full key is of: `tithi` for `tithi.SHUKLA_EKADASHI`.
fn kind_of(key: &str) -> &str {
    key.split_once('.').map_or(key, |(kind, _)| kind)
}

/// A member's key as the sankalpa computes it, at a JSON pointer.
fn computed<'v>(sankalpa: &'v serde_json::Value, pointer: &str) -> Option<&'v str> {
    sankalpa
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
}

/// Why a whole sentence is not of now, read before any claim of it: a
/// question, a negation, another day.
fn said(sentence: &Sentence<'_>, words: &[Word]) -> Option<Unread> {
    let said_any = |listed: &[&str]| words.iter().any(|w| listed.contains(&w.spelled.as_str()));
    if sentence.asks {
        Some(Unread::Question)
    } else if said_any(&NEGATIONS) {
        Some(Unread::Negated)
    } else if said_any(&OTHER_TIMES) {
        Some(Unread::OtherTime)
    } else {
        None
    }
}

/// The kinds a sentence names by their own words (`tithi`, `राशि`).
fn named(words: &[Word]) -> BTreeSet<&'static str> {
    words
        .iter()
        .flat_map(|w| {
            let folded = fold(&w.spelled);
            KIND_WORDS
                .iter()
                .filter(move |(_, spelled)| spelled.iter().any(|s| fold(s) == folded))
                .map(|(kind, _)| *kind)
        })
        .collect()
}

/// The claims one sentence makes.
fn read(
    sentence: &Sentence<'_>,
    matches: &[LookupMatch],
    sankalpa: &serde_json::Value,
) -> Vec<Claim> {
    let words = words(sentence.text);
    let said = said(sentence, &words);
    let named = named(&words);
    let only_grahas = |m: &LookupMatch| m.candidates.iter().all(|c| kind_of(&c.key) == GRAHA);
    // The grahas the sentence names: a span every candidate of which is
    // a graha.
    let mut grahas: BTreeSet<&str> = matches
        .iter()
        .filter(|m| only_grahas(m))
        .flat_map(|m| m.candidates.iter().map(|c| c.key.as_str()))
        .collect();
    let subject = match (grahas.pop_first(), grahas.is_empty()) {
        (None, _) => Subject::None,
        (Some(graha), true) => Subject::One(graha.to_owned()),
        (Some(_), false) => Subject::Several,
    };
    // A fortnight's word just before a tithi (`Shukla Ekadashi`, `कृष्ण
    // अष्टमी`, `sudi dashami`) joins the tithi's span and settles which
    // fortnight's member it is.
    let mut spans: Vec<(Span, Vec<String>)> = Vec::new();
    let mut taken: Vec<(usize, usize)> = Vec::new();
    for found in matches {
        if only_grahas(found) {
            continue;
        }
        let mut span = Span {
            start: found.start,
            end: found.end,
            text: found.text.clone(),
        };
        let mut keys: Vec<String> = found.candidates.iter().map(|c| c.key.clone()).collect();
        let before = words.iter().rev().find(|w| w.end <= found.start);
        if let Some(word) = before
            && let Some(prefix) = fortnight(&word.spelled)
            && sentence
                .text
                .chars()
                .skip(word.end)
                .take(found.start - word.end)
                .all(char::is_whitespace)
        {
            let settled: Vec<String> = keys
                .iter()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect();
            if !settled.is_empty() {
                keys = settled;
                span.start = word.start;
                span.text = sentence
                    .text
                    .chars()
                    .skip(word.start)
                    .take(found.end - word.start)
                    .collect();
                taken.push((word.start, word.end));
            }
        }
        spans.push((span, keys));
    }
    spans
        .into_iter()
        // The fortnight's own word, which may have been found apart (a
        // samvatsara is named `Shukla`), is not a claim of its own.
        .filter(|(span, _)| {
            !taken
                .iter()
                .any(|&(start, end)| span.start >= start && span.end <= end)
        })
        .map(|(span, keys)| {
            judge(
                Span {
                    start: sentence.start + span.start,
                    end: sentence.start + span.end,
                    text: span.text,
                },
                keys,
                said,
                &subject,
                &named,
                sankalpa,
            )
        })
        .collect()
}

/// The fortnight a word before a tithi names, as the prefix of its
/// tithis' keys.
fn fortnight(word: &str) -> Option<&'static str> {
    let folded = fold(word);
    if BRIGHT.contains(&folded.as_str()) {
        Some("tithi.SHUKLA_")
    } else if DARK.contains(&folded.as_str()) {
        Some("tithi.KRISHNA_")
    } else {
        None
    }
}

/// Who a sentence's claims are of.
enum Subject {
    /// No graha: the claims are of the day.
    None,
    /// One graha.
    One(String),
    /// Two or more, which the grammar does not part.
    Several,
}

/// Where a claim's span is.
struct Span {
    start: usize,
    end: usize,
    text: String,
}

/// One span's claim, judged.
fn judge(
    span: Span,
    mut keys: Vec<String>,
    said: Option<Unread>,
    subject: &Subject,
    named: &BTreeSet<&'static str>,
    sankalpa: &serde_json::Value,
) -> Claim {
    let kinds = |keys: &[String]| -> BTreeSet<String> {
        keys.iter().map(|k| kind_of(k).to_owned()).collect()
    };
    // A graha leaves a sign and a nakshatra; no graha leaves the rest.
    let placed = |k: &String| matches!(kind_of(k), RASHI | "nakshatra");
    let narrowed: Vec<String> = match subject {
        Subject::None => keys
            .iter()
            .filter(|k| kind_of(k) != RASHI)
            .cloned()
            .collect(),
        Subject::One(_) | Subject::Several => keys.iter().filter(|k| placed(k)).cloned().collect(),
    };
    if !narrowed.is_empty() {
        keys = narrowed;
    }
    // A kind's own word leaves that kind.
    let by_word: Vec<String> = keys
        .iter()
        .filter(|k| named.contains(kind_of(k)))
        .cloned()
        .collect();
    if !by_word.is_empty() {
        keys = by_word;
    }
    let kinds = kinds(&keys);
    let kind = (kinds.len() == 1).then(|| kinds.into_iter().next().unwrap_or_default());
    let graha = match (subject, kind.as_deref()) {
        (Subject::One(graha), Some(RASHI | "nakshatra")) => Some(graha.clone()),
        _ => None,
    };
    let unread = |reason| Claim {
        start: span.start,
        end: span.end,
        text: span.text.clone(),
        kind: kind.clone(),
        graha: graha.clone(),
        claimed: keys.clone(),
        verdict: Verdict::Unread,
        computed: None,
        reason: Some(reason),
    };
    if let Some(reason) = said {
        return unread(reason);
    }
    let pointer = match (kind.as_deref(), subject) {
        (None, _) | (Some(RASHI | "nakshatra"), Subject::Several) => {
            return unread(Unread::Ambiguous);
        }
        // A sign with no graha: whose?
        (Some(RASHI), Subject::None) => return unread(Unread::Ambiguous),
        (Some(RASHI), Subject::One(graha)) => {
            match SIGNS.iter().find(|(g, _)| *g == graha.as_str()) {
                Some((_, pointer)) => *pointer,
                None => return unread(Unread::NotComputed),
            }
        }
        (Some("nakshatra"), Subject::One(graha)) if graha != STAR_OF => {
            return unread(Unread::NotComputed);
        }
        (Some(kind), _) => match DAY_KINDS.iter().find(|(k, _)| *k == kind) {
            Some((_, pointer)) => *pointer,
            None => return unread(Unread::NotComputed),
        },
    };
    let Some(computed) = computed(sankalpa, pointer) else {
        return unread(Unread::NotComputed);
    };
    let held = keys.iter().any(|k| k == computed);
    Claim {
        start: span.start,
        end: span.end,
        text: span.text,
        kind,
        graha,
        claimed: keys,
        verdict: if held {
            Verdict::Held
        } else {
            Verdict::Contradicted
        },
        computed: Some(computed.to_owned()),
        reason: None,
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests fail by panicking")]

    use super::{sentences, words};

    #[test]
    fn a_text_is_cut_into_sentences_and_words() {
        let cut = sentences("आज एकादशी हो। Is it Rohini?\nyes");
        let parts: Vec<(&str, usize, bool)> =
            cut.iter().map(|s| (s.text, s.start, s.asks)).collect();
        assert_eq!(
            parts,
            [
                ("आज एकादशी हो", 0, false),
                (" Is it Rohini", 13, true),
                ("yes", 28, false)
            ]
        );
        let found = words("It isn't Ekadashi, नहीं");
        let spelled: Vec<&str> = found.iter().map(|w| w.spelled.as_str()).collect();
        assert_eq!(spelled, ["it", "isnt", "ekadashi", "नहीं"]);
        assert_eq!((found[3].start, found[3].end), (19, 23));
    }
}
