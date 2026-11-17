//! Every day a rule or a kind of day falls on, year by year over a span of
//! years in one call (`03-design/rule-years.md`, SR-126): the span's days
//! founded once and the target reckoned over them once, each year then
//! read from that one reckoning, rather than a festival request a year.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::{CalendarDate, FixedDay};
use teistro_core::catalogue::{Calendar, Kind, write_in_full};
use teistro_core::envelope::{Convention, Envelope, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::festival::{NextKind, Observances};
use teistro_time::ZoneSpec;

use super::next::{kept_days, reckoned_for, target_of};
use super::{AlmanacArea, NextTarget, answered_run, widening, without_printed};
use crate::DayClock;
use crate::area::system_of;
use crate::asked::place_of;
use crate::festival_request::FestivalRequest;

/// The field an occurrences request's refusals are named under.
const OCCURRENCES: &str = "occurrences";

/// The most years one occurrences request may span: half a century,
/// past any page that shows a rule's days around a year, and inside the
/// one run of days a span is founded as ([`teistro_panchanga::almanac::MOST_SPAN_DAYS`])
/// with any rule's widening. Past it the refusal names `years` (C2125).
pub const MOST_OCCURRENCE_YEARS: u16 = 50;

/// A span of years, at a place, and what is looked for in each: a rule of
/// a festival request or a kind of day.
///
/// ```
/// use teistro::{NextTarget, OccurrencesRequest};
///
/// let asked = OccurrencesRequest::from_json(
///     r#"{"fromYear": 2080, "years": 8, "calendar": "BIKRAM_SAMBAT",
///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}}"#,
/// )?;
/// assert_eq!((asked.from_year, asked.years), (2080, 8));
/// assert_eq!(asked.target, NextTarget::Rule("VIJAYA_DASHAMI".into()));
/// let none = OccurrencesRequest::from_json(
///     r#"{"fromYear": 2026, "years": 0, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
///         "utcOffsetSeconds": 20700, "kind": "EKADASHI"}"#,
/// )
/// .unwrap_err();
/// assert_eq!(none.field(), Some("occurrences.years"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct OccurrencesRequest {
    /// The first year, in the request's calendar.
    pub from_year: i32,
    /// How many years, the first among them.
    pub years: u16,
    /// The place every day is judged at.
    pub place: Place,
    /// The clock its civil days run on: an offset stated, or a zone each
    /// instant is read at the offset of then.
    pub clock: DayClock,
    /// The calendar the years are counted in and the days written in.
    pub calendar: Calendar,
    /// What is looked for.
    pub target: NextTarget,
    /// The festivals a rule or a group is looked for in; a kind carries
    /// its own.
    pub festivals: Option<FestivalRequest>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "OccurrencesRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OccurrencesAsked {
    /// The first year, in `calendar`: the span begins on its month 1, day 1.
    from_year: i32,
    /// 1 to 50: the span ends the day before month 1 of the year after.
    years: u16,
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
    /// The calendar the years are counted in and the days written in;
    /// Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// A kind of day; or `rule` or `group`, one of the three.
    #[serde(default)]
    kind: Option<NextKind>,
    /// A rule's key in `festivals`; or `kind` or `group`, one of the three.
    #[serde(default)]
    rule: Option<String>,
    /// A festival group's key in `festivals`: its members' days, and each
    /// span reaching a year; or `kind` or `rule`, one of the three.
    #[serde(default)]
    group: Option<String>,
    /// The festivals, as a festival request, with `rule` or `group`.
    #[serde(default)]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<crate::festival_request::Asked>")
    )]
    festivals: Option<Value>,
}

impl OccurrencesRequest {
    /// The record a binding sends, as JSON: `fromYear` and `years`, 1 to
    /// 50, counted in `calendar` (Gregorian when left out), which the
    /// answer's days are written in too; `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds` or `zone`; and one of `kind`
    /// (`EKADASHI`, `PURNIMA`, `AMAVASYA` or `SANKRANTI`), `rule` or
    /// `group`, the last two keys of `festivals`, a festival request.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, none or two of `kind`, `rule` and `group`, `festivals` with a
    /// kind or a rule or group without it, a place or offset out of range, or a festival
    /// request its own reader refuses; `OUT_OF_RANGE` for `years` of 0 or
    /// past [`MOST_OCCURRENCE_YEARS`]; each named under `occurrences`.
    pub fn from_json(text: &str) -> Result<OccurrencesRequest, Error> {
        let asked: OccurrencesAsked = teistro_core::strict::read(text, OCCURRENCES)?;
        asked.request().map_err(|why| why.under(OCCURRENCES))
    }
}

