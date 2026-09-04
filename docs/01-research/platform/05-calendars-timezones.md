# Calendars and time zones

Status: `research`, 2026-09-04. Feeds Q9 and
`02-architecture/04-calendar-time-architecture.md`. Sources: the baseline engine's
calendar and timezone code, ICU4X `icu_calendar` (Gregorian, ISO, Buddhist,
Chinese, Dangi, Coptic, Ethiopian, Hebrew, Indian, Japanese, Julian, Persian,
Hijri civil, tabular, Umm al-Qura and observational, ROC), the
`calendrical_calculations` crate (Reingold and Dershowitz algorithms in
Rust), Time4J's calendar list, the IANA tz database.

## Calendar inventory

| calendar | kind | algorithm | needs ephemeris | baseline | tier |
|---|---|---|---|---|---|
| Gregorian, proleptic Gregorian, Julian, mixed (Julian before 1582-10-15) | arithmetic | standard | no | Gregorian | P0 |
| ISO week date | arithmetic | standard | no | no | P0 |
| Bikram Sambat (BS) | table for the official span; astronomical solar months (sankranti with the local convention) outside it | table plus computed extension with a `source` flag | computed part yes | yes | P0 |
| Nepal Sambat (lunisolar, Newar) | lunisolar with the local Kartik new year; published tables | table-first, computed later | yes | era number only | P1 |
| Indian lunisolar (Vikram, Shaka Amanta and Purnimanta): tithi-based days, adhika and kshaya masa, month names, year boundaries by region (Chaitra, Kartika, Ashadha) | astronomical | new-moon and sankranti crossings from the ephemeris | yes | month systems yes; full calendar no | P0 (the panchanga needs it) |
| Indian national (Saka) | arithmetic | ICU4X | no | era number only | P1 |
| Tamil solar, Malayalam (Kollam), Bengali (Bangabda), Odia, Punjabi (Nanakshahi), Assamese | solar with regional sankranti rules | astronomical or tables | yes | no | P1 |
| Hijri (civil, tabular, Umm al-Qura, observational) | lunar | ICU4X | observational yes | no | P1 |
| Hebrew | lunisolar arithmetic | ICU4X | no | no | P1 |
| Persian (Solar Hijri) | arithmetic (or observational) | ICU4X | no | no | P1 |
| Chinese lunisolar, Dangi (Korean), Vietnamese | astronomical | ICU4X (Chinese, Dangi) | yes | no | P2 |
| Tibetan | lunisolar with skipped and doubled days | Henning's tables and algorithms | partial | no | P2 |
| Buddhist era, Japanese eras, ROC, Ethiopian, Coptic | arithmetic | ICU4X | no | no | P2 |
| Burmese, Thai lunar | astronomical variants | tables | yes | no | P2 |

Eras and cycles that ride on calendars: Vikram, Shaka, Kali, Nepal Sambat,
Buddha, Kollam, Bengali; the 60-year Jovian samvatsara (north and south
schemes); the 60-day and 60-year sexagenary cycle (Chinese).

## The calendar port

Following ICU4X's `Calendar` trait: a calendar converts between its own
(year, month, day, era, leap flags) and a fixed day number, exposes month
lengths and leap rules, formats through the locale layer, and declares its
supported range and whether it depends on the ephemeris. Lunisolar
calendars take the ephemeris port as a dependency and cache the
astronomical events they need (new moons, sankrantis). The Indian lunisolar
calendar is where the panchanga and the calendar meet: a civil date in that
calendar is a tithi at sunrise at a place, so the calendar is
location-dependent and the API must say so.

## Time and time zones

| need | approach |
|---|---|
| Julian day and timescales (UT1, TT, UTC with leap seconds), Delta T | SDK utilities; Delta T from the ephemeris port (the baseline engine) or embedded tables (Teimeris has five models); the port supplies it |
| IANA zone from a zone id at an instant, with historical rules | embedded tzdb (the `jiff` crate bundles it and reads system zoneinfo; `chrono-tz` and `tzdb` crates exist); versioned; updatable at runtime through a timezone provider port |
| zone from coordinates | geo-tz-class data is large (tens of MB); provided through the geo port, with a default package (`teistro-geo`) that ships the shapes, never in the core |
| local mean time (LMT) | longitude × 4 minutes; a first-class `tzSource` |
| DST gaps and ambiguities | policy on the request: error on gap (the baseline engine), or shift forward; ambiguity earlier (the baseline engine) or later; recorded in the result |
| replay-safe metadata | offset minutes, source (`iana`, `lmt`, `manual`), era (`current`, `historical`, `lmt`), tzdb version, so a stored instant can be reproduced after a tzdb update (the baseline engine's model, kept) |
| ghati-pala and the sunrise-anchored day | SDK utilities over the sunrise service |
| planetary hours (hora) and choghadiya | panchanga module |

## Known the baseline engine calendar defects to avoid

- Era numbers from fixed offsets (Kali and Nepal Sambat wrong by
  thousands and hundreds of years respectively in the date-conversion
  service; year-precise offsets in the panchanga payload).
- BS outside the official table treated as "computed" by a simple
  extension; the SDK states the algorithm and its validation against
  published almanacs.

## Recommendation

- Build the calendar port on the ICU4X `Calendar` shape and use
  `icu_calendar` and `calendrical_calculations` for the arithmetic
  calendars (P1) rather than re-implementing them.
- Implement BS (table plus computed), the Indian lunisolar calendar and the
  eras in the SDK's own crates because no library has them.
- Bundle tzdb through the timezone provider with a version stamp; make the
  provider replaceable so a consumer can supply a newer tzdb without an
  SDK release.
