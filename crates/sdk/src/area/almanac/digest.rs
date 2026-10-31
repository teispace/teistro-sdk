//! The N-day digest: a range of days, laid out for a small process that
//! cannot run the engine (`03-design/digest-format.md`, SR-08).
//!
//! A `TSRB` result blob of the `digest` schema, every time a whole second
//! from the first day's local midnight and every name interned once, so
//! 60 days are tens of kilobytes and a reader in C opens them with no
//! allocation (`bindings/c/digest`). No clock is read and no float is
//! written but the place's own, so the same request writes the same bytes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Calendar, Catalogued, Kaala, Tithi};
use teistro_core::envelope::{Envelope, Hash, Provenance, canonical_json};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_idl::blob::{ColumnData, FixedValue, Writer};
use teistro_idl::model::{BlobSchema, ColumnDef, Scalar, SectionSchema};
use teistro_panchanga::Panchanga;
use teistro_panchanga::Span;
use teistro_panchanga::festival::{Observances, PublishedStatus};
use teistro_time::{DayState, PolarKind, ZoneSpec};

use super::feed::{FeedForm, kept_days};
use super::{AlmanacArea, DayClock};
use crate::area::system_of;
use crate::asked::{DayAsked, place_of};
use crate::festival_request::FestivalRequest;

/// The field a digest request's refusals are named under.
const DIGEST: &str = "digest";

/// The schema's name.
///
/// ```
/// assert_eq!(teistro::DIGEST_SCHEMA, "digest");
/// ```
pub const DIGEST_SCHEMA: &str = DIGEST;

/// The format's major version: a reader refuses any other. Raised when a
/// reader of the last would misread, never for a section added at the end.
pub const DIGEST_FORMAT_MAJOR: u32 = 1;

/// The format's minor version, raised when a section is added: 1 added
/// `clock`, each day's own offset.
pub const DIGEST_FORMAT_MINOR: u32 = 1;

/// The days a digest holds when the request does not say.
pub const DIGEST_DEFAULT_DAYS: u32 = 60;

/// The most days a digest holds: a year and a day, past which the times'
/// `i32` still fits and the consumer's few hundred kilobytes do not.
pub const DIGEST_MAX_DAYS: u32 = 366;

/// An instant a day does not have, in a time column.
pub const DIGEST_NO_TIME: i32 = i32::MIN;

/// A span with no name of its own, in a name column.
pub const DIGEST_NO_NAME: u16 = u16::MAX;

/// What a digest's span is. The numbers are the format's and never change.
///
/// ```
/// use teistro::DigestSpan;
///
/// assert_eq!(DigestSpan::Tithi as u8, 1);
/// assert_eq!(DigestSpan::SunSign as u8, 18);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DigestSpan {
    /// The tithi.
    Tithi = 1,
    /// The nakshatra.
    Nakshatra = 2,
    /// The nitya yoga.
    Yoga = 3,
    /// The karana.
    Karana = 4,
    /// Rahu kaala.
    RahuKaala = 5,
    /// Yamaghanda.
    Yamaghanda = 6,
    /// Gulika kaala.
    GulikaKaala = 7,
    /// A choghadiya.
    Choghadiya = 8,
    /// A hora, named by its lord.
    Hora = 9,
    /// Abhijit muhurta.
    Abhijit = 10,
    /// Bhadra, named by where she dwells.
    Bhadra = 11,
    /// Brahma muhurta.
    BrahmaMuhurta = 12,
    /// A durmuhurta.
    Durmuhurta = 13,
    /// The varjyam.
    Varjyam = 14,
    /// The amrita kalam.
    AmritKalam = 15,
    /// The panchaka.
    Panchaka = 16,
    /// The sign the Moon stood in.
    MoonSign = 17,
    /// The sign the Sun stood in.
    SunSign = 18,
}

/// The `flags` bit of an observance that is a fast.
pub const DIGEST_FLAG_FAST: u8 = 1;

/// The `flags` bit of an observance a published set decided.
pub const DIGEST_FLAG_PUBLISHED: u8 = 2;

