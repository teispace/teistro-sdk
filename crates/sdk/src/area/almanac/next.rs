//! The next day a rule, a kind of day or a catalogue festival falls on
//! after an instant at a place, with no year asked
//! (`03-design/next-occurrence.md`, SR-06): the festival reckoning over
//! windows that grow until one holds it or the look-ahead limit is
//! reached.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_core::catalogue::{Calendar, Kind, write_in_full};
use teistro_core::envelope::{Convention, Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_panchanga::festival::{
    GroupSpan, NextKind, Observances, PublishedStatus, limit_days, windows,
};
use teistro_time::ZoneSpec;

use super::AlmanacArea;
use crate::DayClock;
use crate::area::system_of;
use crate::asked::{instant, place_of};
use crate::festival_request::FestivalRequest;

/// The field a next-occurrence request's refusals are named under.
const NEXT: &str = "next";

/// What a search looks for: a rule of a festival request by its key, or a
/// kind of day the SDK reads by rules of its own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NextTarget {
    /// The rule of this key in the request's festivals: a karmakala rule,
    /// an Ekadashi rule, a rule counted from another's day, a dated rule,
    /// an anniversary, or a day a published set adds.
    Rule(String),
    /// A kind of day ([`NextKind::rules`]).
    Kind(NextKind),
    /// The group of this key in the request's festivals: its next span
    /// (`03-design/festival-groups.md` §5).
    Group(String),
}

/// A next-occurrence search: the instant after which, the place and the
/// clock, what is looked for, and how far ahead.
///
/// ```
/// use teistro::{NextKind, NextRequest, NextTarget};
///
/// let asked = NextRequest::from_json(
///     r#"{"after": 2461345.5, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700, "kind": "EKADASHI"}"#,
/// )?;
/// assert_eq!((asked.target, asked.limit_days), (NextTarget::Kind(NextKind::Ekadashi), 400));
/// let both = NextRequest::from_json(
///     r#"{"after": 2461345.5, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700, "kind": "PURNIMA", "rule": "HOLI"}"#,
/// )
/// .unwrap_err();
/// assert_eq!(both.field(), Some("next.rule"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct NextRequest {
    /// The instant after which; the civil day holding it is searched
    /// first, so a day it falls on counts.
    pub after: JulianDay<Utc>,
    /// The place.
    pub place: Place,
    /// The clock its civil days run on: an offset stated, or a zone each
    /// instant is read at the offset of then.
    pub clock: DayClock,
    /// The calendar the days are written in.
    pub calendar: Calendar,
    /// What is looked for.
    pub target: NextTarget,
    /// The festivals a rule is looked for in; a kind carries its own.
    pub festivals: Option<FestivalRequest>,
    /// How many days ahead to look, the first day among them.
    pub limit_days: u32,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "NextRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NextAsked {
    /// The instant, a UTC Julian day.
    after: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the civil days run at, in seconds, east
    /// positive; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone each instant is read on, at its offset then; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    #[serde(default)]
    calendar: Option<Calendar>,
    /// A kind of day; or `rule` or `group`, one of the three.
    #[serde(default)]
    kind: Option<NextKind>,
    /// A rule's key in `festivals`; or `kind` or `group`, one of the three.
    #[serde(default)]
    rule: Option<String>,
    /// A festival group's key in `festivals`: its next span; or `kind` or
    /// `rule`, one of the three.
    #[serde(default)]
    group: Option<String>,
    /// The festivals, as a festival request, with `rule` or `group`.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::festival_request::Asked>")
    )]
    festivals: Option<Value>,
    /// 1 to 3660; 400 when left out.
    #[serde(default)]
    limit_days: Option<u32>,
}

impl NextRequest {
    /// The record a binding sends, as JSON: `after`, a UTC Julian day;
    /// `latitudeDeg`, `longitudeDeg`, `altitudeM` and `utcOffsetSeconds`;
    /// `calendar` for the answer's days (Gregorian when left out); either
    /// `kind` (`EKADASHI`, `PURNIMA`, `AMAVASYA` or `SANKRANTI`) or `rule`,
    /// a key of `festivals`, a festival request; and `limitDays`, how many
    /// days ahead to look, 400 when left out.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, neither or both of `kind` and `rule`, `festivals` with a
    /// kind or a rule without it, a place or offset out of range, or a
    /// festival request its own reader refuses; `OUT_OF_RANGE` for a
    /// `limitDays` of 0 or past 3660; each named under `next`.
    pub fn from_json(text: &str) -> Result<NextRequest, Error> {
        let asked: NextAsked = teistro_core::strict::read(text, NEXT)?;
        asked.request().map_err(|why| why.under(NEXT))
    }
}

