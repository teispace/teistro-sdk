//! Days an authority decides, laid over the rules' as data
//! (`03-design/published-overrides.md`): a [`PublishedSet`]'s entries set,
//! add or withdraw a day, and every day they touch says so.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use teistro_calendar::{CalendarDate, CalendarSystem, EraNumber, FixedDay, Leap, shipped};
use teistro_core::catalogue::{Calendar, Masa};
use teistro_core::envelope::{Hash, content_hash};
use teistro_core::error::Error;
use teistro_core::interval::Interval;

use super::{Observances, Parana, VrataParanaTime};

/// The format a set names, and the one this SDK reads.
pub const PUBLISHED_FORMAT: &str = "teistro.published/1";

/// How far a `SET` or `WITHDRAW` entry's day may lie from the occurrence
/// it is placed on, in days: a published day a week from the rule's is
/// another occurrence, not a disagreement about this one.
pub const NEAREST_DAYS: i64 = 7;

/// A publisher's days for a span, read from JSON or built in code.
///
/// ```
/// use teistro_panchanga::festival::PublishedSet;
///
/// let set = PublishedSet::from_json(r#"{
///     "format": "teistro.published/1",
///     "publisher": "a patro printed in Nepal",
///     "id": "np-2084",
///     "version": 1,
///     "entries": [{
///         "action": "SET",
///         "rule": "GHATASTHAPANA",
///         "day": {"calendar": "BIKRAM_SAMBAT", "year": 2084, "month": 6, "day": 14},
///         "source": "p. 41"
///     }]
/// }"#)?;
/// assert_eq!(set.entries.len(), 1);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublishedSet {
    /// [`PUBLISHED_FORMAT`].
    pub format: String,
    /// Who published it, as an answer names them.
    pub publisher: String,
    /// The set's own name (`np-panchanga-2084`).
    pub id: String,
    /// Its version; a correction is a higher one.
    pub version: u32,
    /// Its days.
    pub entries: Vec<PublishedEntry>,
}

/// A day as a publisher prints it, in any calendar the SDK ships: the
/// answer gives it in the range's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublishedDay {
    /// Its calendar.
    pub calendar: Calendar,
    /// The year.
    pub year: i32,
    /// The month, 1-based.
    pub month: u8,
    /// The day, 1-based.
    pub day: u8,
    /// The era, for a calendar that counts several.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub era: Option<EraNumber>,
    /// A leap month or day, for a calendar that marks one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<Leap>,
}

impl PublishedDay {
    /// The day as the calendar defines it.
    #[must_use]
    pub fn date(&self) -> CalendarDate {
        let mut date = CalendarDate::defined(self.calendar, self.year, self.month, self.day);
        date.era = self.era;
        date.leap = self.leap;
        date
    }

    fn fixed(&self) -> Result<FixedDay, Error> {
        system(self.calendar)?.fixed_of(&self.date())
    }
}

impl From<&CalendarDate> for PublishedDay {
    fn from(date: &CalendarDate) -> PublishedDay {
        PublishedDay {
            calendar: date.calendar,
            year: date.year,
            month: date.month,
            day: date.day,
            era: date.era,
            leap: date.leap,
        }
    }
}

/// One published day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishedEntry {
    /// The day of a rule's occurrence: the one nearest `day`, within
    /// [`NEAREST_DAYS`].
    Set {
        /// The rule's key.
        rule: String,
        /// The published day.
        day: PublishedDay,
        /// Which occurrence, when two lie that near: its amanta month.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        month: Option<Masa>,
        /// And whether that month is adhika.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        adhika: Option<bool>,
        /// A published time (a parana, a sait).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<Interval>,
        /// Where the publisher says it.
        source: String,
    },
    /// A day no rule computes, under its own key.
    Add {
        /// Its key, which no rule of the request uses.
        key: String,
        /// The day.
        day: PublishedDay,
        /// Its name in each locale given (`en-Latn`, `ne-Deva-NP`).
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        names: BTreeMap<String, String>,
        /// A published time.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<Interval>,
        /// Where the publisher says it.
        source: String,
    },
    /// A rule's occurrence the publisher does not keep, found as `SET`
    /// finds one.
    Withdraw {
        /// The rule's key.
        rule: String,
        /// The occurrence's day, as the publisher reckons it.
        day: PublishedDay,
        /// As `SET`'s.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        month: Option<Masa>,
        /// As `SET`'s.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        adhika: Option<bool>,
        /// Where the publisher says it.
        source: String,
    },
}