/// The digest's blob schema, which the C reader in `bindings/c/digest`
/// holds by hand: a section added here goes before `seal`, and a change to
/// one is a major version.
///
/// ```
/// let schema = teistro::digest_schema();
/// assert_eq!((schema.id, schema.name.as_str()), (6, "digest"));
/// assert_eq!(schema.sections.last().map(|section| section.name.as_str()), Some("seal"));
/// ```
#[must_use]
pub fn digest_schema() -> BlobSchema {
    const TIME: &str = "seconds from `base_unix_seconds`";
    BlobSchema {
        name: DIGEST_SCHEMA.to_owned(),
        id: 6,
        doc: "A range of days for a process that cannot run the engine (`03-design/digest-format.md`): every limb and window of each day, its Moon and its observances, every time whole seconds from the first day's local midnight, every name interned once, sealed by SHA-256.".to_owned(),
        sections: vec![
            SectionSchema::fixed(1, "summary", "The format's version, the range and where it was reckoned.", vec![
                ColumnDef::new("format_major", Scalar::U32, "The format's major version; a reader refuses another."),
                ColumnDef::new("format_minor", Scalar::U32, "The format's minor version; a reader skips what it does not know."),
                ColumnDef::new("day_count", Scalar::U32, "How many days the digest holds, the rows of `days`."),
                ColumnDef::new("base_unix_seconds", Scalar::I64, "The first day's local midnight on that day's offset, in seconds since 1970, which every time counts from."),
                ColumnDef::new("utc_offset_seconds", Scalar::I32, "The first day's offset, east positive; `clock` has each day's."),
                ColumnDef::new("calendar", Scalar::U16, "The calendar `days` dates are read in.").of_enum("Calendar"),
                ColumnDef::new("latitude_deg", Scalar::F64, "The place's latitude, degrees north."),
                ColumnDef::new("longitude_deg", Scalar::F64, "The place's longitude, degrees east."),
                ColumnDef::new("altitude_m", Scalar::F64, "The place's altitude, metres."),
            ]),
            SectionSchema::columns(2, "days", "One row a day, in order.", vec![
                ColumnDef::new("year", Scalar::I32, "The date's year, in `summary.calendar`."),
                ColumnDef::new("month", Scalar::U8, "The date's month."),
                ColumnDef::new("day", Scalar::U8, "The date's day of the month."),
                ColumnDef::new("vara", Scalar::U8, "The vara, 0 for Sunday."),
                ColumnDef::new("state", Scalar::U8, "0 an ordinary day, 1 the Sun did not rise, 2 it did not set, when `sunrise` and `sunset` are the policy's."),
                ColumnDef::new("fixed_day", Scalar::I32, "The day's fixed number, 1 for 0001-01-01 Gregorian."),
                ColumnDef::new("sunrise", Scalar::I32, TIME),
                ColumnDef::new("sunset", Scalar::I32, TIME),
                ColumnDef::new("moonrise", Scalar::I32, "The day's first moonrise, in seconds from `base_unix_seconds`, or `i32::MIN`."),
                ColumnDef::new("moonset", Scalar::I32, "The day's first moonset, in seconds from `base_unix_seconds`, or `i32::MIN`."),
                ColumnDef::new("moon_phase", Scalar::U16, "Thousandths of a lunation at sunrise, 0 new and 500 full, read from the tithi that held it and how far through it sunrise fell."),
                ColumnDef::new("lunar_month", Scalar::U16, "The lunar month's row in `names`."),
                ColumnDef::new("span_first", Scalar::U32, "The day's first row in `spans`."),
                ColumnDef::new("span_count", Scalar::U16, "The day's rows in `spans`."),
                ColumnDef::new("observance_first", Scalar::U32, "The day's first row in `observances`."),
                ColumnDef::new("observance_count", Scalar::U16, "The day's rows in `observances`."),
            ]),
            SectionSchema::columns(3, "spans", "Each day's limbs and windows, by kind and then in the order they run. A limb's span is the whole limb, so one may run past either end of its day.", vec![
                ColumnDef::new("kind", Scalar::U8, "What the span is.").of_enum("TsDigestSpan"),
                ColumnDef::new("name", Scalar::U16, "Its row in `names`, or 65535 when it has no name of its own."),
                ColumnDef::new("from", Scalar::I32, TIME),
                ColumnDef::new("to", Scalar::I32, TIME),
            ]),
            SectionSchema::columns(4, "observances", "Each day's observances, sorted by rule.", vec![
                ColumnDef::new("name", Scalar::U16, "Its row in `names`."),
                ColumnDef::new("flags", Scalar::U8, "Bit 0: a fast. Bit 1: a published set decided the day."),
            ]),
            SectionSchema::columns(5, "names", "Every name the digest uses, once, in the order first met.", vec![
                ColumnDef::new("key_at", Scalar::U32, "The catalogue or rule key's offset in `text`."),
                ColumnDef::new("key_length", Scalar::U16, "Its length in octets."),
                ColumnDef::new("text_at", Scalar::U32, "The name's offset in `text`, in the context's locale."),
                ColumnDef::new("text_length", Scalar::U16, "Its length in octets."),
            ]),
            SectionSchema::bytes(6, "text", "UTF-8 text: the keys and names, with nothing between them."),
            SectionSchema::bytes(7, "locale", "UTF-8 text: the BCP 47 tag `names` is in."),
            SectionSchema::bytes(8, "provenance_json", "UTF-8 JSON, canonical: the days' provenance, its `content_hash` theirs."),
            SectionSchema::columns(10, "clock", "One row a day, in the order of `days`: the clock the day was read on, which a range asked by zone changes across a clock change. Added in format 1.1.", vec![
                ColumnDef::new("utc_offset_seconds", Scalar::I32, "The offset at the day's opening, east positive."),
            ]),
            SectionSchema::bytes(9, "seal", "SHA-256 of every byte before this section, which ends the digest, in 64 lowercase hexadecimal digits."),
        ],
    }
}