impl NextAsked {
    fn request(self) -> Result<NextRequest, Error> {
        let (target, festivals) = target_of(self.kind, self.rule, self.group, self.festivals)?;
        Ok(NextRequest {
            after: instant(self.after, "after")?,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            calendar: self.calendar.unwrap_or(Calendar::Gregorian),
            target,
            festivals,
            limit_days: limit_days(self.limit_days)?,
        })
    }
}

/// What a search looks for, read from a request's `kind`, `rule`, `group`
/// and `festivals`: a kind alone, or a rule or a group with the festivals
/// it is a key of. What [`NextRequest`] and an occurrences request read
/// alike, so both refuse alike.
///
/// # Errors
///
/// `INVALID_ARG` for none or two of `kind`, `rule` and `group`, named by
/// the one given second (`rule`, else `group`) or `kind` when none is;
/// `festivals` with a kind, or a rule or group without them, named
/// `festivals`; and what [`FestivalRequest::from_json`] refuses.
pub(super) fn target_of(
    kind: Option<NextKind>,
    rule: Option<String>,
    group: Option<String>,
    festivals: Option<Value>,
) -> Result<(NextTarget, Option<FestivalRequest>), Error> {
    let target = match (kind, rule, group) {
        (Some(kind), None, None) => NextTarget::Kind(kind),
        (None, Some(rule), None) => NextTarget::Rule(rule),
        (None, None, Some(group)) => NextTarget::Group(group),
        (None, None, None) => {
            return Err(Error::invalid_arg(
                "a search names what it looks for: a `kind`, a `rule` or a `group`",
            )
            .with_field("kind"));
        }
        (_, Some(_), _) => {
            return Err(Error::invalid_arg(
                "a search looks for one of a `kind`, a `rule` and a `group`",
            )
            .with_field("rule"));
        }
        (Some(_), None, Some(_)) => {
            return Err(Error::invalid_arg(
                "a search looks for one of a `kind`, a `rule` and a `group`",
            )
            .with_field("group"));
        }
    };
    let festivals = match (&target, festivals) {
        (NextTarget::Kind(_), None) => None,
        (NextTarget::Kind(_), Some(_)) => {
            return Err(Error::invalid_arg(
                "a kind is read by the SDK's own rules; to search a request's rules, name one as `rule`",
            )
            .with_field("festivals"));
        }
        (_, Some(festivals)) => Some(FestivalRequest::from_json(&festivals.to_string())?),
        (_, None) => {
            return Err(Error::invalid_arg(
                "a `rule` or a `group` is a key of `festivals`, which is left out",
            )
            .with_field("festivals")
            .with_hint("name a pack, as `{\"rules\": \"NEPAL_PANCHANGA\"}`"));
        }
    };
    Ok((target, festivals))
}

/// The festival request a target is reckoned over, cut to what it needs,
/// and the keys it looks for: a kind's own rules; a rule, the request
/// narrowed to it ([`FestivalRequest::narrowed_to`]); or a group's
/// members, the request narrowed to the group
/// ([`FestivalRequest::narrowed_to_group`]); checked.
///
/// # Errors
///
/// `INVALID_ARG` for a rule or a group with no festivals, named
/// `festivals`; a rule no list of them has, named `rule`; a group they do
/// not hold, named `group`; and what [`FestivalRequest::check`] refuses.
pub(super) fn reckoned_for(
    target: &NextTarget,
    festivals: Option<&FestivalRequest>,
) -> Result<(FestivalRequest, Vec<String>), Error> {
    let (reckoned, keys) = match target {
        NextTarget::Kind(kind) => {
            let read = kind.rules();
            let keys: Vec<String> = read
                .rules
                .iter()
                .map(|r| r.key.clone())
                .chain(read.ekadashis.iter().map(|r| r.key.clone()))
                .chain(read.dated.iter().map(|r| r.key.clone()))
                .collect();
            (FestivalRequest::of_kind(read), keys)
        }
        NextTarget::Rule(key) => {
            let asked = festivals.ok_or_else(|| {
                Error::invalid_arg("a `rule` is a key of `festivals`, which is left out")
                    .with_field("festivals")
            })?;
            let narrowed = asked.narrowed_to(key).ok_or_else(|| {
                Error::invalid_arg(format!(
                    "`{key}` is no rule of the festival request and no day its published sets add"
                ))
                .with_field("rule")
            })?;
            (narrowed, vec![key.clone()])
        }
        NextTarget::Group(key) => {
            let asked = festivals.ok_or_else(|| {
                Error::invalid_arg("a `group` is a key of `festivals`, which is left out")
                    .with_field("festivals")
            })?;
            let narrowed = asked.narrowed_to_group(key).ok_or_else(|| {
                Error::invalid_arg(format!("`{key}` is no group of the festival request"))
                    .with_field("group")
            })?;
            let members = narrowed
                .groups()
                .iter()
                .flat_map(|group| group.members.clone())
                .collect();
            (narrowed, members)
        }
    };
    reckoned.check()?;
    Ok((reckoned, keys))
}

