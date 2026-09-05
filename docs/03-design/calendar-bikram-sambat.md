# Calendar: Bikram Sambat

Status: `draft`, written 2026-09-05 as a Phase 1 design page; revised
the same day twice: when `crates/calendar` landed the table-driven
calendar, and when the SDK's own engine landed with its measurement
against the official table (the source memo
`docs/calendars/bikram-sambat.md` holds the numbers). The maintainer's
mandate: the SDK computes Bikram Sambat from first principles for any
year, the way the Nepali panchanga does, with the published spans as
the authority inside their range, so that Nepal's panchanga makers can
use it. Derives from `02-architecture/04-calendar-time-architecture.md`
(the dual-source resolution), `01-research/platform/05-calendars-timezones.md`,
`calendar-gregorian-julian.md` (the fixed day), `siddhanta.md` (the
Sun) and the official month lengths for BS 1970 to 2095
(`crates/calendar/data/bikram-sambat.json`, rank 2 until the memo cites
the committee's own publication).

## 1. Purpose and scope

The official calendar of Nepal, the product's first calendar. Twelve
solar months of 29 to 32 days beginning at Baisakh, defined in
practice by the government-published almanac and in principle by the
Sun's entry into each sidereal sign. This page settles the table, the
computed extension and the rule that generates it, the resolution
every date reports, the era numbers that ride on it, the algorithms,
the tests and the source memo the calendar cannot ship without.

## 2. Inputs, settings and ports

A BS date or a fixed day for the table; for the engine, a `SolarModel`
(the Surya Siddhanta today; modern positions through the ephemeris port
with the profile's ayanamsha once the port is promoted), a `LocalClock`
(Nepal's history from the time layer; never hard-coded in the calendar),
a place (Kathmandu by convention for the national calendar, stamped) and
a `MonthStartRule`. The settings knobs `civil_calendar`, `eras`,
`siddhanta` and `ayanamsha` choose these in a profile. Ports: the
ephemeris port through `astro`, for the drik model only.

## 3. The data model

```rust
pub struct Table {                                  // what cargo xtask gen calendars writes (generated.rs)
    first_year: i32, month_lengths: &'static [[u8; 12]],   // 1700 to 2500 BS, Baisakh to Chaitra
    anchor: (i32, (i32, u8, u8)),                   // 1 Baisakh 1970 BS = 13 April 1913
    official: (i32, i32),                           // 1970 to 2095, the published span
    authority: &'static str, edition: &'static str, // the committee; official-1970-2095/1
    frame: &'static str,                            // model, clock, place and rule of the computed rows
    divergences: &'static [Divergence],             // inside the official span, where the engine differs
    model_rows: &'static [ModelRow],                // the engine's rows for those years, to label dates by
}
pub struct Divergence { year: i32, month: u8, tabular: u8, computed: u8 }
pub struct ModelRow { year: i32, start_offset: i8, months: [u8; 12] }
pub enum MonthStartRule {
    SankrantiDay,           // the civil day of the sankranti (Orissa)
    FollowingDay,           // the day after (Bengal)
    Shifted { days: f64 },  // the civil day of the sankranti moved by so many days: the family every uniform convention belongs to
    SunriseToSunrise,       // the almanac day: before dawn belongs to the day before
    BeforeSunset,           // Tamil
    BeforeAparahna,         // Malabar: three fifths of the daylight
    Punyakala,              // the Dharmasindhu: Karka by sunrise, Makara by sunset, the ten others by the civil day; Nepal's official calendar
}
pub trait SolarModel { fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64>; fn day_arc(&self, day: FixedDay, place: &Place) -> Result<Option<DayArc>>; fn describe(&self) -> String; }
pub struct Engine<'a> { model: &'a dyn SolarModel, clock: &'a dyn LocalClock, place: Place, rule: MonthStartRule }
pub struct YearRow { year: i32, start: FixedDay, months: [u8; 12], sankrantis: [JulianDay<Utc>; 12] }
pub struct FitReport { frame, years, years_exact, totals_matched, months, months_matched, drift_end, drift_max, start_offset_max, divergences }
```

A BS date is `CalendarDate { calendar: BIKRAM_SAMBAT, year, month 1 to
12, day 1 to 32, era: Vikrama, resolution }` where the resolution is
`Tabular { authority, edition }` inside the official span, `Computed {
model: frame }` outside it, and `Divergent { tabular: MonthDay,
computed: MonthDay, model }` inside the span on every day the engine
labels differently: the table wins, because it is what the country's
calendars say, and the disagreement is reported. The divergence set is
generated with the table, checked in, and published in the memo: the
first public record of where computation and the official almanac
disagree (eleven boundaries in 126 years).

Era numbers ride on the fixed day, never on offsets: Vikrama is the BS
year; Shaka is the Gregorian year less 78, or less 79 before the Mesha
sankranti of that year; Kali Yuga is the Gregorian year plus 3101 with
the same sankranti rule; Nepal Sambat begins at Kartik's new moon and
is computed by the lunisolar calendar (Phase 2) or, until then, from a
table of new-year dates stamped `Tabular`; the Buddha era's rule is
taken from the memo. The baseline engine's fixed-offset eras were wrong
by centuries (`05-calendars-timezones.md`); every era here has a
new-year rule.

## 4. Algorithms

**To fixed.** Look the year up in the table; the year's start day is
precomputed (`year_starts`, from the anchor in both directions); add the
lengths of the months before the date's month and the day less one.
**From fixed.** Binary search `year_starts` for the year, then walk at
most twelve month lengths; inside the official span, if the year has a
model row, label the day by the engine's row too and report a
`Divergent` resolution when the labels differ. Both are integer
arithmetic; no astronomy runs at conversion time.

**Computing a year** (`Engine::year`). Year Y begins at the Mesha
sankranti of Gregorian year Y − 57: the engine finds the thirteen
sankrantis from that Mesha to the next through `find_sankranti` (the
shared solver of `astro`: a jump by the mean solar rate, a bracket, a
bisection to a tenth of a second), places each on a civil day by the
rule under the clock (the rules that need the day's arc take sunrise and
sunset from the model at the place), and takes the differences as month
lengths. `Engine::sankrantis` and `Engine::year_from` split the two
halves so a measurement finds the instants once and tries every rule.

**The rule.** Chosen by measurement: `cargo xtask calendars bs-fit` runs
every model, clock and rule over the official span and publishes
agreement, drift, the offset of every 1 Baisakh and the divergence set;
`--detail` shows which sankranti decides how. The result (the memo, R5):
ten sankrantis follow the civil day in Nepal's clock, Karka the
sunrise-to-sunrise day and Makara the before-sunset rule, which is the
Dharmasindhu's punya-kala convention, `MonthStartRule::Punyakala`. Under
the text's Surya Siddhanta at Kathmandu it reproduces 1490 of 1512 month
lengths (98.5 %), 116 of 126 years exactly, every year total and every
New Year, with no drift; the baseline engine's generator had reached
87.0 % and 61 year totals with a fitted cutoff.

**Generating the table** (`cargo xtask gen calendars`). The official
rows verbatim, every other row of BS 1700 to 2500 from the engine under
the shipped frame, the divergences and the engine's rows for divergent
years, and the frame stamped in the file's header; the generator
refuses a frame whose running day count leaves zero. `check-calendars`
regenerates in memory and compares, in CI.

## 5. The API

Through the `CalendarSystem` trait, registered under `BIKRAM_SAMBAT`,
plus `BikramSambat::{shipped, over(&Table), years, official_years,
edition, frame, divergences, year_start, table}`; the engine
`bikram_sambat::{Engine, YearRow, KATHMANDU, YEAR_OFFSET, SHIPPED_RULE,
SHIPPED_SPAN, fit, FitReport, Divergence}`; the solar module
`solar::{SolarModel, DayArc, MonthStartRule, find_sankranti, Sankranti}`.
C ABI: the generic calendar entry points; the resolution crosses as a
typed enum with its fields. Bindings: `bs(2081, 1, 1)` constructors
validating month and day against the table, and the resolution as a
discriminated union on every converted date.

## 6. Errors and degenerate states

A day beyond the month's length for that year: `INVALID_ARG` with the
length. A year outside the table: `OUT_OF_RANGE` with the coverage. A
divergent date is a state, not an error. A rule that needs a sunrise on
a day without one: `UNSUPPORTED` naming the day and the place (never at
Kathmandu). A sankranti search that does not converge: `NOT_CONVERGED`
naming the sign and the instant (never seen; every search has a cap).

## 7. Performance budget

| operation | budget | measured |
|---|---:|---:|
| to or from fixed | 50 ns, no allocation | as the arithmetic calendars |
| table size | 9.6 KB of month lengths, 6.4 KB of year starts at runtime | 801 rows |
| computing a year (thirteen sankrantis, about thirty evaluations each at 54 ns) | 100 µs | about 25 µs |
| generating the shipped table (801 years, release) | seconds | about a tenth of a second |

## 8. Tests

- Exhaustive round trip over the whole span (1700 to 2500, about
  292 000 days), never sampled; every year 365 or 366 days.
- Known dates: the anchor (1 Baisakh 1970 is 13 April 1913), 2072 (14
  April 2015), 2081 (13 April 2024), and 2 Magh 1990 (15 January 1934).
- The shipped table is what the engine computes under its frame: a
  sample of computed years reproduces exactly, and the official span's
  divergences are the engine's (so the generated file, the engine and
  the rule cannot drift apart).
- Every divergence names an official year, a real difference and a model
  row; a divergent year has divergent dates carrying both labels; a clean
  year is tabular on every day.
- The rules: each named row places morning, afternoon and evening
  sankrantis as its tradition says (a model whose Sun rises at six and
  sets at eighteen); the clock decides the civil day; a rule needing a
  sunrise refuses a polar day and the punya-kala rule needs the arc only
  at the ayana sankrantis.
- The sankranti finder: the Mesha sankranti of 2024 falls on 13 April,
  the next sign a month later, under sixty evaluations.
- The engine: a year of twelve months of 29 to 32 days, consecutive years
  abutting, the frame stamp naming the rule.
- The fit: months, years, drift and offsets counted on a synthetic table.
- Deferred until the memo's R2: every month length against the
  committee's own publication; the divergence set as a conformance
  fixture; the fixture charts of spike 1 with their BS dates.

## 9. Localisation

`sdk.calendar` holds the month names (Baisakh to Chaitra in the four
launch languages), the `गते` date pattern, Devanagari numerals through
the locale's numbering system, and the era names.

## 10. Open questions

1. The committee's own publications and tables of sankranti instants
   (the memo's R1 to R3): they would settle the eleven residual
   boundaries, which lie within 25 minutes of the rule's boundary, and
   the exact Dharmasindhu verse the punya-kala rule follows (cruxes C29).
2. The drik model through the ephemeris port as a second `SolarModel`,
   so the memo's comparison of classical and modern positions comes from
   the SDK's own code.
3. The Buddha era's new-year rule in Nepali usage.
4. Nepal Sambat's tabular stopgap until the lunisolar calendar exists.
