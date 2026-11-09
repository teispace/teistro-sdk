//! The observance finder (`03-design/observance-finder.md`, A2-1): when one
//! observance falls in a year or a window under each named tradition, the
//! occurrences paired across them, and where they part, what differs and
//! why (P1-10). Each tradition is a festival request's rule reckoned under
//! this context or under another one; the reckoning is
//! [`AlmanacArea::festivals`], and the pairing is the one the disagreements
//! machinery uses, an occurrence being one rule over one tithi.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Calendar, Masa};
use teistro_core::envelope::{Envelope, Hash, ProviderStamp, content_hash};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::Place;
use teistro_panchanga::festival::{EkadashiFast, Observance, Observances};
use teistro_time::ZoneSpec;

use super::AlmanacArea;
use super::disagreement::overlap;
use crate::DayClock;
use crate::area::system_of;
use crate::asked::{DayAsked, place_of};
use crate::context::Context;
use crate::festival_request::FestivalRequest;

/// The field an observance request's refusals are named under.
const OBSERVANCE: &str = "observance";

/// The most traditions one request compares.
pub const MOST_TRADITIONS: usize = 8;

/// Which context reckons a tradition: the call's own, or the other one
/// it was handed (`almanac.observance(request, other)`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ObservanceReckoning"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reckoning {
    /// The context the call is made on.
    #[default]
    This,
    /// The other context the call is handed.
    Other,
}

/// One tradition: a name for it, the festival request whose rule it
/// keeps, that rule's key, and the context that reckons it.
#[derive(Clone, Debug, PartialEq)]
pub struct Tradition {
    /// What the answer calls it.
    pub name: String,
    /// The rule's key in `festivals`.
    pub rule: String,
    /// The festival request, cut to what finding the rule's days needs.
    pub festivals: FestivalRequest,
    /// Which context reckons it.
    pub reckoning: Reckoning,
}

/// An observance finder's request: the days, the place and clock, and the
/// traditions compared.
///
/// ```
/// use teistro::finders::{ObservanceRequest, Reckoning};
///
/// let asked = ObservanceRequest::from_json(
///     r#"{"year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "traditions": [
///           {"name": "smarta", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_SMARTA"},
///           {"name": "vaishnava", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_VAISHNAVA"}]}"#,
/// )?;
/// assert_eq!((asked.first.day, asked.last.month, asked.last.day), (1, 12, 31));
/// assert_eq!(asked.traditions[1].reckoning, Reckoning::This);
/// let typo = ObservanceRequest::from_json(
///     r#"{"year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "traditions": [{"name": "a", "festivals": {"rules": "NEPAL"}, "rule": "DASHAINN"}]}"#,
/// )
/// .unwrap_err();
/// assert_eq!(typo.field(), Some("observance.traditions[0].rule"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ObservanceRequest {
    /// The first day reckoned.
    pub first: CalendarDate,
    /// The last day reckoned, inclusive.
    pub last: CalendarDate,
    /// Where the days are reckoned.
    pub place: Place,
    /// The clock its civil days run on: an offset stated, or a zone each
    /// instant is read at the offset of then.
    pub clock: DayClock,
    /// The traditions, in the order the answer lists them.
    pub traditions: Vec<Tradition>,
}

/// One tradition as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ObservanceTradition"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TraditionAsked {
    /// What the answer calls it; each name once.
    name: String,
    /// The festival request whose rule it keeps: a pack by name, or rules spelt out.
    #[cfg_attr(feature = "schema", schemars(with = "crate::festival_request::Asked"))]
    festivals: Value,
    /// The key of the rule in `festivals` whose days are found: a karmakala rule, an Ekadashi rule or one counted from another's day.
    rule: String,
    /// Which context reckons it: `THIS`, the call's own, when left out, or `OTHER`, the second context a binding passes.
    #[serde(default)]
    reckoning: Reckoning,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ObservanceRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ObservanceAsked {
    /// The calendar the year or the days are in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// A whole year of `calendar`, from its month 1, day 1; or `first`, not both.
    #[serde(default)]
    year: Option<i32>,
    /// The first day; or `year`, not both.
    #[serde(default)]
    first: Option<DayAsked>,
    /// The last day, inclusive, with `first`; the first when left out.
    #[serde(default)]
    last: Option<DayAsked>,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds, east positive;
    /// or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone each instant is read on, at its offset then; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The traditions compared, 1 to 8, in the order the answer lists them.
    traditions: Vec<TraditionAsked>,
}