/// The span a request may be, read or built by hand.
fn check_years(years: u16) -> Result<(), Error> {
    if (1..=MOST_OCCURRENCE_YEARS).contains(&years) {
        return Ok(());
    }
    Err(Error::new(
        Status::OutOfRange,
        format!("a request spans 1 to {MOST_OCCURRENCE_YEARS} years, not {years}"),
    )
    .with_field("years")
    .with_hint(format!(
        "ask 1 to {MOST_OCCURRENCE_YEARS} years, and a longer span as requests that follow on"
    )))
}

impl OccurrencesAsked {
    fn request(self) -> Result<OccurrencesRequest, Error> {
        check_years(self.years)?;
        let (target, festivals) = target_of(self.kind, self.rule, self.group, self.festivals)?;
        Ok(OccurrencesRequest {
            from_year: self.from_year,
            years: self.years,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            calendar: self.calendar.unwrap_or(Calendar::Gregorian),
            target,
            festivals,
        })
    }
}

/// Every day a target falls on, year by year.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Occurrences {
    /// The keys looked for, in order: the rule's, each of a kind's rules
    /// (the Smarta and the Vaishnava Ekadashi), or each of a group's
    /// members.
    pub keys: Vec<String>,
    /// Each year of the span, in order, every one present, a year the
    /// target falls in on no day among them (C2121).
    pub years: Vec<OccurrenceYear>,
}

/// One year of an [`Occurrences`] answer.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct OccurrenceYear {
    /// The year, in the request's calendar.
    pub year: i32,
    /// Its first day: month 1, day 1.
    pub first: CalendarDate,
    /// Its last day: the day before the next year's first.
    pub last: CalendarDate,
    /// Each day of the year the target falls on, in order: none, one, or
    /// more when a lunar rule meets a solar year twice, or a kind recurs
    /// (C2120, C2121). A day a published set withdrew is not among them
    /// (C2122).
    pub days: Vec<OccurrenceDay>,
    /// The reckoning's entries for the keys looked for in the year, as a
    /// festival answer holds them: the days, a withdrawn day marked so,
    /// an occurrence no day could be given to (C2122, C2123), and for a
    /// group each of its spans that reaches the year, whole, as a range
    /// over the year answers it.
    pub festivals: Observances,
}

/// A day of an [`OccurrenceYear`] and the keys that fall on it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct OccurrenceDay {
    /// The day, in the request's calendar.
    pub day: CalendarDate,
    /// The keys looked for that fall on it, in the order they were looked
    /// for: one for a rule; for a kind each of its rules that does.
    pub keys: Vec<String>,
}

impl Occurrences {
    /// Where the answer names catalogue members outside its festivals.
    const MEMBERS: [(&'static str, Kind); 6] = [
        ("years.first.calendar", Kind::Calendar),
        ("years.first.era.era", Kind::Era),
        ("years.last.calendar", Kind::Calendar),
        ("years.last.era.era", Kind::Era),
        ("years.days.day.calendar", Kind::Calendar),
        ("years.days.day.era.era", Kind::Era),
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
            Error::internal(format!("the occurrences did not serialise: {error}"))
        })?;
        for (path, kind) in Occurrences::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        if let Some(Value::Array(years)) = value.get_mut("years") {
            for (written, year) in years.iter_mut().zip(&self.years) {
                if let Some(festivals) = written.get_mut("festivals") {
                    *festivals = year.festivals.in_full()?;
                }
            }
        }
        Ok(value)
    }
}

/// A year's bounds: its number, first and last days, and the instants its
/// first sunrise and the sunrise after its last day fall at, when the
/// founded days hold them.
struct Bounds {
    year: i32,
    first: FixedDay,
    last: FixedDay,
    start: Option<f64>,
    end: Option<f64>,
}

