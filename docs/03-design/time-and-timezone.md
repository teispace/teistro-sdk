# Time and time zones

Status: `draft`, written 2026-09-05 as a Phase 1 design page and revised
the same day when `crates/time` and `crates/port-timezone` were built,
and again when the ephemeris port was promoted: Delta T moved to
`crates/astro`, the local day took the rise and set solver through the
drik solar model and carries its sunrise convention, and the `SUNRISE`
unknown-time fallback resolves; and again when the planetary hours and
the port's DUT1 were added. Derives from
`02-architecture/04-calendar-time-architecture.md`,
`01-research/platform/05-calendars-timezones.md`,
`01-research/platform/13-astronomy-layer.md` (time scales), ADR-0020
(what the envelope stamps about time), spike 3's Delta T finding
(`spikes/03-ephemeris-port/README.md`) and spike 1's fixtures (zone
resolution at Nepal's 1986 change, the New York fold, local mean time),
which the crate reproduces (`crates/time/tests/fixtures.rs`). The
baseline engine's timezone resolver and its replay-safe metadata are the
rank-2 reference for the resolution shapes.

What the built crate differs in from the page as first written, each
with its reason:

- Delta T's model either side of the IERS table is Espenak and Meeus
  (2006), with the standard errors of Morrison and Stephenson (2004) as
  the uncertainty; Stephenson, Morrison and Hohenkerk (2016) is
  registered and refused as unsourced until its spline coefficients are
  cited (cruxes register C32). The IERS series carries UT1 only from
  1956, so the table begins there. Delta T and the UT1 to TT
  conversions live in `crates/astro` (`delta_t`, `scale`), where the
  frame completion and the solvers need them; `teistro_time` re-exports
  them, and the data file is `crates/astro/data/delta-t.json`.
- A zone's era is decided against the offsets the zone applies in the
  database's own year, never against the offset in force when the
  software runs (no clock reads in a computation crate, ADR-0022); the
  baseline compared with the offset at export time, a deliberate
  difference (C33, `fixtures/README.md` convention eleven).