impl ObservanceRequest {
    /// The record a binding sends, as JSON: `year`, or `first` and an
    /// optional `last` as `{year, month, day}`, in `calendar` (Gregorian
    /// when left out); `latitudeDeg`, `longitudeDeg`, `altitudeM` and
    /// `utcOffsetSeconds`; and `traditions`, each `{name, festivals, rule,
    /// reckoning}`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, both or neither of `year` and `first`, a place or offset out
    /// of range, no tradition or a name given twice, a festival request
    /// its reader refuses or a rule it does not carry; `LIMIT` for more
    /// than [`MOST_TRADITIONS`]; `UNSUPPORTED` for a dated rule or an
    /// anniversary, whose day no tradition's tithi decides. Each is named
    /// under `observance`.
    pub fn from_json(text: &str) -> Result<ObservanceRequest, Error> {
        let asked: ObservanceAsked = teistro_core::strict::read(text, OBSERVANCE)?;
        asked.request().map_err(|why| why.under(OBSERVANCE))
    }
}

impl ObservanceAsked {
    fn request(self) -> Result<ObservanceRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        let (first, last) = match (self.year, self.first) {
            (Some(year), None) => {
                if self.last.is_some() {
                    return Err(Error::invalid_arg(
                        "a `year` is the whole year; `last` goes with `first`",
                    )
                    .with_field("last"));
                }
                year_of(calendar, year)?
            }
            (None, Some(first)) => (
                first.in_calendar(calendar),
                self.last.unwrap_or(first).in_calendar(calendar),
            ),
            (Some(_), Some(_)) => {
                return Err(Error::invalid_arg(
                    "a request names a `year` or a `first` day, not both",
                )
                .with_field("first"));
            }
            (None, None) => {
                return Err(Error::invalid_arg(
                    "a request names the days it reckons: a `year`, or a `first` day",
                )
                .with_field("year"));
            }
        };
        Ok(ObservanceRequest {
            first,
            last,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            traditions: traditions(self.traditions)?,
        })
    }
}

/// A calendar's year, its month 1, day 1 to the day before the next's.
fn year_of(calendar: Calendar, year: i32) -> Result<(CalendarDate, CalendarDate), Error> {
    let system = system_of(calendar)?;
    let first_of = |year| system.fixed_of(&CalendarDate::defined(calendar, year, 1, 1));
    let first = first_of(year).map_err(|why| why.with_field("year"))?;
    let next = first_of(year.saturating_add(1)).map_err(|why| why.with_field("year"))?;
    Ok((system.date_of(first)?, system.date_of(next.plus_days(-1))?))
}

/// The traditions read and checked, each refused by its place.
fn traditions(asked: Vec<TraditionAsked>) -> Result<Vec<Tradition>, Error> {
    if asked.is_empty() {
        return Err(Error::invalid_arg(
            "a request compares one tradition at least; two show where they part",
        )
        .with_field("traditions"));
    }
    if asked.len() > MOST_TRADITIONS {
        return Err(Error::limit(format!(
            "a request compares at most {MOST_TRADITIONS} traditions, not {}",
            asked.len()
        ))
        .with_field("traditions"));
    }
    let mut read: Vec<Tradition> = Vec::with_capacity(asked.len());
    for (at, one) in asked.into_iter().enumerate() {
        let place = format!("traditions[{at}]");
        if read.iter().any(|held| held.name == one.name) {
            return Err(Error::invalid_arg(format!(
                "the tradition `{}` is named twice; each name is one column of the answer",
                one.name
            ))
            .with_field(format!("{place}.name")));
        }
        let festivals = FestivalRequest::from_json(&one.festivals.to_string())
            .map_err(|why| why.under(&place))?;
        let Some(narrowed) = festivals.narrowed_to(&one.rule) else {
            return Err(Error::invalid_arg(format!(
                "`{}` is no rule of the tradition's festivals",
                one.rule
            ))
            .with_field(format!("{place}.rule"))
            .with_hint("name a key of the request's rules, as `almanac.festivalPack` lists them"));
        };
        let reckoned = narrowed.rules().iter().any(|rule| rule.key == one.rule)
            || narrowed.ekadashis().iter().any(|rule| rule.key == one.rule)
            || narrowed.following().iter().any(|rule| rule.key == one.rule);
        if !reckoned {
            return Err(Error::unsupported(format!(
                "`{}` is a dated day or an anniversary, which no tradition's tithi decides",
                one.rule
            ))
            .with_field(format!("{place}.rule"))
            .with_hint("ask `almanac.festivals` for its days"));
        }
        read.push(Tradition {
            name: one.name,
            rule: one.rule,
            festivals: narrowed,
            reckoning: one.reckoning,
        });
    }
    Ok(read)
}