/// The span's reckoning cut to one year and to `keys`.
fn year_of(
    found: &Observances,
    keys: &[String],
    bounds: &Bounds,
    calendar: &dyn teistro_calendar::CalendarSystem,
) -> Observances {
    let wanted = |key: &str| keys.iter().any(|k| k == key);
    let inside = |date: &CalendarDate| {
        calendar
            .fixed_of(date)
            .is_ok_and(|day| bounds.first.days_until(day) >= 0 && day.days_until(bounds.last) >= 0)
    };
    let cut = |rule: &str, day: &CalendarDate| wanted(rule) && inside(day);
    Observances {
        observances: found
            .observances
            .iter()
            .filter(|o| cut(&o.rule, &o.day))
            .cloned()
            .collect(),
        ekadashis: found
            .ekadashis
            .iter()
            .filter(|f| cut(&f.rule, &f.day))
            .cloned()
            .collect(),
        unjudged: found
            .unjudged
            .iter()
            .filter(|u| {
                wanted(&u.rule)
                    && bounds.start.is_some_and(|start| u.tithi.to.get() > start)
                    && bounds.end.is_some_and(|end| u.tithi.from.get() < end)
            })
            .cloned()
            .collect(),
        dated: found
            .dated
            .iter()
            .filter(|d| cut(&d.rule, &d.day))
            .cloned()
            .collect(),
        anniversaries: found
            .anniversaries
            .iter()
            .filter(|a| cut(&a.key, &a.observance.day))
            .cloned()
            .collect(),
        added: found
            .added
            .iter()
            .filter(|a| cut(&a.key, &a.day))
            .cloned()
            .collect(),
        unplaced: found
            .unplaced
            .iter()
            .filter(|u| cut(&u.rule, &u.day))
            .cloned()
            .collect(),
        applied: found.applied.clone(),
        // Whole, as a range over the year answers a span reaching it.
        groups: found
            .groups
            .iter()
            .filter(|span| {
                calendar
                    .fixed_of(&span.first)
                    .is_ok_and(|from| from.days_until(bounds.last) >= 0)
                    && calendar
                        .fixed_of(&span.last)
                        .is_ok_and(|to| bounds.first.days_until(to) >= 0)
            })
            .cloned()
            .collect(),
    }
}

/// The days of a year's entries, in order, each with the keys looked for
/// that fall on it, a withdrawn day left out.
fn days_of(
    found: &Observances,
    keys: &[String],
    calendar: &dyn teistro_calendar::CalendarSystem,
) -> Result<Vec<OccurrenceDay>, Error> {
    let mut held: Vec<(FixedDay, CalendarDate, Vec<&str>)> = Vec::new();
    for (day, key) in kept_days(found, keys) {
        let fixed = calendar.fixed_of(day)?;
        match held.iter_mut().find(|(at, _, _)| *at == fixed) {
            Some((_, _, on)) => on.push(key),
            None => held.push((fixed, day.clone(), vec![key])),
        }
    }
    held.sort_by_key(|(fixed, _, _)| *fixed);
    Ok(held
        .into_iter()
        .map(|(_, day, on)| OccurrenceDay {
            day,
            keys: keys
                .iter()
                .filter(|key| on.contains(&key.as_str()))
                .cloned()
                .collect(),
        })
        .collect())
}