/// A digest asked for.
///
/// ```
/// use teistro::DigestRequest;
///
/// let asked = DigestRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 27.7172,
///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"}}"#,
/// )?;
/// assert_eq!(asked.days, 60);
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DigestRequest {
    /// The first day.
    pub first: CalendarDate,
    /// How many days, 1 to [`DIGEST_MAX_DAYS`].
    pub days: u32,
    /// The place.
    pub place: Place,
    /// The clock the civil days run on.
    pub clock: DayClock,
    /// The festivals to list; none lists none.
    pub festivals: Option<FestivalRequest>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DigestRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DigestAsked {
    /// The calendar `first` is in, and the digest's dates; Gregorian when
    /// left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    first: DayAsked,
    /// How many days, 1 to 366; 60 when left out.
    #[serde(default)]
    days: Option<u32>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    /// The clock's offset; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone, each day on its own offset; or `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The festivals to list, as a festival request; none when left out.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::festival_request::Asked>")
    )]
    festivals: Option<Value>,
}

impl DigestRequest {
    /// The record a binding sends, as JSON: `first` as `{"year", "month",
    /// "day"}` in `calendar` (Gregorian when left out), `days` (60 when
    /// left out), `latitudeDeg`, `longitudeDeg`, `altitudeM`,
    /// `utcOffsetSeconds` or `zone`, and optionally `festivals`, a festival
    /// request.
    ///
    /// ```
    /// use teistro::DigestRequest;
    ///
    /// let refused = DigestRequest::from_json(r#"{"first": {"year": 2026, "month": 1, "day": 1}}"#)
    ///     .unwrap_err();
    /// assert_eq!(refused.field(), Some("digest"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a place or offset out
    /// of range, both or neither of `utcOffsetSeconds` and `zone`, or a
    /// festival request its own reader refuses, each named under `digest`.
    pub fn from_json(text: &str) -> Result<DigestRequest, Error> {
        let asked: DigestAsked = teistro_core::strict::read(text, DIGEST)?;
        asked.request().map_err(|why| why.under(DIGEST))
    }
}

impl DigestAsked {
    fn request(self) -> Result<DigestRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(DigestRequest {
            first: self.first.in_calendar(calendar),
            days: self.days.unwrap_or(DIGEST_DEFAULT_DAYS),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            festivals: self
                .festivals
                .map(|festivals| FestivalRequest::from_json(&festivals.to_string()))
                .transpose()?,
        })
    }
}

/// The interned names, in the order first met.
#[derive(Default)]
struct Names {
    rows: BTreeMap<String, u16>,
    key_at: Vec<u32>,
    key_length: Vec<u16>,
    text_at: Vec<u32>,
    text_length: Vec<u16>,
    text: Vec<u8>,
}

