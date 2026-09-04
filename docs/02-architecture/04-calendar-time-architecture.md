# Calendar and time architecture

Status: `draft`, revised 2026-09-04 (the dual-source resolution and the
day boundary added, ADR-0018 and ADR-0020). Depends on Q9.

## Time types (L0)

| type | meaning |
|---|---|
| `Instant` | a Julian day with an explicit timescale (`UT1`, `TT`, `UTC`) in `f64` plus, where needed, a split representation for sub-millisecond precision (integer day and fractional day) |
| `CivilDateTime` | year, month, day, hour, minute, second in a named calendar, without a zone |
| `ZonedDateTime` | civil plus a zone resolution: zone id or LMT, offset minutes, source, era, tzdb version |
| `LocalDay` | a civil date at a place with the sunrise-anchored day boundaries (sunrise, next sunrise) |
| `Duration` | days, ghati-pala, hours; explicit units |

All arithmetic on instants is in Julian days; conversion to civil goes
through a calendar; zones apply only at the civil boundary.

## The calendar port

```rust
pub trait Calendar {
    fn id(&self) -> CalendarId;                          // "gregorian", "bikram-sambat", "indian-lunisolar.amanta"
    fn capabilities(&self) -> CalendarCapabilities;      // range, needs_place, needs_ephemeris, eras
    fn from_fixed(&self, fixed: FixedDay, place: Option<Place>) -> Result<CalendarDate>;
    fn to_fixed(&self, date: &CalendarDate, place: Option<Place>) -> Result<FixedDay>;
    fn month_length(&self, year: i32, month: MonthRef) -> Result<u8>;
    fn is_leap(&self, year: i32) -> bool;
    fn eras(&self) -> &[EraDef];
}
```

`FixedDay` is the Reingold and Dershowitz fixed day number (an integer,
convertible to and from Julian day at civil midnight). `CalendarDate`
carries year, month (with leap and adhika flags), day (with kshaya and
doubled-day flags for lunisolar systems), era and a `resolution`:

```rust
pub enum Resolution {
    Tabular { authority: AuthorityId, edition: Version },   // from the authority's published tables; exact by definition
    Computed { model: ComputeModel, siddhanta: SiddhantaId }, // outside the tables' range
    Divergent { tabular: u8, computed: u8, followed: Followed }, // inside the range and the two disagree: reported, never hidden
}
```

The policy for an authority-published calendar: inside the tabular range
the table wins, because it is what people's calendars say; outside it,
compute and mark `Computed`; where the two disagree inside the range,
follow the table and surface `Divergent` in the provenance. The set of
divergent dates is itself a fixture, and for Bikram Sambat it is
published as a table (the first public record of where computation and
the official almanac disagree). A date that is ambiguous (a repeated
tithi, a doubled day) returns both candidates as a typed result, never a
silent choice.

Registered implementations in v1: Gregorian, proleptic Julian, mixed,
ISO week, Bikram Sambat, Indian lunisolar (Amanta and Purnimanta variants
sharing one engine), with era views (Vikram, Shaka, Kali, Nepal Sambat,
Buddha) computed sankranti-aware. Later: the ICU4X-backed set and regional
solar calendars, each as a plug-in.

Lunisolar calendars depend on the ephemeris port (new moons, sankrantis,
sunrise at the place) and cache their events per year in the context.

## Bikram Sambat specifically

- Official span: a table of month lengths per year from Nepal government
  publications, with a version stamp and a validation script against
  published almanacs (the baseline engine ships such a table and a checker).
- Outside the span: computed from the solar sankranti with Nepal's
  convention (a month begins on the day of the sankranti, or the next
  day if the sankranti falls after a threshold; the exact rule varies by
  almanac and must be a named variant with the official table as
  reference), and the result is marked `computed`.
- Era numbers: Vikram = BS year; Shaka, Kali and Nepal Sambat computed from
  the fixed day with their own new-year rules, never as fixed offsets.

## Day boundaries

A calendar date is meaningless without a day-boundary rule, and the
traditions differ: civil calendars begin at midnight, most Vedic reckoning
at sunrise, some contexts and the Islamic and Hebrew calendars at sunset,
historical astronomy at noon. `DayBoundary` is an explicit settings knob
recorded in provenance; sunrise-based boundaries depend on the place, so
the reference place is part of the query, never a global. This is a
frequent, silent source of one-day errors.

## Time zones

```rust
pub trait TimeZoneProvider {
    fn tzdb_version(&self) -> &str;
    fn offset_at(&self, zone: &ZoneId, instant: Instant) -> Result<OffsetInfo>;
    fn resolve_local(&self, zone: &ZoneId, civil: &CivilDateTime, policy: DstPolicy) -> Result<Resolution>;
    fn zones(&self) -> impl Iterator<Item = &ZoneId>;
}
```

The default implementation embeds tzdb (versioned) and is replaceable at
context creation. `DstPolicy` is explicit: on a gap `Error` (the baseline engine) or
`ShiftForward`; on an overlap `Earlier` (the baseline engine) or `Later`. The result
records what happened. LMT is a zone kind with offset = longitude × 4
minutes. Manual offsets are allowed and stamped `manual`.

The geo port (coordinates to zone id, canonical places) is separate and
optional; its default package carries the shape data.

## Research standard for every calendar

A calendar is marked stable only after the standard in
`09-guidelines/04-adding-a-calendar.md` is met: the authority identified,
primary sources obtained, at least three independent real-world sources
cross-validated (independence verified, since several consumer calendars
share one table), every day in the supported range round-tripped, and the
divergence envelope documented. Festivals are region-scoped rule packs
(`festivals-np`, `festivals-in-north`, ...) over the calendar, each rule
cited; the engine never privileges one region's determination. Hijri
ships tabular variants and Umm al-Qura, documented as computed
approximations, because observational Hijri is not computable; the
Chinese calendar is defined at UTC+8 specifically; Bengali has two
current variants (Bangladesh's reform and India's) and both ship.

## Sunrise-anchored day and ghati-pala

`time::LocalDay` computes sunrise and next sunrise (via the ephemeris port
or the fallback) with the profile's disc convention, day and night
durations, and converts between clock time and ghati-pala under civil
(24-minute) or proportional reckoning, both directions, with the rounding
policy that keeps 0 ghati 0 pala at sunrise (the baseline engine's ceiling rule).

## What the panchanga needs from here

A `LocalDay` at a place, the lunar month state (from the lunisolar
calendar), the era numbers, weekday from the local civil date, and the
formatting of all of it through `intl`. Delta T, sidereal time and the
sunrise solver come from `astro`, never from a provider directly.