impl AlmanacArea<'_> {
    /// Every day `request`'s target falls on in each year of its span
    /// (`03-design/rule-years.md`): a rule of a festival request, narrowed
    /// to what its days need ([`FestivalRequest::narrowed_to`]), or a kind
    /// of day read by the SDK's own rules, as [`AlmanacArea::next`] reads
    /// them. The span's days are founded once, from the widening before
    /// its first year to the widening after its last, and the target is
    /// reckoned over them once; each year is then cut from that one
    /// reckoning, which is the reckoning the year would have had alone. A
    /// year holds the days whose civil date lies in it, so it lists none,
    /// one or several, never a pick among them.
    ///
    /// ```no_run
    /// use teistro::{Context, OccurrencesRequest};
    ///
    /// let sdk = Context::builder().profile("nepali-committee").build()?;
    /// let asked = OccurrencesRequest::from_json(
    ///     r#"{"fromYear": 2080, "years": 8, "calendar": "BIKRAM_SAMBAT",
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}}"#,
    /// )?;
    /// for year in &sdk.almanac().occurrences(&asked)?.value.years {
    ///     println!("{}: {:?}", year.year, year.days.iter().map(|d| d.day.to_string()).collect::<Vec<_>>());
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a rule no list of the request has, named `rule`;
    /// a context with no ephemeris; a calendar the SDK does not ship; a
    /// span the ephemeris does not reach; should the request have been
    /// built by hand, a span past [`MOST_OCCURRENCE_YEARS`], named
    /// `occurrences.years`; and as [`AlmanacArea::festivals`].
    pub fn occurrences(self, request: &OccurrencesRequest) -> Result<Envelope<Occurrences>, Error> {
        let first = CalendarDate::defined(request.calendar, request.from_year, 1, 1);
        self.unclocked()
            .clocked_from(&request.clock, &first, |almanac, offset| {
                almanac.occurrences_on(request, offset)
            })
    }

    /// [`AlmanacArea::occurrences`] on `offset`, the days on the zone this
    /// area is on when it is.
    fn occurrences_on(
        self,
        request: &OccurrencesRequest,
        offset: UtcOffset,
    ) -> Result<Envelope<Occurrences>, Error> {
        check_years(request.years).map_err(|why| why.under(OCCURRENCES))?;
        let (festivals, keys) = reckoned_for(&request.target, request.festivals.as_ref())?;
        let (calendar, place) = (request.calendar, &request.place);
        let system = system_of(calendar)?;
        let first_of = |year: i32| system.fixed_of(&CalendarDate::defined(calendar, year, 1, 1));
        let mut bounds = Vec::with_capacity(usize::from(request.years));
        for step in 0..i32::from(request.years) {
            let year = request.from_year.saturating_add(step);
            bounds.push(Bounds {
                year,
                first: first_of(year)?,
                last: first_of(year.saturating_add(1))?.plus_days(-1),
                start: None,
                end: None,
            });
        }
        let (Some(first), Some(last)) = (
            bounds.first().map(|b| b.first),
            bounds.last().map(|b| b.last),
        ) else {
            return Err(Error::internal("a checked span holds no year"));
        };
        let (back, ahead) = widening(&festivals);
        // The widening is cut to the days the ephemeris answers, as a
        // festival call's is (SR-136, C2248): a span at an end of the
        // coverage is reckoned from the days there are.
        let answered = self.with_almanac(calendar, offset, |almanac| almanac.answered(place))?;
        let (low, high) = answered_run(answered, first.plus_days(-back), last.plus_days(ahead))
            .unwrap_or((first, last));
        let (from, to) = (system.date_of(low)?, system.date_of(high)?);
        let Envelope {
            value: viewed,
            mut provenance,
        } = self.with_almanac(calendar, offset, |almanac| {
            almanac.festival_span(&from, &to, place)
        })?;
        without_printed(&mut provenance);
        for day in &viewed {
            let fixed = system.fixed_of(&day.date)?;
            if let Some(year) = bounds.iter_mut().find(|b| b.first == fixed) {
                year.start = Some(day.sunrise.get());
            }
            if let Some(year) = bounds.iter_mut().find(|b| b.last == fixed) {
                year.end = Some(day.next_sunrise.get());
            }
        }
        let (found, (from, to)) = self.reckoned_over(
            (calendar, first, last),
            (place, offset, answered),
            &festivals,
            (viewed, (from, to)),
        )?;
        let mut years = Vec::with_capacity(bounds.len());
        for year in &bounds {
            let held = year_of(&found, &keys, year, system);
            years.push(OccurrenceYear {
                year: year.year,
                first: system.date_of(year.first)?,
                last: system.date_of(year.last)?,
                days: days_of(&held, &keys, system)?,
                festivals: held,
            });
        }
        // What the provenance hashes: the request, not the days founded.
        let mut input = serde_json::json!({
            "fromYear": request.from_year,
            "years": request.years,
            "place": request.place,
            "calendar": request.calendar,
            "target": request.target,
            "festivals": festivals,
        });
        request.clock.seal_into(&mut input);
        provenance.input_hash = content_hash(&input);
        provenance.applied_conventions.push(Convention {
            knob: String::from("festival.years"),
            value: format!("{from}..{to}"),
            reason: String::from(
                "the span's days were founded once, widened as a festival range is, and the target reckoned over them once; each year was cut from that reckoning, a year holding the days whose civil date lies in it",
            ),
        });
        Ok(Envelope::sealing(Occurrences { keys, years }, provenance))
    }

    /// As [`AlmanacArea::occurrences`], the answer written in full and
    /// sealed over what is written: what the boundary carries.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::occurrences`].
    pub fn occurrences_in_full(
        self,
        request: &OccurrencesRequest,
    ) -> Result<Envelope<Value>, Error> {
        let Envelope { value, provenance } = self.occurrences(request)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