impl Names {
    /// The row of `key`, adding it with what `name` says when it is new.
    fn intern(&mut self, key: &str, name: impl FnOnce() -> String) -> Result<u16, Error> {
        if let Some(&row) = self.rows.get(key) {
            return Ok(row);
        }
        let row = u16::try_from(self.key_at.len())
            .ok()
            .filter(|&row| row != DIGEST_NO_NAME)
            .ok_or_else(|| Error::new(Status::Limit, "a digest holds fewer than 65535 names"))?;
        let (at, length) = self.push(key)?;
        self.key_at.push(at);
        self.key_length.push(length);
        let (at, length) = self.push(&name())?;
        self.text_at.push(at);
        self.text_length.push(length);
        self.rows.insert(key.to_owned(), row);
        Ok(row)
    }

    fn push(&mut self, text: &str) -> Result<(u32, u16), Error> {
        let too_long = || Error::new(Status::Limit, "a digest's name or its text is too long");
        let at = u32::try_from(self.text.len()).map_err(|_| too_long())?;
        let length = u16::try_from(text.len()).map_err(|_| too_long())?;
        self.text.extend_from_slice(text.as_bytes());
        Ok((at, length))
    }
}

/// The columns of a section, row by row.
#[derive(Default)]
struct Spans {
    kind: Vec<u8>,
    name: Vec<u16>,
    from: Vec<i32>,
    to: Vec<i32>,
}

/// The clock of a digest: seconds from its base, and the offset of a day
/// that carries no clock of its own.
#[derive(Clone, Copy)]
struct Clock {
    base_unix_seconds: i64,
    stated: UtcOffset,
}

impl Clock {
    /// The offset a day was read on: its own, asked by zone, or the one
    /// the caller stated.
    fn offset_of(self, day: &Panchanga) -> UtcOffset {
        day.clock.as_ref().map_or(self.stated, |clock| clock.offset)
    }

    fn seconds(self, at: JulianDay<Utc>) -> Result<i32, Error> {
        let seconds = (at.get() - JulianDay::<Utc>::UNIX_EPOCH.get()) * 86_400.0;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a day of the years the calendar holds is far inside i64"
        )]
        let whole = seconds.round() as i64;
        i32::try_from(whole - self.base_unix_seconds)
            .map_err(|_| Error::new(Status::OutOfRange, "an instant is out of a digest's clock"))
    }
}

/// The day's Moon as thousandths of a lunation at sunrise: the tithi that
/// held sunrise, and how far through it sunrise fell.
fn moon_phase(day: &Panchanga) -> u16 {
    let sunrise = day.day.sunrise.get();
    let held = day
        .limbs
        .tithi
        .iter()
        .find(|span| span.whole.from.get() <= sunrise && sunrise < span.whole.to.get())
        .or_else(|| day.limbs.tithi.first());
    let Some(span) = held else {
        return 0;
    };
    let (from, to) = (span.whole.from.get(), span.whole.to.get());
    let through = if to > from {
        ((sunrise - from) / (to - from)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let index = f64::from(Tithi::id(span.member));
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a share of a lunation in thousandths, from 0 to 1000"
    )]
    let phase = ((index + through) * 1_000.0 / 30.0).round() as u16;
    phase % 1_000
}

/// A digest being written, day by day.
struct Rows<'a> {
    area: AlmanacArea<'a>,
    clock: Clock,
    names: Names,
    spans: Spans,
}