/// A tradition as the answer names it: its name, its rule and the
/// reckoning that kept it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TraditionReckoned {
    /// Its name.
    pub name: String,
    /// The rule's key.
    pub rule: String,
    /// Which context reckoned it.
    pub reckoning: Reckoning,
    /// The profile that context resolved.
    pub profile: String,
    /// That context's settings hash.
    pub settings_hash: Hash,
    /// The ephemeris that reckoned it, as the festival answer stamps it:
    /// two contexts of one settings hash over two ephemerides are two
    /// skies.
    pub provider: ProviderStamp,
}

/// One tradition's day for an occurrence, or none.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Kept {
    /// The tradition's name.
    pub tradition: String,
    /// The day; absent when the tradition keeps no day for it in the range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<CalendarDate>,
    /// The observance, as a festival answer holds it, for a karmakala
    /// rule: the tithi's extent on the two days and what decided between
    /// them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observance: Option<Observance>,
    /// The fast, for an Ekadashi rule: the vedha, the excess and the
    /// table's choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast: Option<EkadashiFast>,
}

/// What differs between the traditions' days for one occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Difference {
    /// Some tradition keeps no day for it in the range.
    Absent,
    /// The traditions keep it on different days.
    Day,
    /// They file it under a different month, or one under an adhika one.
    Month,
}

/// Why the traditions part on an occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Parting {
    /// One sky, different rules: each side's record says what its rule
    /// decided (an observance's `decidedBy` and `choice`, a fast's vedha,
    /// excess and `choice`).
    Rule,
    /// One rule, different skies: the tithi's edges fall elsewhere, and
    /// each side's extents say where (P1-10).
    Sky,
    /// Different rules under different skies.
    RuleAndSky,
    /// A published entry set or withdrew a side's day.
    Published,
}

/// One occurrence, paired across the traditions.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    /// Its amanta month, as the first tradition keeping it names it.
    pub month: Masa,
    /// Whether that month is adhika.
    pub adhika: bool,
    /// Each tradition's day, in the request's order.
    pub kept: Vec<Kept>,
    /// Whether every tradition keeps it, on one day, under one month.
    pub agree: bool,
    /// What differs, when they do not agree.
    pub differs: Vec<Difference>,
    /// Why they part, when their days or months differ; absent when they
    /// agree or only a side's absence differs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Parting>,
}

/// An occurrence a tradition could not give a day, and why.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UnjudgedIn {
    /// The tradition's name.
    pub tradition: String,
    /// The occurrence: the tithi's, or the nakshatra's.
    pub tithi: Interval,
    /// Why.
    pub why: String,
}

/// What an observance finder answers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ObservanceFinding {
    /// The traditions, in the request's order.
    pub traditions: Vec<TraditionReckoned>,
    /// Every occurrence any of them keeps, in the order of its tithi.
    pub occurrences: Vec<Occurrence>,
    /// How many occurrences the traditions part on.
    pub parted: usize,
    /// The occurrences a tradition could give no day.
    pub unjudged: Vec<UnjudgedIn>,
}

/// One tradition's entry for an occurrence, before pairing.
#[derive(Clone, Debug)]
struct Entry {
    /// What it pairs on: the tithi (or the 11th, for a fast).
    anchor: Interval,
    day: CalendarDate,
    month: Masa,
    adhika: bool,
    published: bool,
    observance: Option<Observance>,
    fast: Option<EkadashiFast>,
}

/// A tradition's entries for its rule, in the order of their tithis.
fn entries(found: Observances, rule: &str) -> Vec<Entry> {
    let observances = found
        .observances
        .into_iter()
        .filter(|kept| kept.rule == rule)
        .map(|kept| Entry {
            anchor: kept.tithi,
            day: kept.day.clone(),
            month: kept.month,
            adhika: kept.adhika,
            published: kept.published.is_some(),
            observance: Some(kept),
            fast: None,
        });
    let fasts = found
        .ekadashis
        .into_iter()
        .filter(|kept| kept.rule == rule)
        .map(|kept| Entry {
            anchor: kept.tithis[1],
            day: kept.day.clone(),
            month: kept.month,
            adhika: kept.adhika,
            published: kept.published.is_some(),
            observance: None,
            fast: Some(kept),
        });
    let mut all: Vec<Entry> = observances.chain(fasts).collect();
    all.sort_by(|a, b| a.anchor.from.get().total_cmp(&b.anchor.from.get()));
    all
}

