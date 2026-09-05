# Calendars: Gregorian, Julian, mixed and ISO week

Status: `draft`, written 2026-09-05 as a Phase 1 design page; revised
when `crates/calendar` is built. Derives from
`02-architecture/04-calendar-time-architecture.md` (the calendar port,
the fixed day number, resolution), `01-research/platform/05-calendars-timezones.md`,
`09-guidelines/04-adding-a-calendar.md` and `time-and-timezone.md`. The
arithmetic is Reingold and Dershowitz's, with the `calendrical_calculations`
crate as a differential oracle in tests (a dev-dependency under the
Unicode licence, ADR-0019).

## 1. Purpose and scope

The four arithmetic calendars every other calendar converts through:
proleptic Gregorian, proleptic Julian, the mixed civil calendar (Julian
until a transition date, Gregorian after), and the ISO week date. This
page settles the fixed day number and its relation to the Julian day,
the `Calendar` trait as these calendars implement it, the date and
week types, the transition model, era numbering, validation, and the
exhaustive tests. Bikram Sambat and the lunisolar calendars are their
own pages.

## 2. Inputs, settings and ports

A date in one of the four calendars, or a fixed day; the settings knob
`civil_calendar` selects which calendar a request's civil date is in
and `eras` which era numbers a date carries. No port: these calendars
need no ephemeris and no place.

## 3. The data model

### 3.1 The fixed day and the Julian day

```rust
pub struct FixedDay(i64);             // Reingold and Dershowitz: day 1 is Monday, 1 January 1 CE (proleptic Gregorian)
impl FixedDay {
    pub const JD_EPOCH: f64 = 1_721_424.5;                 // the Julian day at midnight beginning fixed day 0
    pub fn jd_at_midnight(self) -> JulianDay<Utc>;         // self + JD_EPOCH
    pub fn from_jd(jd: JulianDay<Utc>) -> (FixedDay, f64);  // the day and the fraction since midnight
    pub fn weekday(self) -> Weekday;                       // (self mod 7): 0 is Sunday
}
```

The Julian day is noon-based and continuous; the fixed day is
midnight-based and integer. Every conversion between an instant and a
civil date passes through `FixedDay`, and the fraction of a day never
touches calendar arithmetic. `Weekday` (Sunday 0 to Saturday 6) is the
calendar's type; the panchanga's `Vara` is the same number under the
sunrise-anchored day and the conversion is a named function, never a
cast.

### 3.2 Dates

```rust
pub struct CalendarDate {
    pub calendar: CalendarKey,
    pub year: i32,                    // astronomical numbering: 1 BCE is 0, 2 BCE is -1
    pub month: u8,                    // 1 to 12
    pub day: u8,                      // 1 to the month's length
    pub era: Option<EraNumber>,       // BCE or CE and the year in it, when requested
    pub flags: DateFlags,             // empty for these calendars
    pub resolution: Resolution,       // `Defined` here: exact by construction
}
pub struct IsoWeekDate { pub week_year: i32, pub week: u8 /* 1 to 53 */, pub weekday: u8 /* 1 Monday to 7 Sunday */ }
```

`Resolution::Defined` is added to the architecture's enum for calendars
that are mathematical definitions; `Tabular`, `Computed` and `Divergent`
stay for authority-published ones. Years are astronomical in the data
model and rendered with an era only at presentation, so year 0 exists
and arithmetic across the era boundary is ordinary.

### 3.3 The mixed calendar

`MIXED` is Julian through the day before a transition and Gregorian
from the transition day, the transition being a fixed day with a name:
`GREGORIAN_1582` (the papal reform: 4 October 1582 Julian is followed
by 15 October 1582 Gregorian) by default, with a cited table of national
transitions (Britain and its colonies 1752, Russia 1918, Greece 1923,
and the rest) as named alternatives selected on the calendar key
(`MIXED_GB`). The days skipped at a transition do not exist and are
refused.