impl Rows<'_> {
    /// The name of a catalogue member in the context's locale, or its key.
    fn member(&mut self, member: impl Catalogued) -> Result<u16, Error> {
        let key = member.full_key();
        let area = self.area;
        self.names.intern(&key, || area.entity_name(&key))
    }

    fn span(&mut self, kind: DigestSpan, name: u16, at: Interval) -> Result<(), Error> {
        self.spans.kind.push(kind as u8);
        self.spans.name.push(name);
        self.spans.from.push(self.clock.seconds(at.from)?);
        self.spans.to.push(self.clock.seconds(at.to)?);
        Ok(())
    }

    fn whole<T: Catalogued + Copy>(
        &mut self,
        kind: DigestSpan,
        spans: &[Span<T>],
    ) -> Result<(), Error> {
        for span in spans {
            let name = self.member(span.member)?;
            self.span(kind, name, span.whole)?;
        }
        Ok(())
    }

    /// One day's spans, by kind and then in the order they run.
    fn of(&mut self, day: &Panchanga) -> Result<(), Error> {
        self.whole(DigestSpan::Tithi, &day.limbs.tithi)?;
        self.whole(DigestSpan::Nakshatra, &day.limbs.nakshatra)?;
        self.whole(DigestSpan::Yoga, &day.limbs.yoga)?;
        self.whole(DigestSpan::Karana, &day.limbs.karana)?;
        for kaala in &day.kaalas {
            let name = self.member(kaala.kaala)?;
            let kind = match kaala.kaala {
                Kaala::RahuKaala => DigestSpan::RahuKaala,
                Kaala::Yamaghanda => DigestSpan::Yamaghanda,
                Kaala::GulikaKaala => DigestSpan::GulikaKaala,
                other => {
                    return Err(Error::new(
                        Status::Internal,
                        format!("a digest has no span kind for {}", other.full_key()),
                    ));
                }
            };
            self.span(kind, name, kaala.at)?;
        }
        for part in &day.choghadiya {
            let name = self.member(part.choghadiya)?;
            self.span(DigestSpan::Choghadiya, name, part.at)?;
        }
        for hora in &day.horas {
            let name = self.member(hora.lord)?;
            self.span(
                DigestSpan::Hora,
                name,
                Interval {
                    from: hora.start,
                    to: hora.end,
                },
            )?;
        }
        if let Some(abhijit) = day.muhurtas.abhijit {
            self.span(DigestSpan::Abhijit, DIGEST_NO_NAME, abhijit)?;
        }
        self.whole(DigestSpan::Bhadra, &day.bhadra_vasa())?;
        if let Some(brahma) = day.muhurtas.brahma {
            self.span(DigestSpan::BrahmaMuhurta, DIGEST_NO_NAME, brahma)?;
        }
        for durmuhurta in &day.durmuhurtas {
            self.span(DigestSpan::Durmuhurta, DIGEST_NO_NAME, durmuhurta.at)?;
        }
        self.whole(DigestSpan::Varjyam, &day.omens.varjyam)?;
        self.whole(DigestSpan::AmritKalam, &day.omens.amrit_kalam)?;
        self.whole(DigestSpan::Panchaka, &day.omens.panchaka)?;
        self.whole(DigestSpan::MoonSign, &day.moon.signs)?;
        self.whole(DigestSpan::SunSign, &day.sun.signs)?;
        Ok(())
    }
}

/// An observance's day index, rule, name and flags.
type Observed = (usize, String, String, u8);