/// The entries of every tradition, paired: one row an occurrence, a
/// column a tradition, an entry joining the first row it overlaps that
/// its tradition has not filled. One occurrence is one rule over one
/// tithi whatever day each sky gives it, as the disagreements pair them.
fn paired(columns: Vec<Vec<Entry>>) -> Vec<Vec<Option<Entry>>> {
    let width = columns.len();
    let mut rows: Vec<Vec<Option<Entry>>> = Vec::new();
    for (column, entries) in columns.into_iter().enumerate() {
        for entry in entries {
            let joined = rows.iter().position(|row| {
                row.get(column).is_some_and(Option::is_none)
                    && row
                        .iter()
                        .flatten()
                        .any(|held| overlap(held.anchor, entry.anchor))
            });
            let at = joined.unwrap_or_else(|| {
                rows.push(vec![None; width]);
                rows.len().saturating_sub(1)
            });
            if let Some(cell) = rows.get_mut(at).and_then(|row| row.get_mut(column)) {
                *cell = Some(entry);
            }
        }
    }
    let start = |row: &[Option<Entry>]| {
        row.iter()
            .flatten()
            .map(|entry| entry.anchor.from.get())
            .fold(f64::INFINITY, f64::min)
    };
    rows.sort_by(|a, b| start(a).total_cmp(&start(b)));
    rows
}

/// What differs on a row, and why, given whether the traditions share one
/// sky and one rule.
fn judged(
    row: &[Option<Entry>],
    one_sky: bool,
    one_rule: bool,
) -> (Vec<Difference>, Option<Parting>) {
    let present: Vec<&Entry> = row.iter().flatten().collect();
    let mut differs = Vec::new();
    if present.len() < row.len() {
        differs.push(Difference::Absent);
    }
    let first = present.first();
    let day = present
        .iter()
        .any(|entry| first.is_some_and(|held| held.day != entry.day));
    if day {
        differs.push(Difference::Day);
    }
    let month = present.iter().any(|entry| {
        first.is_some_and(|held| held.month != entry.month || held.adhika != entry.adhika)
    });
    if month {
        differs.push(Difference::Month);
    }
    let reason = if !(day || month) {
        None
    } else if present.iter().any(|entry| entry.published) {
        Some(Parting::Published)
    } else {
        match (one_sky, one_rule) {
            (true, false) => Some(Parting::Rule),
            (false, true) => Some(Parting::Sky),
            (false, false) => Some(Parting::RuleAndSky),
            // One rule under one sky is one answer, and cannot part.
            (true, true) => None,
        }
    };
    (differs, reason)
}

/// Whether every tradition is reckoned under one sky (one reckoning, or
/// equal settings hashes over one ephemeris) and under one rule (one key
/// of one festival request).
fn shared(request: &ObservanceRequest, skies: &[(Hash, &ProviderStamp)]) -> (bool, bool) {
    let first = request.traditions.first();
    let one_sky = request
        .traditions
        .iter()
        .all(|one| first.is_some_and(|first| first.reckoning == one.reckoning))
        || skies.windows(2).all(|pair| pair.first() == pair.last());
    let one_rule = request.traditions.iter().all(|one| {
        first.is_some_and(|first| first.rule == one.rule && first.festivals == one.festivals)
    });
    (one_sky, one_rule)
}

/// The paired rows as occurrences: each tradition's day in the request's
/// order, and what differs and why.
fn occurrences(
    rows: Vec<Vec<Option<Entry>>>,
    request: &ObservanceRequest,
    one_sky: bool,
    one_rule: bool,
) -> Vec<Occurrence> {
    let names: Vec<&str> = request
        .traditions
        .iter()
        .map(|one| one.name.as_str())
        .collect();
    rows.into_iter()
        .filter_map(|row| {
            let (differs, reason) = judged(&row, one_sky, one_rule);
            let first = row.iter().flatten().next()?;
            let (month, adhika) = (first.month, first.adhika);
            let kept = row
                .into_iter()
                .zip(&names)
                .map(|(entry, name)| match entry {
                    Some(entry) => Kept {
                        tradition: (*name).to_owned(),
                        day: Some(entry.day),
                        observance: entry.observance,
                        fast: entry.fast,
                    },
                    None => Kept {
                        tradition: (*name).to_owned(),
                        day: None,
                        observance: None,
                        fast: None,
                    },
                })
                .collect();
            Some(Occurrence {
                month,
                adhika,
                kept,
                agree: differs.is_empty(),
                differs,
                reason,
            })
        })
        .collect()
}

