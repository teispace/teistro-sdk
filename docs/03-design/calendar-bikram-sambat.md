# Calendar: Bikram Sambat

Status: `draft`, written 2026-09-05 as a Phase 1 design page and revised
the same day when `crates/calendar` landed the table-driven calendar
(BS 1856 to 2457, anchored on 1 Baisakh 1970, every day round-tripped)
and the source memo (`docs/calendars/bikram-sambat.md`) recorded what
the baseline engine's generator established. The maintainer's mandate:
the SDK computes Bikram Sambat from first principles for any year, the
way the Nepali panchanga does, with the published spans as the authority
inside their range, so that Nepal's panchanga makers can use it. Derives from
`02-architecture/04-calendar-time-architecture.md` (the dual-source
resolution), `01-research/platform/05-calendars-timezones.md`,
`calendar-gregorian-julian.md` (the fixed day) and the baseline
engine's Bikram Sambat data: an official table of month lengths for
1970 to 2095 BS with its epoch, a computed extension for 1856 to 2457 BS,
and a `source` flag per year, all rank 2 (ADR-0018) until the memo
cites the authority's publications.

## 1. Purpose and scope

The official calendar of Nepal, the product's first calendar. Twelve
solar months of 29 to 32 days beginning at Baisakh, defined in
practice by the government-published almanac and in principle by the
Sun's entry into each sidereal sign. This page settles the table, the
computed extension and the rule that generates it, the resolution
every date reports, the era numbers that ride on it, the algorithms,
the tests and the source memo the calendar cannot ship without.

## 2. Inputs, settings and ports

A BS date or a fixed day; the settings knobs `civil_calendar`, `eras`,
`ayanamsha` and `siddhanta` (for the computed extension: the sankranti
instants come from `astro` over the ephemeris port under the profile's
sidereal frame), `day_boundary` (the civil day in Nepal begins at
sunrise for almanac purposes) and the place for sunrise-based rules
(Kathmandu by convention for the national calendar, stamped). Ports:
the ephemeris port through `astro`, only for years outside the table.

## 3. The data model

```rust
pub struct BsTable {
    pub authority: AuthorityId,             // the publishing body, from the memo
    pub edition: Version,                   // the table's version stamp
    pub first_year: i32, pub last_year: i32,   // 1970 to 2095 in the baseline's data
    pub epoch: FixedDay,                    // 1 Baisakh 1970 BS = 13 April 1913 Gregorian
    pub month_lengths: &'static [[u8; 12]], // per year, Baisakh to Chaitra
    pub year_starts: &'static [FixedDay],   // derived at build time for O(1) conversion
}
pub struct BsComputed {
    pub rule: MonthStartRule,               // the named sankranti-to-month-start convention
    pub frame: FrameStamp,                  // ayanamsha, siddhanta, place, day boundary used
    pub first_year: i32, pub last_year: i32,   // 1856 to 2457 in the baseline's data
    pub month_lengths: Vec<[u8; 12]>,       // generated and shipped as data with its frame stamped
}
pub enum MonthStartRule {
    SankrantiDay,                           // the month begins on the civil day containing the sankranti
    Threshold { fraction: DayFraction },    // or the next day when the sankranti falls at or after the fraction (0.705 fitted)
    SunriseToSunrise,                       // the day whose sunrise-to-sunrise span contains the sankranti
    BeforeSunset,                           // Tamil usage
    BeforeAparahna,                         // Malayalam usage
    BeforeMidnight,                         // Bengali usage
}
pub enum SankrantiSource {
    SuryaSiddhanta { bija: bool },          // the classical model; 87 % of month splits against the official table
    Drik { ayanamsha: AyanamshaChoice },    // modern positions through the ephemeris port; 72 %
}
```

A BS date is `CalendarDate { calendar: BIKRAM_SAMBAT, year, month 1 to
12, day 1 to 32, era: Vikrama, resolution }` where the resolution is
`Tabular { authority, edition }` inside the table, `Computed { model:
rule, siddhanta }` outside it, and `Divergent { tabular, computed,
followed: Tabular }` inside the table wherever the rule's answer differs
from the table's: the table wins, because it is what the country's
calendars say, and the disagreement is reported. The set of divergent
dates is generated when the extension is built, checked in as a
fixture, and published in the docs: the first public record of where
computation and the official almanac disagree.

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