/// The day's observances, sorted by day and rule, the published additions
/// among them.
fn observances(
    area: AlmanacArea<'_>,
    value: &Observances,
    first: i64,
    days: i64,
) -> Result<Vec<Observed>, Error> {
    let index = |day: &CalendarDate| -> Result<Option<usize>, Error> {
        let fixed = system_of(day.calendar)?.fixed_of(day)?.get();
        Ok((first..first + days)
            .contains(&fixed)
            .then(|| usize::try_from(fixed - first).unwrap_or(usize::MAX)))
    };
    let mut rows = Vec::new();
    for (rule, day, published, form) in kept_days(value) {
        if let Some(at) = index(day)? {
            let fast = if form == FeedForm::Ekadashi {
                DIGEST_FLAG_FAST
            } else {
                0
            };
            let decided = published.as_ref().map_or(0, |set| match set.status {
                PublishedStatus::Withdrawn => 0,
                PublishedStatus::Published => DIGEST_FLAG_PUBLISHED,
            });
            rows.push((at, rule.clone(), area.festival_name(rule), fast | decided));
        }
    }
    for added in &value.added {
        if let Some(at) = index(&added.day)? {
            rows.push((
                at,
                added.key.clone(),
                area.added_name(&added.key, &added.names),
                DIGEST_FLAG_PUBLISHED,
            ));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(rows)
}

/// The columns of `days`.
#[derive(Default)]
struct DayColumns {
    year: Vec<i32>,
    month: Vec<u8>,
    day: Vec<u8>,
    vara: Vec<u8>,
    state: Vec<u8>,
    fixed_day: Vec<i32>,
    sunrise: Vec<i32>,
    sunset: Vec<i32>,
    moonrise: Vec<i32>,
    moonset: Vec<i32>,
    moon_phase: Vec<u16>,
    utc_offset_seconds: Vec<i32>,
    lunar_month: Vec<u16>,
    span_first: Vec<u32>,
    span_count: Vec<u16>,
    observance_first: Vec<u32>,
    observance_count: Vec<u16>,
}

fn blob_error(why: &teistro_idl::blob::BlobError) -> Error {
    Error::new(Status::Internal, format!("the digest's schema: {why}"))
}

impl AlmanacArea<'_> {
    /// **The N-day digest:** the days from `asked.first`, every limb and
    /// window, the Moon and the observances, as bytes a process that
    /// cannot run the engine reads with `bindings/c/digest`
    /// (`03-design/digest-format.md`).
    ///
    /// The same request writes the same bytes on every platform: nothing
    /// reads a clock, the ephemeris and the arithmetic hold to the bit
    /// (`05-testing/01-quality-bar.md`) and every instant is written as a
    /// whole second. The envelope's content hash is SHA-256 of the bytes.
    ///
    /// ```no_run
    /// use teistro::{Context, DigestRequest};
    ///
    /// let sdk = Context::builder().nepal_official([]).build()?;
    /// let asked = DigestRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 27.7172,
    ///         "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}"#,
    /// )?;
    /// std::fs::write("widget.digest", sdk.almanac().digest(&asked)?.value)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`] and [`AlmanacArea::festivals`];
    /// `OUT_OF_RANGE`, on `days`, for a count outside 1 to
    /// [`DIGEST_MAX_DAYS`]; as [`AlmanacArea::in_zone`] for a zone.
    pub fn digest(self, asked: &DigestRequest) -> Result<Envelope<Vec<u8>>, Error> {
        if !(1..=DIGEST_MAX_DAYS).contains(&asked.days) {
            return Err(Error::new(
                Status::OutOfRange,
                format!(
                    "a digest holds 1 to {DIGEST_MAX_DAYS} days, not {}",
                    asked.days
                ),
            )
            .with_field("days"));
        }
        let system = system_of(asked.first.calendar)?;
        let first = system.fixed_of(&asked.first)?;
        let last = system.date_of(first.plus_days(i64::from(asked.days) - 1))?;
        match &asked.clock {
            DayClock::Offset(offset) => self.digest_on(asked, &last, *offset),
            DayClock::Zone(zone) => {
                let clock = zone.clock(self.context.zones())?;
                let offset = Self::steady_offset(&asked.first, &last, &clock, true)?;
                self.on(&clock).digest_on(asked, &last, offset)
            }
        }
    }

    /// The digest through `last`, on this area's clock or else `offset`.
    fn digest_on(
        self,
        asked: &DigestRequest,
        last: &CalendarDate,
        offset: UtcOffset,
    ) -> Result<Envelope<Vec<u8>>, Error> {
        let first = system_of(asked.first.calendar)?.fixed_of(&asked.first)?;
        let days = self.of(&asked.first, last, &asked.place, offset)?;
        let observed = match &asked.festivals {
            Some(festivals) => {
                let answer = self.festivals(&asked.first, last, &asked.place, offset, festivals)?;
                observances(self, &answer.value, first.get(), i64::from(asked.days))?
            }
            None => Vec::new(),
        };
        let written = Written {
            first: first.get(),
            stated: offset,
            days: &days.value,
            provenance: &days.provenance,
            observed: &observed,
        };
        let bytes = self.written(asked, &written)?;
        let provenance = Provenance {
            content_hash: Hash::of(&bytes),
            ..days.provenance
        };
        Ok(Envelope {
            value: bytes,
            provenance,
        })
    }

    /// The bytes: the rows gathered, the schema's sections written and the
    /// seal set over what precedes it.
    fn written(self, asked: &DigestRequest, from: &Written<'_>) -> Result<Vec<u8>, Error> {
        let Written {
            first,
            stated,
            days,
            provenance,
            observed,
        } = *from;
        let first_offset = days
            .first()
            .and_then(|day| day.clock.as_ref())
            .map_or(stated, |clock| clock.offset);
        let base_unix_seconds =
            (first - UNIX_EPOCH_DAY) * 86_400 - i64::from(first_offset.seconds());
        let clock = Clock {
            base_unix_seconds,
            stated,
        };
        let gathered = self.gathered(clock, first, days, observed)?;
        let locale = self.context.locale_engine().locale().to_owned();
        let meta = Meta {
            asked,
            base_unix_seconds,
            first_offset,
            locale: &locale,
            provenance,
        };
        let schema = digest_schema();
        let mut writer = Writer::new(&schema);
        encode(&mut writer, &meta, &gathered).map_err(|why| blob_error(&why))?;
        let mut bytes = writer.finish().map_err(|why| blob_error(&why))?;
        let sealed = bytes.len() - SEAL_LEN;
        let seal = hex(Hash::of(bytes.get(..sealed).unwrap_or_default()).bytes());
        if let Some(slot) = bytes.get_mut(sealed..) {
            slot.copy_from_slice(seal.as_bytes());
        }
        Ok(bytes)
    }

    /// Every section's rows, day by day.
    fn gathered(
        self,
        clock: Clock,
        first: i64,
        days: &[Panchanga],
        observed: &[Observed],
    ) -> Result<Gathered, Error> {
        let too_many = || Error::new(Status::Limit, "a digest's day holds too many rows");
        let mut rows = Rows {
            area: self,
            clock,
            names: Names::default(),
            spans: Spans::default(),
        };
        let mut columns = DayColumns::default();
        let mut observance_name: Vec<u16> = Vec::with_capacity(observed.len());
        let mut observance_flags: Vec<u8> = Vec::with_capacity(observed.len());
        for (index, day) in days.iter().enumerate() {
            let fixed = i32::try_from(first + i64::try_from(index).unwrap_or(i64::MAX))
                .map_err(|_| Error::new(Status::OutOfRange, "a digest's day is out of range"))?;
            columns.push_day(clock, day, fixed)?;
            columns.lunar_month.push(rows.member(day.month.month)?);
            let begun = rows.spans.kind.len();
            rows.of(day)?;
            columns
                .span_first
                .push(u32::try_from(begun).map_err(|_| too_many())?);
            columns
                .span_count
                .push(u16::try_from(rows.spans.kind.len() - begun).map_err(|_| too_many())?);
            columns
                .observance_first
                .push(u32::try_from(observance_name.len()).map_err(|_| too_many())?);
            let mut count = 0_u16;
            for (_, rule, name, flags) in observed.iter().filter(|row| row.0 == index) {
                observance_name.push(rows.names.intern(rule, || name.clone())?);
                observance_flags.push(*flags);
                count += 1;
            }
            columns.observance_count.push(count);
        }
        Ok(Gathered {
            columns,
            names: rows.names,
            spans: rows.spans,
            observance_name,
            observance_flags,
        })
    }
}

impl DayColumns {
    /// A day's own columns, those that read nothing but the day.
    fn push_day(&mut self, clock: Clock, day: &Panchanga, fixed: i32) -> Result<(), Error> {
        let date = &day.day.date;
        self.year.push(date.year);
        self.month.push(date.month);
        self.day.push(date.day);
        self.vara
            .push(u8::try_from(day.day.vara.id()).unwrap_or(u8::MAX));
        self.state.push(match day.day.state {
            DayState::Normal => 0,
            DayState::Polar {
                kind: PolarKind::Night,
                ..
            } => 1,
            DayState::Polar {
                kind: PolarKind::Day,
                ..
            } => 2,
        });
        self.fixed_day.push(fixed);
        self.sunrise.push(clock.seconds(day.day.sunrise)?);
        self.sunset.push(clock.seconds(day.day.sunset)?);
        let first_of = |list: &[JulianDay<Utc>]| {
            list.first()
                .map_or(Ok(DIGEST_NO_TIME), |at| clock.seconds(*at))
        };
        self.moonrise.push(first_of(&day.moon.rises)?);
        self.moonset.push(first_of(&day.moon.sets)?);
        self.moon_phase.push(moon_phase(day));
        self.utc_offset_seconds.push(clock.offset_of(day).seconds());
        Ok(())
    }
}

/// What [`AlmanacArea::written`] writes from.
#[derive(Clone, Copy)]
struct Written<'a> {
    first: i64,
    stated: UtcOffset,
    days: &'a [Panchanga],
    provenance: &'a Provenance,
    observed: &'a [Observed],
}