/// The next day the target falls on, and what the festival reckoning
/// says of it there.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NextOccurrence {
    /// The day, in the request's calendar.
    pub day: CalendarDate,
    /// The keys of the rules that fall on it, in order: one for a rule,
    /// and for a kind each of its rules that does (the Smarta and the
    /// Vaishnava Ekadashi when they agree).
    pub keys: Vec<String>,
    /// The reckoning's entries for those keys on that day: the
    /// observances, Ekadashi fasts, dated days, anniversaries and added
    /// days, each as a festival answer holds it.
    pub festivals: Observances,
    /// The first day searched: the civil day holding the instant.
    pub from: CalendarDate,
    /// The look-ahead limit, in days.
    pub limit_days: u32,
    /// The days reckoned until it was found, the window holding it
    /// whole.
    pub searched_days: u32,
    /// For a group, its span: `day` is its first day, `keys` the members
    /// that fall in it and `festivals` their entries over the span; left
    /// out for a rule or a kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<GroupSpan>,
}

impl NextOccurrence {
    /// Where the answer names catalogue members outside its festivals.
    const MEMBERS: [(&'static str, Kind); 12] = [
        ("day.calendar", Kind::Calendar),
        ("day.era.era", Kind::Era),
        ("from.calendar", Kind::Calendar),
        ("from.era.era", Kind::Era),
        ("group.first.calendar", Kind::Calendar),
        ("group.first.era.era", Kind::Era),
        ("group.last.calendar", Kind::Calendar),
        ("group.last.era.era", Kind::Era),
        ("group.members.day.calendar", Kind::Calendar),
        ("group.members.day.era.era", Kind::Era),
        ("group.days.day.calendar", Kind::Calendar),
        ("group.days.day.era.era", Kind::Era),
    ];

    /// The answer as JSON, every catalogue member written as its full key:
    /// what the boundary carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the answer does not serialise, which one this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!("the next occurrence did not serialise: {error}"))
        })?;
        for (path, kind) in NextOccurrence::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        if let Some(festivals) = value.get_mut("festivals") {
            *festivals = self.festivals.in_full()?;
        }
        Ok(value)
    }
}

/// Whether a published set withdrew the day.
pub(super) fn withdrawn(published: Option<&teistro_panchanga::festival::Published>) -> bool {
    published.is_some_and(|p| p.status == PublishedStatus::Withdrawn)
}

/// Each day `keys` fall on in a reckoning, with the key: the observances,
/// Ekadashi fasts, dated days, anniversaries and added days, in that order,
/// a day a published set withdrew left out.
pub(super) fn kept_days<'f>(
    found: &'f Observances,
    keys: &'f [String],
) -> impl Iterator<Item = (&'f CalendarDate, &'f str)> + 'f {
    let wanted = move |key: &str| keys.iter().any(|k| k == key);
    found
        .observances
        .iter()
        .filter(move |o| wanted(&o.rule) && !withdrawn(o.published.as_ref()))
        .map(|o| (&o.day, o.rule.as_str()))
        .chain(
            found
                .ekadashis
                .iter()
                .filter(move |f| wanted(&f.rule) && !withdrawn(f.published.as_ref()))
                .map(|f| (&f.day, f.rule.as_str())),
        )
        .chain(
            found
                .dated
                .iter()
                .filter(move |d| wanted(&d.rule) && !withdrawn(d.published.as_ref()))
                .map(|d| (&d.day, d.rule.as_str())),
        )
        .chain(
            found
                .anniversaries
                .iter()
                .filter(move |a| wanted(&a.key) && !withdrawn(a.observance.published.as_ref()))
                .map(|a| (&a.observance.day, a.key.as_str())),
        )
        .chain(
            found
                .added
                .iter()
                .filter(move |a| wanted(&a.key))
                .map(|a| (&a.day, a.key.as_str())),
        )
}