**To fixed.** Look the year up in the table or the extension; the
year's start day is precomputed (`year_starts`); add the lengths of the
months before the date's month and the day less one. **From fixed.**
Binary search `year_starts` for the year, then walk at most twelve
month lengths. Both are integer arithmetic; the extension's lengths are
data, so no astronomy runs at conversion time.

**Building the extension.** For each year outside the table, compute
the twelve sankranti instants (the sidereal Sun entering each sign) from
the `SankrantiSource` under the stamped frame, convert each to a civil
day at Kathmandu under Nepal's offset history (local mean time before
1920, +5:30 to 1986, +5:45 after), apply the `MonthStartRule`, and take
the differences as month lengths; year Y starts at the Mesha sankranti
of AD Y − 57. The source and the rule are chosen by measurement: each
candidate is run over the table's own span and the pair that reproduces
the most official month lengths is the default, its miss rate, its
cumulative drift and the divergent dates published. The baseline
engine's generator measured Surya Siddhanta with a 0.705 threshold at
87.0 % of month splits, 61 of 126 year totals, and a running day count
that never leaves ±1 day; those are the numbers to beat. The extension is generated by `cargo xtask gen calendars`
and checked in with its frame stamp, so a consumer's conversions are
deterministic without an ephemeris and the generating settings are on
record.

## 5. The API

Through the `Calendar` trait, registered under `BIKRAM_SAMBAT`, plus
`bikram_sambat::table_edition()`, `bikram_sambat::coverage() ->
(tabular range, computed range)` and `bikram_sambat::divergences() ->
&[Divergence]`. C ABI: the generic calendar entry points; the
resolution crosses as a typed enum with its fields. Bindings: `bs(2072,
1, 1)` constructors validating month and day against the table, and the
resolution as a discriminated union on every converted date.

## 6. Errors and degenerate states

A day beyond the month's length for that year: `INVALID_ARG` with the
length. A year outside both ranges: `OUT_OF_RANGE` with the coverage.
A divergent date is a state, not an error. A request for the computed
extension when no ephemeris is available is impossible by design: the
extension is data.

## 7. Performance budget

| operation | budget |
|---|---:|
| to or from fixed | 50 ns, no allocation |
| table size | about 1.6 KB official, 7 KB computed, 5 KB of year starts |
| building the extension (offline) | seconds per century |

## 8. Tests

- Exhaustive round trip over the whole span (1856 to 2457, about
  220 000 days), never sampled.
- Every month length in the table against the authority's publication
  (the memo's R2), and the year lengths (365 or 366) against the
  Gregorian days between consecutive Baisakh firsts.
- Known dates: the epoch (1 Baisakh 1970 is 13 April 1913), the New Year
  dates of the last decade from the official almanac, and the fixture
  charts of spike 1 with their BS dates.
- Every rare event as a fixture: each 32-day and 29-day month in the
  table, a sankranti within minutes of the day boundary in the
  extension, both sides of Nepal's 1986 zone change (the civil day is
  unaffected; the test proves it).
- The divergence set is a fixture; a change in it after regenerating
  the extension requires a memo update and a calculation version entry.
- Differential: the baseline engine's computed years (rank 2) against
  the extension, differences explained in the deliberate-difference
  registry.

## 9. Localisation

`sdk.calendar` holds the month names (Baisakh to Chaitra in the four
launch languages), the `गते` date pattern, Devanagari numerals through
the locale's numbering system, and the era names.

## 10. Open questions

1. The rule and the frame the official almanac follows: the memo's R1
   to R3 (the committee's publications and stated method) answer both;
   until then the baseline engine's computed table ships as rank-2 data
   with its stamp, and the SDK's own engine (the `siddhanta` crate for
   the Surya Siddhanta Sun, the ephemeris port for drik, the rule rows,
   the fit harness) is the next calendar work.
2. The Buddha era's new-year rule in Nepali usage.
3. Nepal Sambat's tabular stopgap until the lunisolar calendar exists.