impl PublishedEntry {
    /// The day it names.
    #[must_use]
    pub fn day(&self) -> &PublishedDay {
        match self {
            PublishedEntry::Set { day, .. }
            | PublishedEntry::Add { day, .. }
            | PublishedEntry::Withdraw { day, .. } => day,
        }
    }

    /// The rule a `SET` or `WITHDRAW` names, or the key an `ADD` gives.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            PublishedEntry::Set { rule, .. } | PublishedEntry::Withdraw { rule, .. } => rule,
            PublishedEntry::Add { key, .. } => key,
        }
    }

    fn month(&self) -> (Option<Masa>, Option<bool>) {
        match self {
            PublishedEntry::Set { month, adhika, .. }
            | PublishedEntry::Withdraw { month, adhika, .. } => (*month, *adhika),
            PublishedEntry::Add { .. } => (None, None),
        }
    }
}

/// Whether a published entry kept a day or withdrew it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishedStatus {
    /// The publisher keeps it on the answer's day.
    Published,
    /// The publisher does not keep it.
    Withdrawn,
}

/// What a published entry did to a day; absent from a day no entry
/// touched, which is the rule's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Published {
    /// Kept or withdrawn.
    pub status: PublishedStatus,
    /// The set's publisher.
    pub publisher: String,
    /// The set's id.
    pub set: String,
    /// Its version.
    pub version: u32,
    /// Where the publisher says it.
    pub source: String,
    /// The rule's own day, when the publisher's differs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed: Option<CalendarDate>,
    /// A published time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<Interval>,
}

/// A day an `ADD` entry gave.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AddedDay {
    /// Its key.
    pub key: String,
    /// The day, in the range's calendar.
    pub day: CalendarDate,
    /// Its names, by locale.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub names: BTreeMap<String, String>,
    /// Who published it, and where.
    pub published: Published,
}

/// An entry inside the range that no occurrence took.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Unplaced {
    /// The set's id.
    pub set: String,
    /// The entry's index in it.
    pub entry: usize,
    /// The rule it names.
    pub rule: String,
    /// The day it names, in the range's calendar.
    pub day: CalendarDate,
}

/// A set an answer applied, so a stored answer says which data made it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AppliedSet {
    /// Its id.
    pub id: String,
    /// Its version.
    pub version: u32,
    /// Its publisher.
    pub publisher: String,
    /// The SHA-256 of its canonical JSON.
    pub hash: Hash,
}

impl PublishedSet {
    /// A set read strictly from JSON and checked.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the field that is not JSON of its kind or not
    /// a field, or as [`PublishedSet::check`].
    pub fn from_json(text: &str) -> Result<PublishedSet, Error> {
        let set: PublishedSet = teistro_core::strict::read(text, "published")?;
        set.check()?;
        Ok(set)
    }