## 4. Algorithms

Gregorian, from the book: `fixed = 365(y−1) + ⌊(y−1)/4⌋ − ⌊(y−1)/100⌋ +
⌊(y−1)/400⌋ + ⌊(367m − 362)/12⌋ + correction + d`, the correction 0 in
January and February, −1 in a leap year afterwards, −2 otherwise; the
inverse by the year approximation `⌊(400 × (fixed − epoch + 2)) / 146 097⌋`
refined by one comparison, then the month by the same formula. Julian
likewise with `⌊(y−1)/4⌋` alone and its own epoch. ISO week: the
week-year is the year of the Thursday of the week; week 1 holds the
first Thursday; the weekday is `((fixed − 1) mod 7) + 1`. Leap years:
Gregorian divisible by 4 except by 100 unless by 400; Julian divisible
by 4 (and, before 8 CE, the historical irregularity is out of scope:
the calendar is proleptic and says so). Month lengths from a table with
the leap adjustment. Every function is integer arithmetic on `i64` and
`i128` where a product can exceed `i64`; no floating point anywhere in
this crate.

## 5. The API

The `Calendar` trait of the architecture page, implemented by four
zero-sized types registered under `GREGORIAN`, `JULIAN`, `MIXED` (with
its variants) and `ISO_WEEK`; `calendar::convert(date, to:
CalendarKey)`, `calendar::from_fixed`, `calendar::to_fixed`,
`calendar::weekday`, `calendar::iso_week`, and batch forms over arrays.
C ABI: `ts_calendar_to_fixed`, `ts_calendar_from_fixed`,
`ts_calendar_convert`, `ts_calendar_month_length`, `ts_calendar_is_leap`;
dates cross as a `struct_size` struct with the calendar key id.
Bindings: a `CalendarDate` value type with validating constructors per
calendar (`gregorian(2000, 1, 1)`) and typed conversion methods.

## 6. Errors and degenerate states

A day beyond the month's length, a month outside 1 to 12, or a date in
a transition gap: `INVALID_ARG` with the field, the value and the
accepted range (for a gap, both edges). A year outside the supported
range (−9999 to 9999 in v1, a limit not a truth): `OUT_OF_RANGE`. There
are no degenerate states: these calendars are total on their range.

## 7. Performance budget

| operation | budget |
|---|---:|
| to or from fixed | 20 ns, no allocation |
| ISO week | 30 ns |
| a year of dates in a batch | 10 µs |

## 8. Tests

- Exhaustive round trip: every fixed day from −9999 to 9999 converts to
  a date and back in each calendar (about 7.3 million days per
  calendar, seconds in a test), never sampled.
- Differential: every day in the range agrees with the
  `calendrical_calculations` oracle for Gregorian, Julian and ISO week.
- Known dates: the epochs (fixed day 1, JD 1 721 425.5), J2000 (JD
  2 451 545.0 is 1 January 2000 at noon), the 1582 transition on both
  sides, the century rules (1900 not leap, 2000 leap, 2100 not), ISO
  week edges (31 December 2020 is week 53; 3 January 2021 is week 53 of
  2020; 30 December 2024 is week 1 of 2025), weekdays of known events.
- Property: month lengths sum to the year length; `weekday` advances by
  one per day and cycles at seven; the mixed calendar is monotonic
  across its transition.
- The cross-architecture hash matrix carries the round-trip corpus
  (ADR-0022).

## 9. Localisation

`sdk.calendar` holds month and weekday names, era names and date
patterns per calendar and locale; the calendars emit numbers and keys
only.

## 10. Open questions

- The national transition table for `MIXED` variants: the cited list
  and which ship in v1 (the 1582 default ships; Britain 1752 is the
  first variant because the English-language references use it).
- The supported year range: −9999 to 9999 as a limit; wider if a
  consumer needs it, since the arithmetic has no edge.