- The local day takes any `SolarModel` from the calendar crate for its
  arc: the Surya Siddhanta, or modern astronomy through the ephemeris
  port (`DrikSun`, which reads the sunrise convention knob and runs
  `astro`'s rise and set solver); every model states its convention and
  the local day carries it.
- Instants are `f64` Julian days, about fifty microseconds of resolution
  in the present era; ghati-pala snaps to a tenth of a millisecond and
  is exact on that grid.
- A civil 23:59:60 is accepted only where the leap-second table has one
  and folds onto the following midnight, stamped.

## 1. Purpose and scope

Every chart begins with "when": a civil date and time at a place,
resolved to an instant on a named time scale, with enough recorded to
reproduce the resolution after the zone database changes. This page
settles the time types in `core`, the scales and their conversions
(Delta T, leap seconds), zone resolution with its policies and metadata,
local mean time, the sunrise-anchored local day, ghati-pala, and the
time facts a result stamps. Rise and set solving lives in `astro`;
civil-to-day-number arithmetic lives in `calendar`; this page consumes
both.

## 2. Inputs, settings and ports

Inputs are a `CivilDateTime` in a calendar, a `ZoneSpec`, a place when
the day is sunrise-anchored, and the settings knobs `dst_gap`,
`dst_overlap`, `unknown_time`, `delta_t`, `sunrise`, `day_boundary`,
`polar_day_policy` and `ghati_reckoning` (`settings-and-profiles.md`).
Ports: `TimeZoneProvider` (the embedded tzdb by default, replaceable at
context creation; versioned) and the ephemeris port through `astro` for
sunrise. The geo port (coordinates to zone) is optional and never
consulted implicitly: a zone is given or it is local mean time.

## 3. The data model

### 3.1 Scales and instants

```rust
pub struct JulianDay<S: Scale>(f64);             // in core; S is Ut1, Tt or Utc
pub struct SplitJulianDay<S> { day: i64, fraction: f64 }   // for sub-millisecond work in astro
pub enum DeltaTModel { TableThenModel, EspenakMeeus2006, StephensonMorrisonHohenkerk2016, IersTable(TableVersion), Provider, Custom(Seconds) }
pub struct DeltaT { seconds: f64, model: DeltaTModel, uncertainty_seconds: Option<f64> }
pub struct LeapSeconds { table_version: TableVersion, /* (utc_jd, tai_minus_utc) rows */ }
```

Conversions are explicit functions, never `From`: `tt_from_ut1(jd,
delta_t)`, `ut1_from_tt`, `utc_from_ut1` and back. UTC to UT1 assumes
DUT1 is zero (under 0.9 s by definition, a hundredth of an arcsecond of
lunar motion) and stamps `dut1_applied: 0`; `ut1_from_utc_with` and
`utc_from_ut1_with` take an ephemeris provider and apply the DUT1 it
declares (the port's `DUT1` override, from an engine with IERS
bulletins), checked against the 0.9 s bound and stamped as applied. UTC
before 1972 is treated as UT1 and stamped `proleptic_utc`. TT is never
exposed to a consumer as an input; every request instant is civil or
UTC.

Delta T, per spike 3: a table of the measured values to the present
(the IERS series, shipped with the data packs and versioned) and a
model on either side (Stephenson, Morrison and Hohenkerk 2016 with its
long-term parabola), the Espenak and Meeus fit only as a selectable
model for reproducing older software; a provider's native Delta T is an
override under `PREFER_NATIVE`. The value and the model are stamped;
beyond the modelled era the stamp carries `uncertainty_seconds`, and a
result whose instant is uncertain by more than a minute carries
`time_uncertainty` in the envelope so a consumer can say so.

### 3.2 Civil time and zones

```rust
pub struct CivilDateTime { calendar: CalendarKey, date: CalendarDate, time: Option<CivilTime> }
pub struct CivilTime { hour: u8, minute: u8, second: u8, nanos: u32 }
pub enum ZoneSpec { Iana(ZoneId), LocalMean { longitude: Longitude }, Fixed { offset_min: i16 } }
pub enum DstPolicy { gap: GapPolicy, overlap: OverlapPolicy }
pub struct ZoneResolution {
    offset_min: i16,
    source: ZoneSource,               // Iana | Lmt | Manual
    era: ZoneEra,                     // Current | Historical | BeforeRules (tzdb's LMT stub)
    tzdb_version: String,
    dst: DstOutcome,                  // None | Gap { shifted_by_min } | Overlap { chosen: Earlier | Later }
    time_known: bool,                 // false when a fallback supplied the time
    warnings: Vec<Warning>,           // OFFSET_DIFFERS_FROM_CURRENT_RULES and the like
}
pub struct Resolved { instant: JulianDay<Utc>, zone: ZoneResolution, civil: CivilDateTime }
```

A stored chart keeps `ZoneResolution` beside the instant. Replaying it
under a newer tzdb reproduces the same instant because the offset that
was applied is stored, and the difference against the newer rules is
reported, not silently applied: the baseline engine's model, kept.

### 3.3 The local day and ghati-pala

```rust
pub struct LocalDay {
    place: Place,
    date: CalendarDate,               // under the profile's day boundary
    sunrise: JulianDay<Utc>, sunset: JulianDay<Utc>, next_sunrise: JulianDay<Utc>,
    state: DayState,                  // Normal | Polar { kind: Day | Night, policy applied }
    convention: SunriseConvention,    // what the model reckoned the arc by
    model: String,                    // the model's stamp
}
pub struct GhatiPala { ghati: u8, pala: u8, vipala: u8 }   // 0 ghati 0 pala at sunrise
pub enum Reckoning { Civil, Proportional }
pub struct Hora { number: u8 /* 1 to 24 */, lord: Graha, start: JulianDay<Utc>, end: JulianDay<Utc> }
pub enum hora::Reckoning { Proportional, Equal }   // from the `hora_reckoning` knob
```

The local day carries its `vara` (the weekday of the sunrise-anchored
day), which the planetary hours start from.

Civil reckoning: a ghati is 24 minutes, a pala 24 seconds, a vipala 0.4
seconds, counted from sunrise. Proportional: thirty ghatis span the
actual day from sunrise to sunset and thirty the night to the next
sunrise. Conversion in both directions is exact integer arithmetic on
microseconds, the vipala floored, so that sunrise is 0-0-0 and the
instant one vipala later is 0-0-1; the baseline engine's rounding is
confirmed against its fixtures before the rule is pinned (open
question 2).

## 4. Algorithms

**Zone resolution.** For `Iana`, ask the provider for the offsets at
the civil time: one candidate is the answer; two (an overlap) are
resolved by `dst_overlap`; none (a gap) by `dst_gap` (`ERROR` returns
both surrounding offsets in the error; `SHIFT_FORWARD` adds the gap and
stamps it). For `LocalMean`, the offset is the longitude times four
minutes, computed as `round(longitude_deg × 240) seconds` from the
validated longitude, stamped `Lmt`. For `Fixed`, the offset is taken
and stamped `Manual`. Before a zone's first rule the provider returns
tzdb's LMT stub and the era is `BeforeRules`; a resolution whose offset
differs from the zone's current rules is stamped `Historical` with the
baseline's warning. An absent time applies `unknown_time`: `REFUSE` is
an error naming the fallbacks; `NOON` and `MIDNIGHT` supply it and set
`time_known: false`, which every downstream result inherits; `SUNRISE`
needs the place and a solar model, so `resolve_at_place` takes a
`DayContext` (model, place, polar-day policy), computes the local day of
the date at the place through a clock read from the zone provider, makes
its sunrise the instant, and reports that instant's civil time with
`time_known: false` and the fallback warning; the plain `resolve` refuses
with a hint naming the way through.

**Instant to civil.** UTC to a fixed day and a time of day through the
calendar; the zone offset applied at the civil boundary; the day
boundary knob decides which calendar date the instant belongs to
(`SUNRISE` needs the local day; `SUNSET` and `NOON` likewise).

**The local day.** Sunrise and next sunrise from the solar model: the
Surya Siddhanta's arc, or `astro`'s rise and set solver under the
convention (centre without refraction; upper or lower limb with
refraction; a custom altitude) through `DrikSun`, with the provider's
native rise and set as an override under `PREFER_NATIVE` when the
context wires it (`astro-events-and-crossings.md`). Above the polar
circles the solver reports no event; `polar_day_policy` then yields an
`undefined` state (`UNDEFINED`), the nearest event of the right kind
(`NEAREST_EVENT`, stamped as a convention), or civil midnight
(`CIVIL_MIDNIGHT`, stamped). The baseline engine synthesises polar days;
the SDK reports what it did.

**Planetary hours.** Twenty-four horas from sunrise, each ruled by a
graha: the first by the day's vara lord, each next by the lord of the
sixth weekday on, which is the Chaldean order (Sun, Venus, Mercury,
Moon, Saturn, Jupiter, Mars). Under `PROPORTIONAL` twelve horas divide
the day from sunrise to sunset and twelve the night to the next sunrise;
under `EQUAL` each is sixty minutes from sunrise. `horas(day, reckoning)`
lists the twenty-four; `hora_at(day, instant, reckoning)` finds the one
holding an instant and refuses one outside the day, naming the day's
span. The baseline engine reckons proportionally (its fixtures decide,
convention thirteen in `fixtures/README.md`).

**Ghati-pala.** `elapsed = instant − sunrise` in microseconds; civil:
`vipala = elapsed / 400 000`; proportional: `vipala = elapsed × 108 000
/ span` where `span` is the day or night length in microseconds and the
division is integer; then `ghati = vipala / 3600`, `pala = vipala / 60 mod
60`, `vipala mod 60`. The inverse multiplies back and lands on the
sunrise-anchored instant to the microsecond.

## 5. The API

Rust, as built: `resolve(&CivilDateTime, &ZoneSpec, &Policy, &dyn
TimeZoneProvider) -> Result<Resolved>` and `resolve_with(&dyn
CalendarSystem, ..)`; `civil_of(instant, &ZoneSpec, provider)` and
`civil_of_with(calendar, ..)`; `resolve_at_place(.., DayContext { model,
place, polar_day_policy })` and `resolve_at_place_with(calendar, ..)` for
the `SUNRISE` fallback; `date_of(instant, &dyn LocalClock, calendar)`;
`local_day(&dyn SolarModel, calendar, &dyn LocalClock, &Place,
&CalendarDate, PolarDayPolicy)`; `ghati_pala(&LocalDay, instant,
Reckoning)` and `instant_of(&LocalDay, GhatiPala, Reckoning)`;
`horas(&LocalDay, hora::Reckoning)` and `hora_at(&LocalDay, instant,
hora::Reckoning)`; `delta_t(jd_ut1, DeltaTModel)`, `tt_from_ut1`,
`ut1_from_tt`, `ut1_from_utc`, `utc_from_ut1`, `ut1_from_utc_with` and
`utc_from_ut1_with` (over a provider's DUT1), `tt_from_utc`,
`utc_from_tt` and `stamp` for the envelope; `EmbeddedTzdb::shared()` as the default provider and
`EmbeddedTzdb::clock(name)` for a zone as a `LocalClock`; `zones::nepal()`.
The port (`crates/port-timezone`): `TimeZoneProvider` with `version`,
`has_zone`, `zones`, `offset_at`, `candidates` and `current_offsets`.
C ABI:
`ts_time_resolve`, `ts_time_civil`, `ts_time_convert` and
`ts_time_delta_t` are built (`ffi-abi-and-api-description.md`: the civil
date-time and the zone as `struct_size` structs, the resolution with the
offset, source, era, database version, abbreviation, what the policy did
and the warnings as a bit set); `ts_time_local_day`, `ts_time_ghati` and
batch forms over arrays of civil
times or instants with capacities follow. Bindings: `Resolved` and
`ZoneResolution` as readonly typed objects; `ZoneSpec` as a tagged
union; `CivilDateTime` constructed only through validating builders;
`JulianDay` branded per scale so a UT1 value cannot be passed as TT.

## 6. Errors and degenerate states

| situation | outcome |
|---|---|
| a civil time in a DST gap under `ERROR` | `INVALID_ARG`, detail `DST_GAP`, both candidate offsets |
| an unknown zone id | `INVALID_ARG` with the nearest ids by prefix |
| an instant before tzdb's first rule | resolves under the LMT stub, era `BeforeRules` |
| an instant beyond the Delta T table | modelled, `uncertainty_seconds` stamped; never an error |
| no sunrise at the place and date | `DayState::Polar` with the policy applied, or `undefined` |
| a time absent under `REFUSE` | `INVALID_ARG`, detail `TIME_UNKNOWN`, the fallbacks named |
| a second of 60 (a leap second) | accepted, converted through the table, stamped |

## 7. Performance budget

| operation | budget | measured (`cargo bench -p teistro-time`, Apple M-series, 2026-09-05) |
|---|---:|---:|
| zone resolution through the embedded tzdb | 20 µs, one allocation for the version string | 238 ns |
| zone resolution, local mean time | 1 µs | 62 ns |
| the civil time of an instant in a zone | 20 µs | 299 ns |
| Delta T from the table | 200 ns | 6 ns |
| Delta T from the model (1850) | 500 ns | 198 ns |
| TT from UTC through the leap-second table | 100 ns | 4 ns |
| ghati-pala either direction | 100 ns, no allocation | 10 to 23 ns |
| local day | the sunrise solver's budget (two arcs) | 1.4 µs with the Surya Siddhanta's arc |

## 8. Tests

- Spike 1's fixtures (`tests/fixtures.rs`): the 55 charts' civil times
  resolve to the baseline's instant within a second (within a minute for
  the local-mean-time charts the baseline rounds), with its offset,
  source and era, including both sides of Nepal's 1986 change, the New
  York and São Paulo folds under both choices, the stubs before a zone's
  first rule, and local mean time; the one era the baseline labels by
  its export-time offset is named as the deliberate difference.
- Property: for every instant and zone in a corpus, `resolve(civil(instant))`
  returns the instant except inside a gap or an overlap, where the
  policy's documented outcome holds.
- Leap seconds: the table's rows against the IERS list; a `60` second
  converts; UTC before 1972 is proleptic and stamped.
- Delta T: continuity within 0.5 s across the table-to-model seam;
  the spike 3 kit's bound inside the measured era against both engines;
  the stamp names the model.
- Ghati-pala: exhaustive round trip over a day at microsecond steps
  in both reckonings; the sunrise instant is 0-0-0.
- Planetary hours (`tests/hora_fixtures.rs`): the lord at the birth
  instant matches the baseline's for every fixture chart under the
  proportional reckoning except the three the baseline's day-early block
  or its synthesised polar day decides (c022, c028, c039); the equal
  reckoning disagrees on fewer than thirty, which shows the fixtures
  distinguish the two; the lords follow the Chaldean order from the vara
  lord; the horas tile the day exactly.
- DUT1: a provider that declares the override moves UT1 by its value and
  the stamp says so; a value outside 0.9 s is refused as the provider's
  error naming the field.
- Polar: the fixtures from the baseline's synthesised days become
  `NEAREST_EVENT` cases with the convention asserted in provenance.
- The tzdb version stamp changes when the embedded database is updated;
  a replay under a newer tzdb reproduces the stored instant and reports
  the difference.

## 9. Localisation

`sdk.time` holds the state and warning keys (`polarDay`, `dstGap`,
`timeUnknownFallback`) and `sdk.calendar` the unit names (ghati, pala,
vipala) and the zone-source labels.

## 10. Open questions

1. Closed: DUT1 comes from a provider that declares the port's `DUT1`
   override (`ut1_from_utc_with`); without one `ut1_from_utc` applies
   zero and says so. No shipped provider declares it yet.
2. The ghati-pala rounding at the sunrise boundary is pinned by the
   baseline's fixtures when the harness runs; floor on a tenth-of-a-
   millisecond grid is the working rule.
3. The `polar_day_policy` default for `nepali-default` is `NEAREST_EVENT`
   to match the baseline's behaviour; the maintainer confirms.
4. Stephenson, Morrison and Hohenkerk (2016): the spline coefficients to
   cite (C32); until then the knob value is refused as unsourced.
5. Closed: the `SUNRISE` unknown-time fallback resolves through
   `resolve_at_place` with a `DayContext` (model, place, polar policy).