    /// A set read from a signed pack: its exact bytes, the publisher's
    /// detached Ed25519 signature over them and the publisher's key, which
    /// the caller fixes at its own build (the SDK embeds none). The
    /// signature is checked strictly (`verify_strict`, which refuses
    /// small-order and non-canonical points) before a byte is read, and a
    /// version under `at_least`, the newest the caller has applied, is
    /// refused, so a replayed pack cannot roll a correction back
    /// (`03-design/published-overrides.md` §5).
    ///
    /// ```
    /// use teistro_panchanga::festival::PublishedSet;
    ///
    /// // A key whose y-coordinate, 2, has no curve point is refused before
    /// // any signature is checked.
    /// let mut key = [0_u8; 32];
    /// key[0] = 2;
    /// let refused = PublishedSet::from_signed(b"{}", &[0; 64], &key, 0).unwrap_err();
    /// assert_eq!(refused.field(), Some("key"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `key` for one that is not 32 bytes of a curve
    /// point, on `signature` for one that is not 64 bytes or does not
    /// verify (the pack was altered, or signed by another key), on `pack`
    /// for bytes that are not UTF-8, on `version` under `at_least`, or as
    /// [`PublishedSet::from_json`].
    #[cfg(feature = "signed-packs")]
    pub fn from_signed(
        pack: &[u8],
        signature: &[u8],
        key: &[u8],
        at_least: u32,
    ) -> Result<PublishedSet, Error> {
        let key = <[u8; ed25519_dalek::PUBLIC_KEY_LENGTH]>::try_from(key)
            .ok()
            .and_then(|bytes| ed25519_dalek::VerifyingKey::from_bytes(&bytes).ok())
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "the publisher's key is {} bytes of an Ed25519 point",
                    ed25519_dalek::PUBLIC_KEY_LENGTH
                ))
                .with_field("key")
            })?;
        let signature = <[u8; ed25519_dalek::SIGNATURE_LENGTH]>::try_from(signature)
            .map(|bytes| ed25519_dalek::Signature::from_bytes(&bytes))
            .map_err(|_| {
                Error::invalid_arg(format!(
                    "a detached Ed25519 signature is {} bytes, not {}",
                    ed25519_dalek::SIGNATURE_LENGTH,
                    signature.len()
                ))
                .with_field("signature")
            })?;
        key.verify_strict(pack, &signature).map_err(|_| {
            Error::invalid_arg(
                "the signature does not verify: the pack was altered, or signed by another key",
            )
            .with_field("signature")
        })?;
        let text = core::str::from_utf8(pack).map_err(|e| {
            Error::invalid_arg(format!("a pack is UTF-8 JSON: {e}")).with_field("pack")
        })?;
        let set = PublishedSet::from_json(text)?;
        if set.version < at_least {
            return Err(Error::invalid_arg(format!(
                "version {} is older than {at_least}, the newest applied; a replayed pack cannot roll a correction back",
                set.version
            ))
            .with_field("version"));
        }
        Ok(set)
    }

    /// Refuses a set this SDK cannot read, a day it cannot convert, and a
    /// second entry for one occurrence.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `format`, or on `entries[i]` and its field.
    pub fn check(&self) -> Result<(), Error> {
        if self.format != PUBLISHED_FORMAT {
            return Err(Error::invalid_arg(format!(
                "`{}` is not a format this SDK reads; it reads `{PUBLISHED_FORMAT}`",
                self.format
            ))
            .with_field("format"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let at = format!("entries[{index}]");
            let fixed = entry
                .day()
                .fixed()
                .map_err(|why| invalid(why).with_field(format!("{at}.day")))?;
            let (month, adhika) = entry.month();
            if !seen.insert((entry.key(), fixed, month, adhika)) {
                return Err(Error::invalid_arg(format!(
                    "`{}` on {} is given twice; a set publishes an occurrence once",
                    entry.key(),
                    entry.day().date()
                ))
                .with_field(at));
            }
        }
        Ok(())
    }

    /// Its record in an answer.
    #[must_use]
    pub fn applied(&self) -> AppliedSet {
        AppliedSet {
            id: self.id.clone(),
            version: self.version,
            publisher: self.publisher.clone(),
            hash: content_hash(self),
        }
    }

    fn published(&self, status: PublishedStatus, source: &str) -> Published {
        Published {
            status,
            publisher: self.publisher.clone(),
            set: self.id.clone(),
            version: self.version,
            source: source.to_owned(),
            computed: None,
            window: None,
        }
    }
}

/// A date's refusal as the caller's argument: a set's day is input.
fn invalid(why: Error) -> Error {
    Error::invalid_arg(why.message)
}

fn system(calendar: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(calendar).ok_or_else(|| {
        Error::invalid_arg(format!(
            "the SDK does not ship the `{}` calendar",
            calendar.key()
        ))
    })
}

/// One occurrence an entry can be placed on (a karmakala observance, an
/// Ekadashi fast or a dated observance), seen through what placing needs.
struct Occurrence<'a> {
    rule: &'a str,
    day: &'a mut CalendarDate,
    month: Option<(Masa, bool)>,
    published: &'a mut Option<Published>,
    /// An Ekadashi fast's parana, which a moved fast cannot keep.
    fast_parana: Option<&'a mut Option<Parana>>,
    /// A vrata's paranas, which a moved observance cannot keep either.
    vrata_parana: Option<&'a mut Vec<VrataParanaTime>>,
}