/// What the header sections say.
struct Meta<'a> {
    asked: &'a DigestRequest,
    base_unix_seconds: i64,
    first_offset: UtcOffset,
    locale: &'a str,
    provenance: &'a Provenance,
}

/// Every section's rows.
struct Gathered {
    columns: DayColumns,
    names: Names,
    spans: Spans,
    observance_name: Vec<u16>,
    observance_flags: Vec<u8>,
}

/// Writes every section in the schema's order, the seal zero for now.
fn encode(
    writer: &mut Writer<'_>,
    meta: &Meta<'_>,
    rows: &Gathered,
) -> Result<(), teistro_idl::blob::BlobError> {
    let Gathered {
        columns,
        names,
        spans,
        observance_name,
        observance_flags,
    } = rows;
    let asked = meta.asked;
    writer.fixed(
        "summary",
        &[
            FixedValue::from(DIGEST_FORMAT_MAJOR),
            FixedValue::from(DIGEST_FORMAT_MINOR),
            FixedValue::from(u64::try_from(columns.year.len()).unwrap_or(u64::MAX)),
            FixedValue::from(meta.base_unix_seconds),
            FixedValue::from(i64::from(meta.first_offset.seconds())),
            FixedValue::from(u64::from(asked.first.calendar.id())),
            FixedValue::from(asked.place.latitude.get()),
            FixedValue::from(asked.place.longitude.get()),
            FixedValue::from(asked.place.altitude.get()),
        ],
    )?;
    writer.columns(
        "days",
        columns.year.len(),
        &[
            ColumnData::I32(&columns.year),
            ColumnData::U8(&columns.month),
            ColumnData::U8(&columns.day),
            ColumnData::U8(&columns.vara),
            ColumnData::U8(&columns.state),
            ColumnData::I32(&columns.fixed_day),
            ColumnData::I32(&columns.sunrise),
            ColumnData::I32(&columns.sunset),
            ColumnData::I32(&columns.moonrise),
            ColumnData::I32(&columns.moonset),
            ColumnData::U16(&columns.moon_phase),
            ColumnData::U16(&columns.lunar_month),
            ColumnData::U32(&columns.span_first),
            ColumnData::U16(&columns.span_count),
            ColumnData::U32(&columns.observance_first),
            ColumnData::U16(&columns.observance_count),
        ],
    )?;
    writer.columns(
        "spans",
        spans.kind.len(),
        &[
            ColumnData::U8(&spans.kind),
            ColumnData::U16(&spans.name),
            ColumnData::I32(&spans.from),
            ColumnData::I32(&spans.to),
        ],
    )?;
    writer.columns(
        "observances",
        observance_name.len(),
        &[
            ColumnData::U16(observance_name),
            ColumnData::U8(observance_flags),
        ],
    )?;
    writer.columns(
        "names",
        names.key_at.len(),
        &[
            ColumnData::U32(&names.key_at),
            ColumnData::U16(&names.key_length),
            ColumnData::U32(&names.text_at),
            ColumnData::U16(&names.text_length),
        ],
    )?;
    writer.bytes("text", &names.text)?;
    writer.bytes("locale", meta.locale.as_bytes())?;
    writer.bytes(
        "provenance_json",
        canonical_json(meta.provenance).as_bytes(),
    )?;
    writer.columns(
        "clock",
        columns.utc_offset_seconds.len(),
        &[ColumnData::I32(&columns.utc_offset_seconds)],
    )?;
    writer.bytes("seal", &[0; SEAL_LEN])
}

/// The length of the seal: SHA-256 in hexadecimal, text, since a bindings'
/// generated decoder reads a bytes section as UTF-8.
const SEAL_LEN: usize = 64;

/// Bytes in lowercase hexadecimal.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(SEAL_LEN), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// The fixed day of 1970-01-01.
const UNIX_EPOCH_DAY: i64 = 719_163;