/// The earliest day `keys` fall on in a reckoning, and the keys there.
fn earliest(
    found: &Observances,
    keys: &[String],
    calendar: &dyn teistro_calendar::CalendarSystem,
) -> Result<Option<(FixedDay, CalendarDate)>, Error> {
    let mut best: Option<(FixedDay, CalendarDate)> = None;
    for (day, _) in kept_days(found, keys) {
        let fixed = calendar.fixed_of(day)?;
        if best.as_ref().is_none_or(|(held, _)| fixed < *held) {
            best = Some((fixed, day.clone()));
        }
    }
    Ok(best)
}

/// The reckoning cut to `keys` on `day`, its unjudged and unplaced
/// entries left out, and the keys that remain, in order.
fn on_the_day(
    mut found: Observances,
    keys: &[String],
    day: &CalendarDate,
) -> (Observances, Vec<String>) {
    let wanted = |key: &str| keys.iter().any(|k| k == key);
    found
        .observances
        .retain(|o| wanted(&o.rule) && o.day == *day && !withdrawn(o.published.as_ref()));
    found
        .ekadashis
        .retain(|f| wanted(&f.rule) && f.day == *day && !withdrawn(f.published.as_ref()));
    found
        .dated
        .retain(|d| wanted(&d.rule) && d.day == *day && !withdrawn(d.published.as_ref()));
    found.anniversaries.retain(|a| {
        wanted(&a.key) && a.observance.day == *day && !withdrawn(a.observance.published.as_ref())
    });
    found.added.retain(|a| wanted(&a.key) && a.day == *day);
    found.unjudged.clear();
    found.unplaced.clear();
    let held: Vec<String> = keys
        .iter()
        .filter(|key| {
            found.observances.iter().any(|o| &o.rule == *key)
                || found.ekadashis.iter().any(|f| &f.rule == *key)
                || found.dated.iter().any(|d| &d.rule == *key)
                || found.anniversaries.iter().any(|a| &a.key == *key)
                || found.added.iter().any(|a| &a.key == *key)
        })
        .cloned()
        .collect();
    (found, held)
}

/// A span's reckoning cut to the group's member `keys`, its unjudged and
/// unplaced entries and its groups left out, and the members that fall in
/// it, in the group's order.
fn in_the_span(mut found: Observances, keys: &[String]) -> (Observances, Vec<String>) {
    let wanted = |key: &str| keys.iter().any(|k| k == key);
    found.keep(wanted);
    found
        .observances
        .retain(|o| !withdrawn(o.published.as_ref()));
    found.ekadashis.retain(|f| !withdrawn(f.published.as_ref()));
    found.dated.retain(|d| !withdrawn(d.published.as_ref()));
    found
        .anniversaries
        .retain(|a| !withdrawn(a.observance.published.as_ref()));
    found.unjudged.clear();
    found.unplaced.clear();
    found.groups.clear();
    let held = keys
        .iter()
        .filter(|key| {
            found.observances.iter().any(|o| &o.rule == *key)
                || found.ekadashis.iter().any(|f| &f.rule == *key)
                || found.dated.iter().any(|d| &d.rule == *key)
                || found.anniversaries.iter().any(|a| &a.key == *key)
                || found.added.iter().any(|a| &a.key == *key)
        })
        .cloned()
        .collect();
    (found, held)
}