/// Lays `sets` over an answer for the days `from` to `to` inclusive, in
/// order, a later set's entry replacing an earlier one's for the same
/// occurrence. Every day it writes is in `calendar`, the range's.
///
/// The answer is laid over before it is trimmed to the range, so a day
/// published across the range's edge moves in or out of it with its
/// occurrence. A `SET` or `WITHDRAW` within [`NEAREST_DAYS`] of the range
/// is placed on its rule's occurrence nearest its day, within those days
/// and in the month it names; one inside the range that no occurrence
/// takes is listed in `unplaced`. An `ADD` is taken inside the range. A
/// moved observance keeps no parana but the published window: the
/// computed one was the computed day's.
///
/// # Errors
///
/// `INVALID_ARG` on `published[i].entries[j]` for an entry two
/// occurrences fit, or a day the SDK cannot convert.
pub fn apply(
    answer: &mut Observances,
    sets: &[PublishedSet],
    calendar: Calendar,
    (from, to): (FixedDay, FixedDay),
) -> Result<(), Error> {
    let range = Range {
        system: system(calendar)?,
        from,
        to,
    };
    for (index, set) in sets.iter().enumerate() {
        apply_one(answer, set, &range).map_err(|why| why.under(&format!("published[{index}]")))?;
        answer.applied.push(set.applied());
    }
    Ok(())
}

/// The range an answer covers, and its calendar.
struct Range {
    system: &'static dyn CalendarSystem,
    from: FixedDay,
    to: FixedDay,
}

impl Range {
    fn inside(&self, day: FixedDay) -> bool {
        day >= self.from && day <= self.to
    }

    fn near(&self, day: FixedDay) -> bool {
        day >= self.from.plus_days(-NEAREST_DAYS) && day <= self.to.plus_days(NEAREST_DAYS)
    }
}

fn apply_one(answer: &mut Observances, set: &PublishedSet, range: &Range) -> Result<(), Error> {
    for (index, entry) in set.entries.iter().enumerate() {
        let at = format!("entries[{index}]");
        let day = entry
            .day()
            .fixed()
            .map_err(|why| invalid(why).with_field(format!("{at}.day")))?;
        let date = range.system.date_of(day)?;
        let PublishedEntry::Add {
            key,
            names,
            window,
            source,
            ..
        } = entry
        else {
            if range.near(day) {
                place(answer, set, (index, entry), (day, date))
                    .map_err(|why| why.with_field(at))?;
            }
            continue;
        };
        if !range.inside(day) {
            continue;
        }
        let mut published = set.published(PublishedStatus::Published, source);
        published.window = *window;
        answer
            .added
            .retain(|added| added.key != *key || added.day != date);
        answer.added.push(AddedDay {
            key: key.clone(),
            day: date,
            names: names.clone(),
            published,
        });
    }
    let inside = |unplaced: &Unplaced| {
        range
            .system
            .fixed_of(&unplaced.day)
            .is_ok_and(|day| range.inside(day))
    };
    answer.unplaced.retain(inside);
    Ok(())
}