impl AlmanacArea<'_> {
    /// **When an observance falls under each tradition**: each tradition's
    /// rule reckoned over the request's days, the occurrences paired
    /// across them, and on each where they part, what differs and why
    /// (`03-design/observance-finder.md`). A tradition reckoned `OTHER` is
    /// reckoned under `other`, so two reckonings of one day (the official
    /// and the modern, P1-10) are two traditions of one rule.
    ///
    /// ```no_run
    /// use teistro::Context;
    /// use teistro::finders::ObservanceRequest;
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = ObservanceRequest::from_json(
    ///     r#"{"year": 2026, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "traditions": [
    ///           {"name": "smarta", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_SMARTA"},
    ///           {"name": "vaishnava", "festivals": {"rules": "DHARMASINDHU"}, "rule": "EKADASHI_VAISHNAVA"}]}"#,
    /// )?;
    /// let found = sdk.almanac().observance(&asked, None)?.value;
    /// for parted in found.occurrences.iter().filter(|o| !o.agree) {
    ///     println!("{:?}: {:?} ({:?})", parted.month, parted.differs, parted.reason);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `traditions[i].reckoning` for a tradition
    /// reckoned `OTHER` with no other context; and as
    /// [`AlmanacArea::festivals`], under either context.
    pub fn observance(
        self,
        request: &ObservanceRequest,
        other: Option<&Context>,
    ) -> Result<Envelope<ObservanceFinding>, Error> {
        let mut columns = Vec::with_capacity(request.traditions.len());
        let mut reckoned = Vec::with_capacity(request.traditions.len());
        let mut unjudged = Vec::new();
        let mut sealed = None;
        for (at, tradition) in request.traditions.iter().enumerate() {
            let context = match tradition.reckoning {
                Reckoning::This => self.context,
                Reckoning::Other => other.ok_or_else(|| {
                    Error::invalid_arg(
                        "the tradition is reckoned under the other context, and none was passed",
                    )
                    .with_field(format!("observance.traditions[{at}].reckoning"))
                    .with_hint("pass the second context beside the request, or reckon it `THIS`")
                })?,
            };
            let area = if core::ptr::eq(context, self.context) {
                self
            } else {
                context.almanac()
            };
            let Envelope { value, provenance } = area.unclocked().clocked_from(
                &request.clock,
                &request.first,
                |almanac, offset| {
                    almanac.festivals(
                        &request.first,
                        &request.last,
                        &request.place,
                        offset,
                        &tradition.festivals,
                    )
                },
            )?;
            unjudged.extend(
                value
                    .unjudged
                    .iter()
                    .filter(|missed| missed.rule == tradition.rule)
                    .map(|missed| UnjudgedIn {
                        tradition: tradition.name.clone(),
                        tithi: missed.tithi,
                        why: missed.why.clone(),
                    }),
            );
            columns.push(entries(value, &tradition.rule));
            reckoned.push(TraditionReckoned {
                name: tradition.name.clone(),
                rule: tradition.rule.clone(),
                reckoning: tradition.reckoning,
                profile: context.profile().to_owned(),
                settings_hash: context.settings_hash(),
                provider: provenance.provider.clone(),
            });
            sealed.get_or_insert(provenance);
        }
        let skies: Vec<(Hash, &ProviderStamp)> = reckoned
            .iter()
            .map(|one| (one.settings_hash, &one.provider))
            .collect();
        let (one_sky, one_rule) = shared(request, &skies);
        let occurrences = occurrences(paired(columns), request, one_sky, one_rule);
        let Some(mut provenance) = sealed else {
            return Err(Error::internal(
                "a checked request names a tradition, so one was reckoned",
            ));
        };
        let mut input = serde_json::json!({
            "first": request.first.to_string(),
            "last": request.last.to_string(),
            "place": request.place,
            "traditions": request.traditions.iter().map(|one| serde_json::json!({
                "name": one.name,
                "rule": one.rule,
                "reckoning": one.reckoning,
                "festivals": one.festivals,
            })).collect::<Vec<_>>(),
            "skies": skies,
        });
        request.clock.seal_into(&mut input);
        provenance.input_hash = content_hash(&input);
        let parted = occurrences.iter().filter(|one| !one.agree).count();
        Ok(Envelope::sealing(
            ObservanceFinding {
                traditions: reckoned,
                occurrences,
                parted,
                unjudged,
            },
            provenance,
        ))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn entry(day: u8, from: f64, month: Masa, published: bool) -> Entry {
        Entry {
            anchor: Interval::literal(2_460_000.0 + from, 2_460_001.0 + from),
            day: CalendarDate::defined(Calendar::Gregorian, 2026, 1, day),
            month,
            adhika: false,
            published,
            observance: None,
            fast: None,
        }
    }

    #[test]
    fn entries_pair_by_their_tithi_and_a_column_fills_once() {
        let rows = paired(vec![
            vec![
                entry(1, 0.0, Masa::Magha, false),
                entry(15, 14.0, Masa::Magha, false),
            ],
            vec![
                entry(2, 0.3, Masa::Magha, false),
                entry(3, 0.6, Masa::Magha, false),
            ],
        ]);
        // The second column's two overlapping entries take two rows: the
        // first pairs with the first column's, the second stands alone.
        assert_eq!(rows.len(), 3);
        assert!(rows[0].iter().all(Option::is_some));
        assert_eq!(
            rows.iter()
                .map(|row| row.iter().flatten().count())
                .sum::<usize>(),
            4
        );
        assert!(rows[2][1].is_none() && rows[2][0].is_some());
    }

    #[test]
    fn a_parting_is_named_by_what_the_traditions_share() {
        let same = [
            Some(entry(1, 0.0, Masa::Magha, false)),
            Some(entry(1, 0.1, Masa::Magha, false)),
        ];
        assert_eq!(judged(&same, true, false), (Vec::new(), None));
        let moved = [
            Some(entry(1, 0.0, Masa::Magha, false)),
            Some(entry(2, 0.1, Masa::Magha, false)),
        ];
        assert_eq!(
            judged(&moved, true, false),
            (vec![Difference::Day], Some(Parting::Rule))
        );
        assert_eq!(judged(&moved, false, true).1, Some(Parting::Sky));
        assert_eq!(judged(&moved, false, false).1, Some(Parting::RuleAndSky));
        assert_eq!(judged(&moved, true, true).1, None);
        let filed = [
            Some(entry(1, 0.0, Masa::Magha, false)),
            Some(entry(1, 0.1, Masa::Phalguna, true)),
        ];
        assert_eq!(
            judged(&filed, false, true),
            (vec![Difference::Month], Some(Parting::Published))
        );
        let alone = [Some(entry(1, 0.0, Masa::Magha, false)), None];
        assert_eq!(
            judged(&alone, true, false),
            (vec![Difference::Absent], None)
        );
    }

    #[test]
    fn a_request_is_read_and_refused_by_its_place() {
        let base = r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700"#;
        let tradition = r#"{"name": "a", "festivals": {"rules": "NEPAL_PANCHANGA"}, "rule": "HARIBODHINI_EKADASHI"}"#;
        let read = |head: &str, list: &str| {
            ObservanceRequest::from_json(&format!(r#"{{{head} {base}, "traditions": [{list}]}}"#))
        };
        let window = read(
            r#""first": {"year": 2026, "month": 10, "day": 1},"#,
            tradition,
        )
        .unwrap();
        assert_eq!(window.first, window.last);
        let field =
            |head: &str, list: &str| read(head, list).unwrap_err().field().map(str::to_owned);
        let year = r#""year": 2026,"#;
        assert_eq!(field("", tradition).as_deref(), Some("observance.year"));
        assert_eq!(
            field(
                r#""year": 2026, "first": {"year": 2026, "month": 1, "day": 1},"#,
                tradition
            )
            .as_deref(),
            Some("observance.first")
        );
        assert_eq!(
            field(
                r#""year": 2026, "last": {"year": 2026, "month": 1, "day": 1},"#,
                tradition
            )
            .as_deref(),
            Some("observance.last")
        );
        assert_eq!(field(year, "").as_deref(), Some("observance.traditions"));
        let twice = format!("{tradition}, {tradition}");
        assert_eq!(
            field(year, &twice).as_deref(),
            Some("observance.traditions[1].name")
        );
        let many = [tradition; MOST_TRADITIONS + 1].join(", ");
        let refused = read(year, &many).unwrap_err();
        assert_eq!(refused.status, teistro_core::error::Status::Limit);
        let wrong =
            r#"{"name": "a", "festivals": {"rules": "NEPALL"}, "rule": "HARIBODHINI_EKADASHI"}"#;
        assert_eq!(
            field(year, wrong).as_deref(),
            Some("observance.traditions[0].festivals.rules")
        );
    }
}