impl AlmanacArea<'_> {
    /// The next day `request`'s target falls on, from the civil day holding
    /// its instant on its clock, within its look-ahead limit
    /// (`03-design/next-occurrence.md`). The festival reckoning
    /// ([`AlmanacArea::festivals`]) runs over windows of 32 days, then
    /// twice the last, so a day found early costs one short window; each
    /// window is the reckoning that range would have had on its own, its
    /// edges widened as every range's are. A rule is looked for with only
    /// what it needs of its request: itself, the rule it counts from, its
    /// saits and its published entries.
    ///
    /// ```no_run
    /// use teistro::{Context, NextRequest};
    ///
    /// let sdk = Context::builder().profile("nepali-committee").build()?;
    /// let asked = NextRequest::from_json(
    ///     r#"{"after": 2461345.5, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///         "utcOffsetSeconds": 20700, "rule": "VIJAYA_DASHAMI",
    ///         "festivals": {"rules": "NEPAL_PANCHANGA"}}"#,
    /// )?;
    /// let next = sdk.almanac().next(&asked)?;
    /// println!("{} on {}", next.value.keys[0], next.value.day);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a rule no list of the request has, named
    /// `rule`; `LIMIT` naming `limitDays` when nothing falls within the
    /// limit, with the limit's most as a hint; and as
    /// [`AlmanacArea::festivals`].
    pub fn next(self, request: &NextRequest) -> Result<Envelope<NextOccurrence>, Error> {
        self.unclocked().clocked(
            &request.clock,
            || Ok(request.after),
            |almanac, offset| almanac.next_on(request, offset),
        )
    }

    /// [`AlmanacArea::next`] on `offset`, the clock's at the instant, the
    /// days on the zone this area is on when it is.
    fn next_on(
        self,
        request: &NextRequest,
        offset: UtcOffset,
    ) -> Result<Envelope<NextOccurrence>, Error> {
        let (festivals, keys) = reckoned_for(&request.target, request.festivals.as_ref())?;
        let calendar = system_of(request.calendar)?;
        let local = request.after.get() + f64::from(offset.seconds()) / 86_400.0;
        let (start, _) = FixedDay::from_local_jd(local);
        let first = calendar.date_of(start)?;
        let mut searched = 0;
        for (ahead, days) in windows(request.limit_days) {
            let from = calendar.date_of(start.plus_days(i64::from(ahead)))?;
            let to = calendar.date_of(start.plus_days(i64::from(ahead + days) - 1))?;
            let Envelope {
                value,
                mut provenance,
            } = self.festivals(&from, &to, &request.place, offset, &festivals)?;
            searched += days;
            let (found, held, day, group) = if let NextTarget::Group(key) = &request.target {
                let Some(span) = value.groups.into_iter().find(|span| {
                    span.group == *key
                        && calendar
                            .fixed_of(&span.last)
                            .is_ok_and(|last| last >= start)
                }) else {
                    continue;
                };
                // The span's own reckoning, so its entries are whole
                // whichever windows it crossed.
                let whole =
                    self.festivals(&span.first, &span.last, &request.place, offset, &festivals)?;
                provenance = whole.provenance;
                let (found, held) = in_the_span(whole.value, &keys);
                (found, held, span.first.clone(), Some(span))
            } else {
                let Some((_, day)) = earliest(&value, &keys, calendar)? else {
                    continue;
                };
                let (found, held) = on_the_day(value, &keys, &day);
                (found, held, day, None)
            };
            // What the provenance hashes: the request, not the window that
            // held the day.
            let mut input = serde_json::json!({
                "after": request.after.get(),
                "place": request.place,
                "calendar": request.calendar,
                "target": request.target,
                "festivals": festivals,
                "limitDays": request.limit_days,
            });
            request.clock.seal_into(&mut input);
            provenance.input_hash = content_hash(&input);
            provenance.applied_conventions.push(Convention {
                knob: String::from("festival.next"),
                value: format!("{first}+{searched}d"),
                reason: String::from(
                    "the festival reckoning ran over windows of 32 days and then twice the last, from the civil day holding the instant, until one held the day; each window is the reckoning that range would have had",
                ),
            });
            return Ok(Envelope::sealing(
                NextOccurrence {
                    day,
                    keys: held,
                    festivals: found,
                    from: first,
                    limit_days: request.limit_days,
                    searched_days: searched,
                    group,
                },
                provenance,
            ));
        }
        Err(Error::limit(format!(
            "nothing falls within {} days of {first}",
            request.limit_days
        ))
        .with_field("limitDays")
        .with_hint(format!(
            "look further, up to {} days",
            teistro_panchanga::festival::MOST_LIMIT_DAYS
        )))
    }

    /// As [`AlmanacArea::next`], the answer written in full and sealed
    /// over what is written: what the boundary carries.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::next`].
    pub fn next_in_full(self, request: &NextRequest) -> Result<Envelope<Value>, Error> {
        let Envelope { value, provenance } = self.next(request)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