/// Places a `SET` or `WITHDRAW` on its occurrence, `day` being the
/// entry's day as a fixed day and as a date in the range's calendar.
fn place(
    answer: &mut Observances,
    set: &PublishedSet,
    (index, entry): (usize, &PublishedEntry),
    (day, date): (FixedDay, CalendarDate),
) -> Result<(), Error> {
    let rule = entry.key();
    let (month, adhika) = entry.month();
    let mut fits = Vec::new();
    for occurrence in occurrences(answer) {
        if occurrence.rule != rule {
            continue;
        }
        // The rule's own day, which an earlier set may have moved.
        let own = occurrence
            .published
            .as_ref()
            .and_then(|published| published.computed.clone())
            .unwrap_or_else(|| occurrence.day.clone());
        let own_fixed = system(own.calendar)?.fixed_of(&own)?;
        let in_month = match occurrence.month {
            Some((masa, is_adhika)) => {
                month.is_none_or(|want| want == masa) && adhika.is_none_or(|want| want == is_adhika)
            }
            None => month.is_none() && adhika.is_none(),
        };
        if (own_fixed.get() - day.get()).abs() <= NEAREST_DAYS && in_month {
            fits.push((occurrence, own, own_fixed));
        }
    }
    if fits.len() > 1 {
        return Err(Error::invalid_arg(format!(
            "`{rule}` on {date} fits {} occurrences within {NEAREST_DAYS} days; name the one meant by its `month` and `adhika`",
            fits.len()
        )));
    }
    let Some((occurrence, own, own_fixed)) = fits.pop() else {
        // Kept until the set is done, then trimmed to the range.
        answer.unplaced.push(Unplaced {
            set: set.id.clone(),
            entry: index,
            rule: rule.to_owned(),
            day: date,
        });
        return Ok(());
    };
    let (status, window, source) = match entry {
        PublishedEntry::Set { window, source, .. } => (PublishedStatus::Published, *window, source),
        PublishedEntry::Withdraw { source, .. } => (PublishedStatus::Withdrawn, None, source),
        PublishedEntry::Add { .. } => return Ok(()),
    };
    let mut published = set.published(status, source);
    published.window = window;
    // A withdrawal leaves the occurrence on the rule's day.
    if status == PublishedStatus::Published && own_fixed != day {
        published.computed = Some(own);
        if let Some(parana) = occurrence.fast_parana {
            *parana = None;
        }
        if let Some(paranas) = occurrence.vrata_parana {
            paranas.clear();
        }
        *occurrence.day = date;
    } else {
        *occurrence.day = own;
    }
    *occurrence.published = Some(published);
    Ok(())
}

fn occurrences(answer: &mut Observances) -> Vec<Occurrence<'_>> {
    let observances = answer.observances.iter_mut().map(|o| Occurrence {
        rule: &o.rule,
        day: &mut o.day,
        month: Some((o.month, o.adhika)),
        published: &mut o.published,
        fast_parana: None,
        vrata_parana: Some(&mut o.parana),
    });
    let fasts = answer.ekadashis.iter_mut().map(|f| Occurrence {
        rule: &f.rule,
        day: &mut f.day,
        month: Some((f.month, f.adhika)),
        published: &mut f.published,
        fast_parana: Some(&mut f.parana),
        vrata_parana: None,
    });
    let dated = answer.dated.iter_mut().map(|d| Occurrence {
        rule: &d.rule,
        day: &mut d.day,
        month: None,
        published: &mut d.published,
        fast_parana: None,
        vrata_parana: None,
    });
    observances.chain(fasts).chain(dated).collect()
}
