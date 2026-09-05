# Status

The living tracker. Read this first in any session; update it before ending
one. It answers four questions: what is done, what is being done now, what
comes next, and what happened in each session.

**Project phase:** Phase 0 exited on 2026-09-05 (decisions made, four
spikes measured, repository live); Phase 1, Foundation, begins with its
design pages.
**Repository:** https://github.com/teispace/teistro-sdk (public,
Apache-2.0, created 2026-09-04). `main` is protected: pull requests with
the `fast-check` status, linear history. Changes land by branch, pull
request (the `dco` and `fast-check` jobs), rebase merge.
**Last updated:** 2026-09-05, end of the twenty-third session (the
sidereal time moved to the IAU 2006 expression `gst06b`, held to
Teimeris within 0.0012″ strictly inside its 1850 to 2050 window by
`tests/teimeris_sidereal.rs`; F1 measured beyond the window, F6 filed;
`astro-events-and-crossings.md` §4); before that the twenty-second
session (the `CROSSINGS` override with its vtable slot and kit checks,
18 checks against Teimeris); before that the sixteenth session (the
twenty-two house systems in `astro::houses` with the auxiliary points
and the polar policies, within 5e-6° of Teimeris at ten latitudes and
0.0002° of the baseline's 55 charts; `astro-house-systems.md`); before
that the fifteenth session (Phase 2's
astronomy begun: `astro::precession` as a catalogue of four models over
new ERFA ports, `astro::ayanamsha` computing every epoch-defined member
within 1e-7″ of Teimeris over 1044 recorded rows, the completion
completing the sidereal zodiac from the SDK's catalogue, the design
pages `astro-timescales-and-frames.md` and
`astro-ayanamsha-catalogue.md`); before that the fourteenth session (the
national panchanga committee's 2082 and 2083 panchangas obtained from
`npns.gov.np` through the browser and read into
`fixtures/official/npns-2082-2083.json`: the SDK's engine reproduces all
24 sankranti instants within 1.6 minutes and every month start; the
committee's Sun is the text's within 3″, its Moon the text's with four
revolutions fewer on the apsis, its star planets modern positions in
the Lahiri frame, its sunrise modern under a convention of its own; R2
closed for the method, C30 explained, C38 and C39 opened); before that
the thirteenth session (the
`siddhanta` crate completed to the text with the sighra daily motion,
the latitudes and the Lagna, each reproduced against Burgess's worked
1860 computation, and presented behind the ephemeris port as a classical
astronomy that passes the kit; the port's `Astronomy`, `SpeedModel`,
`DistanceUnit` and `DUT1` override; the completion's ordering of the
zodiac shift and the rotation; the planetary hours in `time` under the
`hora_reckoning` knob, decided by the baseline's fixtures; UT1 from a
provider's DUT1).

## How to resume

1. Read this file, then `QUESTIONS.md` (every decision, one open question).
2. The local checkout is the repository root; `cargo xtask check-docs`
   and `cargo deny check` must pass before any commit; commits are
   signed off (`git commit -s`) with Conventional Commits subjects; the
   clean-room policy (`CLEAN_ROOM.md`) is binding. A discrepancy traced
   to the reference engine is measured, filed as an issue in
   `teispace/teimeris` with its reproduction and assigned to the engine's
   maintainer, and entered in `05-testing/02-engine-findings.md` with
   the bound the SDK holds it at meanwhile (the maintainer's rule,
   2026-09-05).
3. The next task is Phase 2's close: the items under "Now". `crates/core`, `crates/port-ephemeris`,
   `crates/astro`, `crates/ephemeris-kit`, `crates/siddhanta`,
   `crates/calendar` (with the Bikram Sambat engine and the drik model),
   `crates/time` and `crates/port-timezone` exist; `cargo xtask
   check-catalogue`, `check-calendars`, `check-time` and `check-accuracy`
   (the accuracy document, `05-testing/ACCURACY.md`) are gates; the
   conformance kit runs in `cargo test -p teistro-ephemeris-kit`;
   `cargo xtask calendars bs-fit` (and `--detail`) is the measurement
   behind the shipped Bikram Sambat rule (`docs/calendars/bikram-sambat.md`);
   `gen time` rebuilds the Delta T tables (`crates/astro/data/`) and the
   leap seconds (`crates/time/data/`). The adapters under `adapters/`
   are outside the workspace and need the engines locally
   (`TEIMERIS_LIB_DIR`, `SWEPH_SRC_DIR`; `adapters/README.md`): their
   kit binaries, the Teimeris `bs-fit` binary (the drik comparison) and
   the Teimeris fixture test (the solver against the 55 charts) are run
   by hand.
   Spike 1 is done: its fixtures are in
   `fixtures/baseline/` (read `fixtures/README.md` before touching them;
   `cargo xtask check-fixtures` is their gate); the export script lives
   inside the baseline engine's own repository, in that repository's
   language, uncommitted there unless the maintainer asks, and is run
   again only for a deliberate corpus version bump. Spike 2 is done: the
   toolchain is option A (ADR-0007) and `spikes/02-binding-toolchain/`
   holds the model for the Phase 1 extractor and generators. Spike 3 is
   done: `spikes/03-ephemeris-port/port` is the model for the Phase 1
   `ephemeris` module and its kit; the two adapters there are standalone
   crates outside the workspace (ADR-0019) and need the engines locally
   (`TEIMERIS_LIB_DIR`, `SWEPH_SRC_DIR`; see the spike's README). Spike 4
   is done: `spikes/04-teistro-intl/intl` is the model for the Phase 1
   `intl` crate and the `teistro-intl` CLI; its harnesses need Node with
   `npm install` and Dart with `dart pub get` (see the spike's README).

## Done

- 2026-09-04: In-depth analysis of the baseline engine's backend (the reference and
  minimum bar). Recorded in `01-research/baseline-engine/`.
- 2026-09-04: Surface read of Teimeris (the default native ephemeris and
  the model for rigour and generated bindings).
- 2026-09-04: Web research on competitor feature sets and platform
  technology (Diplomat, UniFFI, ICU4X, MessageFormat 2, docs frameworks,
  WebAssembly, next-intl, slang, VSOP87 and ELP theories, Astronomy
  Engine, Swiss sidereal modes).
- 2026-09-04: Documentation written and revised: 86 pages (vision,
  research, architecture, roadmap, decisions, guidelines, living files).
- 2026-09-04: Twenty-five questions put to the maintainer and all
  decided; fifteen ADRs, fourteen accepted (0007, the binding generator,
  waits for the spike).
- 2026-09-04: The quality bar made binding (`05-testing/01-quality-bar.md`,
  ADR-0015) and landed through pull request #1, which proved the `dco` and
  `fast-check` path; `main` is at two commits.
- 2026-09-04: Open-source scaffolding: Apache-2.0 licence, `NOTICE`,
  `DCO`, Contributor Covenant 2.1, security policy, governance,
  `CODEOWNERS`, issue and pull request templates, RFC process, the Rust
  workspace with the `xtask` crate holding the documentation gate
  (`cargo xtask check-docs`) and the DCO check (`cargo xtask check-dco`),
  and the `fast-check` workflow (format, lint, tests, gates). Repository
  created and configured.
- 2026-09-04 (second session): eight further decisions recorded and
  accepted (ADR-0016 exact classification and periods, 0017 kernel and
  table, 0018 evidence ranks and mark-and-continue, 0019 clean room and
  licence containment, 0020 calculation version and provenance, 0021 the
  reference-accuracy ephemeris path, 0022 the determinism contract and
  the conformance repository, 0023 type safety in every binding); five
  design pages written in Phase 0 because they are retrofit-hostile
  (`03-design/`: exact arithmetic, dasha kernels with 56 systems as rows,
  the varga kernel, strength schemes, the rules engine v2); the
  verification-cruxes register (26 open items); `CLEAN_ROOM.md`,
  `deny.toml` with `cargo deny` in the fast check, library lints, the
  forbidden-terms gate extended; principles 16 to 18; the roadmap,
  quality bar, data model, API conventions, binding, performance,
  calendar, localisation, extensibility, guideline and glossary pages
  revised accordingly; `08-adding-a-dasha-system.md` written.
- 2026-09-04 (third session): spike 1 done. The golden-vector export
  script written in the baseline engine's repository and run: 55 charts
  (48 chosen for zone, latitude, altitude and data-range hostility, 7
  placed by search to the second at classification boundaries in the
  topocentric frame), 115 fixtures under 13 settings profiles, 8.9 MB,
  in `fixtures/baseline/` with a manifest; `fixtures/README.md` (layout,
  provenance, the chart set, the profiles, the schema, ten baseline
  conventions for the deliberate-difference registry);
  `fixtures/tolerances.json` (provisional, keyed by field and provider
  class); `cargo xtask check-fixtures` in the fast check;
  `05-testing/01-golden-vectors.md` as the spike's result page; the
  roadmap, testing map, glossary, layout and the varga and dasha design
  pages updated.
- 2026-09-05 (fourth session): spike 2 done. The same slice (a context,
  settings, the ephemeris port as a host callback, one batch call
  returning a tree) built both ways under `spikes/02-binding-toolchain/`:
  option A as a designed C ABI, an extractor over its Rust source, and
  Rust emitters for the C header, the napi glue, TypeScript, the blob
  decoder and Dart, with hand-written ergonomic layers; option B as a
  Diplomat bridge generated into JavaScript (wasm) and Dart. Measured
  in Node and Dart: callbacks, marshalling of trees to depth 5, code
  volume, the typed surface. Decided as option A (ADR-0007, Q2): Diplomat
  0.16 cannot pass a host provider from JavaScript or Dart, marshals
  trees only through per-node accessors (an order of magnitude slower
  than the blob), and serves Node only through wasm. The maintainer's
  mandate of the day (type safe, DRY, clean, no repetition) recorded in
  the coding standards; the spike's emitters share one `common` module
  and the benchmarks one harness per language.
- 2026-09-05 (fifth session): spike 3 done. The ephemeris port built
  under `spikes/03-ephemeris-port/`: the model (frames packed to 32
  bits, columns instants outermost, a status and a source per cell,
  capabilities with content hashes, reserved error codes), the trait
  with positions required and three overrides, the `#[repr(C)]` vtable
  with `struct_size` handshakes and a bit-identical round trip, the IAU
  2006 obliquity and IAU 2000B nutation ported from ERFA (`NOTICE`
  updated), frame completion by policy with stamped steps, a
  thirteen-check conformance kit under one published set of bounds, a
  shared kit runner and timing helper, and two adapters outside the
  workspace: Teimeris (one context behind a mutex, the body-major grid
  transposed) and the Swiss Ephemeris (compiled from `SWEPH_SRC_DIR`,
  one process-wide lock, explicit flags, fallback reported as missing
  data, hashed data, a cross-thread stress test). All three providers
  pass the kit; the two engines give the same numbers on every check;
  the port costs 0.2 % over Teimeris's own call, the vtable 1.8 %, and
  completion 0.16 to 0.32 µs per cell. Finding: the SDK's Delta T fit is
  5 s high by 2025 against the engines' tables, so Phase 1's Delta T is
  a table plus a model. The design page
  `03-design/ephemeris-port-and-adapters.md` written; the architecture,
  roadmap, spikes index and testing pages updated.
- 2026-09-05 (sixth session): spike 4 done. Teistro Intl built under
  `spikes/04-teistro-intl/`: the stable `MessageFormat 2` grammar in
  full (data model, parser with offsets and the data-model checks,
  serialiser with a property-tested round trip), the `i18n/`
  conventions (metadata, namespaces, keys, entity records, source
  order), the engine with the SDK's functions (`:string`, `:integer`,
  `:number`, `:dms`, `:zodiac`, `:entity`, `:list`, `:msg`), ICU4X plural
  rules, numbering systems, fallback chains with provenance and a parse
  cache, the validator with twelve proven gates, the `.tpack` container
  (zero-copy, checksummed, hashed, carrying the locale metadata), typed
  accessor generators for TypeScript and Dart over one model, the CLI
  (`validate`, `build`, `gen`, `render`, `report`), 49 entities and 13
  messages in `en-Latn` and `ne-Deva-NP`, and harnesses proving both
  surfaces compile and reject wrong usages. Findings: the stable syntax
  differs from the draft the architecture quoted; no parameter sidecar
  is needed; entities select on their own gender; Nepali ordinals need
  exact keys; packs keep source text and the Phase 1 container bundles a
  locale's namespaces. The design page `03-design/intl-engine-and-packs.md`
  written; the localisation architecture, design index, spikes index,
  roadmap, adding-a-language guide and glossary updated. Phase 0's exit
  criteria met.
- 2026-09-05 (seventh session): the five Phase 1 foundation design
  pages written in `03-design/`: core types and the catalogue (forty
  kinds, keys and ids with their C packing, the catalogue sources and
  generator, the three rules separating facts from school choices and
  presentation, the quantity newtypes, closed unions per binding, the
  envelope and status codes, registries and limits); settings and
  profiles (the typed knob inventory, profiles as patches over a cited
  root, coherence rules, the canonical form and hash, five shipped
  profiles including `conformance-baseline` for the fixtures); time
  and time zones (scales, Delta T as a table then a model, zone
  resolution with replay-safe metadata, local mean time, the local
  day, ghati-pala as exact integer arithmetic); the arithmetic
  calendars (the fixed day, Reingold and Dershowitz, the mixed
  transition, ISO weeks, exhaustive and differential tests); Bikram
  Sambat (table plus computed extension with the month-start rule
  chosen by measurement, the divergence set as a fixture, eras by
  new-year rules, the source memo). The calendar and data-model
  architecture pages and the design index updated.
- 2026-09-05 (eighth session): `crates/core` built. `catalogue/`: 53
  kinds, 629 members, attributes and citations in YAML (the baseline
  engine's entity data at rank 2 plus the standard facts, each row
  marked), with a README; `cargo xtask gen catalogue` generates one
  enum per kind with stable ids, typed attribute tables, aliases, marks,
  sources, serde forms and resolvers, plus `catalogue.json` and the
  entity skeleton, and `check-catalogue` gates the output in CI. The
  crate: keys and packed ids with suggestions for wrong keys, validated
  quantities with compile-fail proofs (swapped place, wrong time scale,
  bare number, private constructor), `Nas` with property-tested exact
  classification, bounded `Ratio`, the status codes and a small `Error`,
  the provenance envelope, registries for open kinds, limits, and the
  settings module (twenty-six knob sets in thirteen groups, patches,
  five shipped profiles over a cited root, coherence rules, canonical
  JSON and hash). 29 unit tests, 9 doctests, 4 compile-fail tests, a
  criterion benchmark; every budget met except the settings hash (15 µs
  for a 2 KB document against 10 µs per KB, recorded).
- 2026-09-05 (ninth session): `crates/calendar` built from its two
  design pages: the fixed day with its Julian-day relation and weekdays;
  proleptic Gregorian and Julian in astronomical years; the mixed
  calendar with the 1582, 1752 and 1918 transitions and the gap refused;
  the ISO week date; the `CalendarSystem` trait (`date_of`, `fixed_of`,
  month lengths, leap years, conversion) and the shipped calendars by
  key; Bikram Sambat over the baseline engine's table (BS 1856 to 2457;
  official 1970 to 2100 stamped `Tabular`, the rest `Computed`),
  anchored on 1 Baisakh 1970 = 13 April 1913. Tests: every day of −9999
  to 9999 round-trips in each arithmetic calendar and agrees with the
  `calendrical_calculations` oracle; every day of the Bikram Sambat span
  round-trips; the anchors of 2072 and 2081 hold. The catalogue's `era`
  kind gained `COMMON_ERA` and `BEFORE_COMMON_ERA`. The source memo
  `docs/calendars/bikram-sambat.md` opened with what the baseline
  engine's generator established (Surya Siddhanta longitudes at
  Kathmandu with Nepal's offset history and a fitted 0.705 day cutoff
  reproduce 87 % of official month splits and never drift beyond a
  day) and the SDK's engine plan. The maintainer's mandate recorded: the
  SDK must compute Bikram Sambat from first principles for any year the
  way the Nepali panchanga does, so Nepal's panchanga makers can use it.
- 2026-09-05 (tenth session): the Bikram Sambat computation engine.
  `crates/siddhanta`: the Surya Siddhanta as the text prints it
  (Burgess, 1860), every number cited by verse; mean places in exact
  integer arithmetic from the text's own epoch (midnight at Lanka at the
  start of the Kali age); the sine table with the text's interpolation;
  the manda and sighra equations, the four steps, the true daily motion,
  the text's precession, declination and ascensional difference; a bija
  overlay with no shipped set (unsourced); the classical path uses no
  platform mathematics and is bit-identical everywhere (the Sun in 54 ns).
  `crates/astro` seeded with the shared boundary solver; `crates/time`
  seeded with a zone's offset history as a local clock and Nepal's rows
  from tzdb; `core::time` with `UtcOffset`, the `LocalClock` trait and
  local mean time; `core`'s `Divergent` resolution now carries both
  labels. `crates/calendar`: the `SolarModel` trait, the sankranti finder,
  the month-start rules as cited rows (Orissa, Bengal, Tamil, Malabar,
  the almanac day, the shift family, the Dharmasindhu's punya-kala), the
  engine (a year from a model, a clock, a place and a rule), the fit
  report, and the table regenerated for BS 1700 to 2500 by
  `cargo xtask gen calendars` from the official rows
  (`crates/calendar/data/bikram-sambat.json`) and the engine, held by
  `check-calendars` in CI; dates inside the official span report
  `Divergent` where the engine differs. The measurement
  (`cargo xtask calendars bs-fit`): under the text's Sun and Nepal's
  clock the civil-day rule reproduces every official New Year and year
  total and 90.1 % of month lengths, and the per-sankranti analysis
  showed the two ayana sankrantis follow the Dharmasindhu's punya-kala
  convention (Karka by the sunrise-to-sunrise day, Makara by sunset),
  which reproduces 1490 of 1512 month lengths (98.5 %), 116 of 126 years
  exactly, with no drift; the eleven residual boundaries lie within 25
  minutes of the rule's boundary. The source memo, the Bikram Sambat and
  time design pages, the new `siddhanta.md`, the module catalogue, the
  cruxes register (C27 to C31), the changelog and the glossary updated.
- 2026-09-05 (eleventh session): `crates/time` built from
  `03-design/time-and-timezone.md`, with `crates/port-timezone` as the
  zone database's contract. Scales: TT from UT1 and back through Delta
  T, UTC read as UT1 with DUT1 zero and stamped, UTC before 1972 as
  proleptic, TT from UTC through the leap-second table exactly; the
  envelope's time stamp filled from what was applied. Delta T: the IERS
  EOP C01 series (UT1 from 1956 to August 2026, 708 rows at a tenth of a
  year, fetched from the IERS with its provenance) interpolated where
  measured, Espenak and Meeus (2006) either side with the seam offset
  tapered and the end slope trusted for a decade, Morrison and
  Stephenson's (2004) standard errors as the uncertainty before the
  atomic era and a growing one after the table; Stephenson, Morrison and
  Hohenkerk (2016) registered and refused as unsourced (C32). Leap
  seconds: the IANA list (28 rows, expiring 2027-06-28) with a warning
  beyond its word; a civil 23:59:60 accepted only where the table has
  one, folded onto the following midnight. Zones: `ZoneSpec` (IANA, local
  mean time, a fixed offset), the gap and overlap policies from the
  settings, `ZoneResolution` with offset, source, era (current, earlier
  rules, before the zone's first rule, decided against the offsets the
  zone applies in the database's own year, never the clock), the
  database version, the abbreviation, what the policy did and the
  warnings; a stored resolution is itself a clock for replay; the
  embedded database is `jiff`'s bundled tzdb (2026c), never the host's,
  with suggestions for a misspelt zone. The local day from any
  `SolarModel` with the three polar policies; ghati-pala exact on a
  tenth-of-a-millisecond grid in both reckonings, every vipala of a day
  round-tripping. Tests: 55 fixture charts reproduce the baseline's
  instant (to the second; to the minute for its rounded local mean
  time), offset, source, era and warnings, with the five era labels the
  baseline took from its export-time offset named as deliberate
  difference eleven (C33). `cargo xtask gen time` and `check-time`;
  `core::settings` now exports its knob enums; the calendar's solar
  model reports polar days; the Bikram Sambat table regenerated under
  the tzdb-backed clock (same rows, the frame stamp naming the version).
- 2026-09-05 (twelfth session): spike 3's port promoted:
  `crates/port-ephemeris` (the model renamed into the catalogue, the
  rise and set override with the horizon convention as port vocabulary,
  the C vtable, the `.se1` scanner, the analytic test provider);
  `crates/astro` (Delta T and the UT1/TT scales moved in from `time`,
  the IAU routines ported from ERFA 2.0.1 with a provenance table and
  tested against ERFA's reference values, sidereal time and the
  obliquity, frame completion by policy, the boundary solver's
  `first_zero`, the rise and set solver by Meeus's iteration with a scan
  as the safety net); `crates/ephemeris-kit` (fifteen checks; the test
  provider in CI; Teimeris and the Swiss Ephemeris passing by hand);
  `DrikSun` as the calendars' second solar model; the local day's
  convention and `resolve_at_place` for the `SUNRISE` fallback; the
  adapters moved to `adapters/` with a Teimeris rise and set override
  and a `bs-fit` binary. Measured: the geometric sunrise within 0.13 s
  of Teimeris's search, the refracted one within 7.3 s at 64°N (the
  refraction convention, C34) and within 2.5 s of the baseline's
  fixtures below 60° (C35 names three fixtures a day early); modern
  positions reproduce 65 % of the official Bikram Sambat months under
  the shipped rule against the text's 98.5 %; the committee's own
  announcement names the Surya Siddhanta as its method (R1, in part).
  Design pages: `astro-events-and-crossings.md` written; the port,
  time and Bikram Sambat pages and the memo revised.
- 2026-09-05 (thirteenth session): `crates/siddhanta` completed to the
  text: the sighra daily motion (II.50 to 51), the latitudes (I.68 to
  70, II.56 to 58) and the Lagna from the oblique ascensions (III.42 to
  50), with Burgess's worked computation for midnight of 1 January 1860
  at Washington as rank-1 test vectors (the day count, the mean places,
  the precession, the true motions, the latitudes, the rising times and
  the horoscope point all reproduce; his printed Moon anomaly is a
  misprint his own table corrects); `SiddhantaProvider` behind the
  ephemeris port, declaring `Astronomy::Classical`, `SpeedModel::Rule`
  and `DistanceUnit::MeanDistances`, with the text's obliquity,
  ayanamsha and sunrise as overrides, passing the kit (`tests/kit.rs`).
  The port gained those three capability fields and the `DUT1`
  override; the kit gained `override_dut1` (sixteen checks), a
  second-difference continuity check for speeds by rule, informational
  rows for a classical astronomy (the obliquity 2065″ from IAU, the
  sunrise 250 s from hour-angle geometry, the speed rule 0.23° a day
  from the derivative; C36, C37), a skip when a provider refuses a
  horizon convention, and the text's ayanamsha against Burgess's
  20°24′39″; the completion applies the zodiac shift while the columns
  are ecliptic and asks the provider's own frame for apparent
  positions, so the classical provider runs the SDK's rise and set
  solver. `crates/time` gained the planetary hours (`horas`, `hora_at`,
  the `hora_reckoning` knob, proportional by default: the fixtures
  reproduce the baseline's lord for every chart but the three its
  day-early or polar blocks decide) and `ut1_from_utc_with` over a
  provider's DUT1, bounded at 0.9 s. R2 stayed open that session (the
  committee's site serves its yearly panchanga through scripts) and was
  closed the next through the browser. Design pages revised: siddhanta (§3 to §5, §7, §8,
  §10), time (DUT1, horas), the port (capabilities, completion, the
  kit's table with the text's column), settings, the module catalogue,
  the glossary, cruxes C36 and C37, fixtures convention thirteen.
- 2026-09-05 (fourteenth session): the committee's publications read
  as data. The 2082 and 2083 Rashtriya Panchangam PDFs (no text layer;
  read from page images) gave 24 sankranti instants, four rows of
  printed places at sunrise, 22 days of sunrise and sunset and eight
  tithi ends (`fixtures/official/npns-2082-2083.json`, with provenance).
  Against the SDK: every instant within 1.6 minutes and every month
  start by the shipped rule (including a Makara at 03:23 kept on its
  civil day); the Sun the text's within 3″ (a modern Sun is 5.5′ off);
  the Moon the text's with `Bija { moon_apsis: -4 }` within 0.5′ at ten
  printed points, no other Moon bija or the swapped epicycle convention
  fitting as well; the star planets and node modern positions in the
  Lahiri frame (Saturn, Jupiter and the node within 1′ to 11′ of
  Teimeris, Mars, Mercury and Venus within 7′ to 94′; the text's places
  2° to 11° away); sunrise and sunset modern, the committee's 1.8 to
  2.8 minutes later than the almanac's upper-limb convention at rising;
  the printed velantara is the text's equation of time (reproduced
  within 4 seconds). The eleven residual boundaries (C30) are thereby
  the earlier makers' decisions inside their tolerance, not a different
  rule: the text's arc in mean time removes one, the drik arc adds two.
  Tests: `crates/calendar/tests/official.rs`,
  `crates/siddhanta/tests/official.rs`. Memo R1 to R3 revised, cruxes
  C28 and C30 updated, C38 and C39 added, fixtures README `official/`.
- 2026-09-05 (fifteenth session): Phase 2's astronomy begun. ERFA ports
  added with their reference values: the IAU 2006 precession angles and
  matrices (`p06e`, `pfw06`, `fw2m`, `pmat06`, `bp06`, `bi00`), the
  long-term poles and matrices of Vondrák 2011 (`ltpecl`, `ltpequ`,
  `ltp`, `ltpb`) with the paper's own obliquity series (`ltpeps`, a
  microarcsecond at J2000), and the vector primitives.
  `astro::precession`: four models (Vondrák 2011 the default, IAU 2006,
  IAU 1976, Newcomb) with the obliquity each is consistent with; 142 ns
  a matrix. `astro::ayanamsha`: the forty-seven members as definitions
  (epoch and value, frame, anchor), the published construction with
  the fitted-model correction, mean and nutated values, custom linear
  definitions, the twelve anchored members refused by name; 0.58 µs a
  value. The completion completes a sidereal zodiac from the catalogue
  when the provider declares no override. Measured against Teimeris's
  recorded values (`fixtures/teimeris/ayanamsha.json`, 1044 rows from
  the adapter's new `ayanamsha-table` binary): TT-epoch definitions
  within 1e-7″ (bit-identical in most rows), UT-epoch definitions within
  2.1e-4″ (the two Delta T models in antiquity). Design pages written:
  `astro-timescales-and-frames.md` (the models, the completion steps
  built and designed), `astro-ayanamsha-catalogue.md` (every member with
  its source); the port page, module catalogue, fixtures README and
  astro README revised.
- 2026-09-05 (sixteenth session): `astro::houses`, the twenty-two
  catalogued systems as one construction (the ecliptic point of a great
  circle of a pole height meeting the equator at a right ascension) with
  the circles each picks: whole sign, equal (three forms), Vehlow,
  Porphyry, Sripati, Regiomontanus, Campanus, Topocentric, Alcabitius,
  Koch, Placidus (iterated), Meridian, Morinus, Carter, Horizon,
  Krusinski, APC, Pullen's two, Sunshine (Treindl's construction); the
  auxiliary points; the sign-based systems in the zodiac in use; the four
  polar policies with `Outcome::{Defined, Substituted, Clamped}`.
  Measured: within 4.8e-6° of Teimeris over 25 194 cusps and angles at
  ten latitudes from −66° to 80° (`fixtures/teimeris/houses.json`, the
  adapter's new `houses-table` binary; the polar substitutions agree
  row for row), and within 0.00021° of the baseline's 55 charts for all
  twenty-two systems between 1800 and 2200 (0.0033° beyond 2200, where
  the engine behind the baseline uses a long-term sidereal time;
  Sunshine 0.05° where the Sun barely rises). Design page
  `astro-house-systems.md`; the design index, module catalogue, settings
  page, fixtures README, astro README and CHANGELOG revised.

## Decided (all on 2026-09-04 unless dated)

Rust core with a C ABI. Generated bindings with a parity gate, generator
chosen by spike. v1.0 is baseline parity with Western and Hellenistic
designed in. Apache-2.0 open core. Teispace owns all baseline engine content and
the baseline engine will migrate onto the SDK. A built-in analytic ephemeris in three
tiers (`standard` default) ships in v1 as its own phase. The SDK owns the
entire astronomy layer above raw positions; provider overrides
`prefer-native` by default. Calendars at least at the baseline engine's level in v1.
Teistro Intl as the single localisation standard (base locale `en-Latn`,
JSON, `i18n/<locale>/<namespace>.json`), offered to consumers too.
Fumadocs. British spelling. Precision-first `f64`. Two-person team,
public repository. Teimeris updated as needed. Names as recommended.
Binding order Node native, wasm, Dart/Flutter, Python, Rust, Java. DCO and
Conventional Commits. Eclipses and the full star catalogue in v1.x.
Everything we author is Rust, tooling included (`cargo xtask`). The
quality bar (tests, benchmarks, memory and leak checks, size and coverage
gates) is part of "done" for every module (ADR-0015). Second session:
`f64` astronomy with canonical nanoarcsecond angles, exact integer
classification and exact rational dasha spans (ADR-0016); one kernel per
family with systems as cited rows, falsified before code, and a lazy dasha
cursor (ADR-0017); evidence ranks with V/T/S marks and refusal of
unsourced variants (ADR-0018); a clean-room policy, a licence allow list
and adapter containment (ADR-0019); a calculation version and the
extended envelope (ADR-0020); the IAU routines as an ERFA port, a DE file
reader and a DE-refit `reference` tier (ADR-0021); byte identity across
architectures by hash and a separate CC0 conformance repository
(ADR-0022); type safety with generated, documented surfaces in every
binding (ADR-0023).

## Now

- Phase 2's close: one request for both bodies of a composite crossing;
  the visibility follow-ups (a photometric criterion, the stars' heliacal
  search, the Moon's first crescent; `astro-planetary-phenomena.md` §10);
  cusp speeds and house positions when the chart layer needs them
  (Phase 4). The completion's
  centre, corrections and equinox steps wait for the built-in ephemeris
  (Phase 3). The memo's R3 stays open (a third source; the committee's
  earlier years are not online).

## Next

1. Phase 2's astronomy as above (it is "Now"); a settable atmosphere
   for the rise and set solver (C34); the classical provider's Lagna and
   planetary hours exposed through the chart layer when it exists.
2. ADR-0007's consequences into `02-architecture/07-binding-architecture.md`
   in Phase 1: the blob encoding as the designed wire format, finaliser-
   backed handles with explicit `dispose`, the `api:` metadata line as
   the description's source, generated decoders.
3. Spike 3's remaining consequences: the kit's corpus checks (positions
   against fixtures per tier) and the `sdk-only` cross-provider
   byte-identity check; the Teimeris adapter as the Teimeris package's
   own crate.
4. Spike 4's consequences in Phase 1: the `intl` crate and the
   `teistro-intl` CLI from the spike's crate, the locale bundle
   container, the calendar-aware `:date` family, rich renderers per
   binding, and `migrate baseline` into the entity namespace.
5. Q24: conduct and security mailboxes on the Teispace domain.
6. Before Phase 1 exits: create `teispace/teistro-conformance` (CC0-1.0)
   and move `fixtures/` into it as a submodule (ADR-0022); the
   maintainer creates the repository.
7. Close the cruxes that block Phase 5 (C6 year length per system, C1,
   C2, C3, C8) by reading the texts; tradition reviewers as they appear.
8. A second baseline export (the same script, more sections) for the
   seventeen other dasha systems, aspects, yogas and doshas, strengths,
   Ashtakavarga, the Jaimini slice, KP and milan, once the design pages
   say what each fixture must carry; and the harness itself in Phase 1
   (`05-testing/01-golden-vectors.md`).

## Session log

| date | what happened |
|---|---|
| 2026-09-04 | The baseline engine analysis, Teimeris survey, competitive and platform research, docs written. Twenty-three questions compiled and decided by the maintainer the same day. Architecture revised for the astronomy layer, the built-in ephemeris and Teistro Intl; roadmap restructured into ten phases; governance and scaffolding written; the tooling made Rust-only (`xtask`) before the founding commit was finalised; repository `teispace/teistro-sdk` created public with the docs as the first commit and `main` protected. Next: spike 1, the golden-vector export from the baseline engine. |
| 2026-09-04 (second session) | A review of the team's earlier internal planning notes for the same SDK, now retired; everything worth keeping was absorbed into this repository in its own words and the notes are not referenced. Eight decisions (Q26 to Q33, ADR-0016 to ADR-0023), five falsified kernel and arithmetic designs, the cruxes register, the clean-room policy and dependency allow list, library lints, and the corresponding revisions across the architecture, quality bar, roadmap and guidelines. The maintainer added the type-safety mandate (Q33). Next: spike 1 unchanged. |
| 2026-09-04 (third session) | Spike 1 done: the export script written beside the baseline engine and run; 55 charts (48 chosen adversarially, 7 placed by search at classification boundaries in the topocentric frame), 115 fixtures under 13 settings profiles in `fixtures/baseline/`; the fixtures README with the schema and ten baseline conventions for the deliberate-difference registry; the provisional central tolerance file; `cargo xtask check-fixtures` in the fast check; `05-testing/01-golden-vectors.md` as the result page. Findings: the natal panchanga is topocentric while the daily one is geocentric; local mean time is rounded to the minute; Placidus above the polar circle is not flagged degenerate. Next: spike 2, the binding toolchain. |
| 2026-09-05 (fourth session) | Spike 2 done and decided: option A (ADR-0007). The slice, a designed C ABI with a result blob, a Rust extractor and five emitters sharing one rules module, generated and hand-written Node and Dart layers, a Diplomat bridge with its JavaScript (wasm) and Dart outputs, one benchmark harness per language, and four result files under `spikes/02-binding-toolchain/`. Findings: Diplomat 0.16 refuses host callbacks in JavaScript and Dart; a C-ABI callback costs 0.5 µs into JavaScript and 0.1 µs into Dart; a depth-3 tree marshals in 6 µs as a blob against 179 µs as accessors; Diplomat's Dart output emitted the keyword `true` as an enum member. The maintainer's mandate (type safe, DRY, clean, no repetition) recorded. Next: spike 3, the ephemeris port. |
| 2026-09-05 (fifth session) | Spike 3 done: the ephemeris port under `spikes/03-ephemeris-port/`, a port crate in the workspace (model, trait, C vtable, ERFA-ported obliquity and nutation, frame completion by policy, a thirteen-check kit, runner, timing helper, `.se1` scanner, the spike-2 test provider behind the port) and two standalone adapters outside it (Teimeris; the Swiss Ephemeris compiled from `SWEPH_SRC_DIR` under the containment rules, with the cross-thread stress test). Three providers pass the same kit; the engines agree to every printed digit; the port costs 0.2 % over Teimeris's own call and 1.8 % through the vtable. Findings: the SDK's Delta T fit is 5 s stale by 2025 (Phase 1 uses a table plus a model); Teimeris's grid is body-major and its ayanamsha call takes only the no-nutation switch; the mean ayanamsha is the override to expose. The design page written; `NOTICE` gains ERFA. Next: spike 4, Teistro Intl. |
| 2026-09-05 (sixth session) | Spike 4 done: Teistro Intl under `spikes/04-teistro-intl/`, the stable `MessageFormat 2` grammar with the SDK's functions and ICU4X plurals, the `i18n/` conventions on 49 entities and 13 messages in English and Nepali, a validator with twelve proven gates, the `.tpack` container, typed accessors for TypeScript and Dart with harnesses that reject wrong usages, and the CLI. Measured: renders 0.5 to 2.7 µs, a 6 KB pack verified in 1.4 µs, a lookup in 0.46 µs. Findings: stable syntax over the draft, no parameter sidecar, entities select on their own gender, exact ordinal keys for Nepali, source text in packs with a locale bundle to come. The design page written; Phase 0 exited. Next: the Phase 1 design pages (core types and catalogue first). |
| 2026-09-05 (seventh session) | The five Phase 1 foundation design pages written in `03-design/` (core types and the catalogue, settings and profiles, time and time zones, the arithmetic calendars, Bikram Sambat), each following the ten-section template with a data model, algorithms, an API, errors, a budget, tests, localisation and open questions; the architecture pages they settle and the design index updated. Decisions recorded: the lagna is a point, not a graha; school-dependent values are kernel rows, never catalogue attributes; only the resolved settings are hashed; Delta T is a table then a model; `Resolution` gains `Defined`; Bikram Sambat's month-start rule is chosen by measurement against the official table. Next: `crates/core`. |
| 2026-09-05 (eighth session) | `crates/core` built from its design page: the catalogue as YAML sources (53 kinds, 629 members, cited and marked) with a generator and a CI gate; keys, ids and suggestions; validated quantities with compile-fail proofs; the exact angle with a property-tested partition of the circle; bounded rationals; status codes and a small error; the provenance envelope; registries and limits; settings with thirteen knob groups, patches, five shipped profiles over a cited root, coherence rules and a canonical hash. Benchmarked: key resolution 40 ns, classification 1.7 ns, profile resolution 1.9 µs, settings hash 15 µs. Next: `crates/time` and `crates/calendar`. |
| 2026-09-05 (eleventh session) | `crates/time` and `crates/port-timezone` built: scales and their stamps, Delta T as the IERS series (1956 to August 2026, fetched with provenance) then Espenak and Meeus with Morrison and Stephenson's uncertainties, the IANA leap seconds with folding and expiry, civil time, zone resolution over the embedded tzdb (2026c) with replay-safe metadata under the daylight-saving policies and an era decided without a clock, local mean time, the local day with the polar policies, ghati-pala exact on a tenth-of-a-millisecond grid; `gen time` and `check-time`. The 55 fixture charts reproduce the baseline's instant and metadata; five era labels differ by design (C33). Findings: the IERS carries UT1 only from 1956; Stephenson, Morrison and Hohenkerk's coefficients are not in hand (C32); an `f64` Julian day resolves fifty microseconds, so ghati-pala snaps to a hundred. Next: the port promotion with the drik solar model and the rise and set solver. |
| 2026-09-05 (twelfth session) | Spike 3's port promoted into `crates/port-ephemeris` (with the rise and set override), `crates/astro` (Delta T moved in from `time`; the ERFA ports with a provenance table; sidereal time and the obliquity; frame completion; the rise and set solver) and `crates/ephemeris-kit` (fifteen checks); `DrikSun` for the calendars; the local day's convention and the `SUNRISE` fallback; the adapters under `adapters/` with a Teimeris rise and set override and a `bs-fit` binary. Measured: 0.13 s against Teimeris's geometric sunrise, 7.3 s refracted at 64°N (C34), 2.5 s against the fixtures below 60° (C35); modern positions 65 % of the official Bikram Sambat months against the text's 98.5 %; the committee names the Surya Siddhanta as its method. Next: the `siddhanta` verses and provider adapter, planetary hours, DUT1. |
| 2026-09-05 (thirteenth session) | `crates/siddhanta` completed to the text (the sighra daily motion, the latitudes, the Lagna) against Burgess's 1860 worked computation, and presented behind the ephemeris port as a classical astronomy (`SiddhantaProvider`) that passes the kit; the port's `Astronomy`, `SpeedModel`, `DistanceUnit` and `DUT1` override; the kit's sixteenth check, its second-difference continuity and its informational rows for a classical provider (C36, C37); the completion's zodiac-then-rotation order; the planetary hours in `time` under `hora_reckoning` (proportional, as 52 of 55 fixtures decide and the other three cannot) and UT1 from a provider's DUT1. Next: Phase 2's astronomy pages, the ayanamsha catalogue, houses, crossings and stations. |
| 2026-09-05 (fourteenth session) | The national panchanga committee's 2082 and 2083 panchangas fetched from `npns.gov.np` through the browser and read into `fixtures/official/npns-2082-2083.json` (24 sankranti instants, printed places, sunrise and sunset, tithi ends). The SDK's engine reproduces every instant within 1.6 minutes and every month start; the committee's Sun is the text's within 3″, its Moon the text's with four revolutions fewer on the apsis, its star planets modern positions in the Lahiri frame, its sunrise modern under its own convention. R2 closed for the method; C30 explained as the earlier makers' decisions; C38 and C39 opened. Next: Phase 2's astronomy pages. |
| 2026-09-05 (fifteenth session) | Phase 2's astronomy begun: new ERFA ports (IAU 2006 precession, Vondrák 2011 long-term poles and matrices, the vector primitives) with their reference values; `astro::precession` as four models with consistent obliquities; `astro::ayanamsha` computing every epoch-defined member from its published definition with the fitted-model correction, mean or nutated, custom definitions, the anchored members refused by name; the completion completing the sidereal zodiac from the SDK's catalogue. Measured: TT-epoch definitions within 1e-7″ of Teimeris and UT-epoch ones within 2.1e-4″ over 1044 recorded rows; 142 ns a precession matrix, 0.58 µs an ayanamsha. Design pages `astro-timescales-and-frames.md` and `astro-ayanamsha-catalogue.md`. Next: houses, crossings and stations, the star table. |
| 2026-09-05 (twenty-fourth session) | Visibility and the heliacal phenomena (`astro::visibility`): three named criteria, the Surya Siddhanta's degrees of time (IX.2 to 11, X.1, read from Burgess's 1860 translation: Jupiter 11, Saturn 15, Mars 17, Venus 10/8, Mercury 14/12, the Moon 12; the star classes of IX.12 to 15 and the six stars of IX.18 as data), the tradition's combustion orb over the same numbers in longitude, and Ptolemy's arcus visionis (Almagest XIII.7 to 9 as Burgess quotes them) read at the deepest twilight the body is up in; the state of a local mean day and the day-by-day scan for the four heliacal events over the rise and set solver and the completion, so any provider answers. `Solver::altitude_deg`, `sky::local_mean_midnight` (moved from the classical crate). By hand against Teimeris's photometric model at Kathmandu: Venus's rising of June 2020 and setting of May 2020 within two days under every criterion, Jupiter's rising of February 2021 within a week (bound ten days). Research: the tradition's combustion orbs verified as the text's own numbers (C17 closed at rank 1; C44 opened on the unit). Next: one request for both bodies of a composite crossing; the visibility follow-ups. |
| 2026-09-05 (twenty-third session) | The sidereal time expression question closed by measurement: the engine's default sidereal time strictly inside its 1850 to 2050 window is the IERS 2010 expression and agrees with the SDK's IAU 2006 form to 0.0012″ (1850) and 0.0004″ (from 1875); the +0.088″ once read "inside the window" was the boundary instant, which the engine gives to its long-term branch. The SDK's GAST moved from `gst00b` to `gst06b` (IAU 2006 mean sidereal time, IAU 2006 mean obliquity, IAU 2000B nutation): cusps move under 0.002″ between 1950 and 2050. New: `iau::ee06b`, `iau::gst06b` (against ERFA's `ee06a`/`gst06a` within the 2000B truncation), the adapter's `sidereal-table` binary, `fixtures/teimeris/sidereal.json` (49 instants), `tests/teimeris_sidereal.rs` (three accuracy rows in CI), crux C43 (the 2000B nutation read two ways). Engine findings: F1 measured beyond the window (−0.50″ at 1700 to +2.46″ at 2300, commented on teimeris#1); F6 filed (teimeris#6: the 2000B fixed offsets −0.135/+0.388 mas omitted, inherited from upstream). Next: the heliacal phenomena. |
| 2026-09-05 (twenty-second session) | The `CROSSINGS` override: the crossing vocabulary moved into the port (`crossing.rs`: `Quantity`, `Lattice`, `Direction`, `Event`, `CrossingRequest`; the events module re-exports it), `EphemerisProvider::crossings`, a vtable slot with a caller-owned buffer that grows to the count reported (ABI version 2), `Completion::crossings` choosing by the override policy and falling to the kernel for a request the provider refuses, the Teimeris adapter's implementation over its crossing search (the direction from the quantity's rate), and the kit's two crossings checks. Against Teimeris: Mercury's sign crossings within 0.0034 s and the tithis within 0.0039 s of the kernel, 18 checks all passed. A latent kit defect fixed on the way: the Surya Siddhanta ayanamsha expectation was the text's own value for every provider; for a modern engine it is now the catalogued epoch definition. Next: the heliacal phenomena, the sidereal-time expression question. |
| 2026-09-05 (twenty-first session) | The accuracy document, the Phase 2 exit artefact: `cargo xtask accuracy` runs the astronomy layer's measurement tests with `TEISTRO_ACCURACY_DIR` set, each recording its worst difference against its recorded engine or baseline table (`crates/astro/tests/common/mod.rs`), and renders `05-testing/ACCURACY.md` from those measurements and `accuracy-rows.yaml` (the seventeen areas of the astronomy layer with their targets, evidence and by-hand measurements); `check-accuracy` regenerates and compares in CI. Next: the `CROSSINGS` override, the heliacal phenomena. |
| 2026-09-05 (twentieth session) | The engine findings register (`05-testing/02-engine-findings.md`) and the rule behind it: five discrepancies the measurements traced to Teimeris measured, filed as `teispace/teimeris` issues #1 to #5 with reproductions and suggested fixes, assigned to its maintainer, and entered with the SDK's handling: the sidereal-time steps of 1.9″ at 2050 and 0.1″ at 1850 (which explain the equation-of-time gap; the earlier Delta T reading was wrong and is corrected), the Moon's disc and parallax from distances 40 km apart, a point's magnitude as 0.0, the Horizon system's Munkasey co-ascendant at the equator, and five star-catalogue rows. Next: the accuracy document, the `CROSSINGS` override. |
| 2026-09-05 (nineteenth session) | `astro::phenomena` and the equation of time: elongation, phase angle and illuminated fraction, the apparent disc and horizontal parallax (the rise and set solver's `Disc`), and the visual magnitude under the Almanac's models (Mallama and Hilton 2018 for Mercury to Uranus, Neptune's calendar step, Pluto's IAU 1986 polynomial, Allen with Samaha's crescent for the Moon, the inverse-square Sun), over the completion (the provider's heliocentric position at the retarded instant when it answers one, the geocentric difference otherwise) or a supplied geometry; `sky::equation_of_time_seconds` from the SDK's sidereal time and the Sun's apparent right ascension. Measured against Teimeris over its own geometry: angles within 1e-9°, magnitudes within 0.001 (the Sun's disc radius), the equation of time within a millisecond. Design page `astro-planetary-phenomena.md`; C19 updated. Next: the accuracy document, the `CROSSINGS` override. |
| 2026-09-05 (eighteenth session) | The star table: `catalogue/star.yaml` (kind 56, 128 members: the 27 yogataras with Vega, the ayanamsha anchors, the bright fixed stars, Sagittarius A* and the two galactic poles, each with SIMBAD's ICRS astrometry and the bibcode of every value) and `star_class`; new ERFA ports (`epv00` with its 1951 rows, `pmpx`, `ld`, `ldsun`, `ab`, `numat`) against the reference values; `astro::stars` placing a direction on the equator and ecliptic of date (proper motion, parallax, deflection, aberration over the SDK's own Earth ephemeris, frame bias, precession, nutation); the twelve anchored ayanamshas computing through it. Measured against Teimeris: the mean places over the engine's own astrometry bit-identical, the apparent within 0.0005″ (the nutation models), the parallax kept under its true-position flag; the SDK's astrometry against the engine's within 0.71″ (Gaia DR3 against Hipparcos); the anchored ayanamshas within 0.003″ where the rows are the same and by the data where they are not (C40 to C42). 12.6 µs the Earth's state, 13.9 µs a star's place. Design page `astro-star-table.md`. Next: a `CROSSINGS` override, cusp speeds, the equation of time. |
| 2026-09-05 (seventeenth session) | `astro::events`: the crossings and stations kernel over the boundary solver: a body's longitude, a composite angle of two bodies or a speed over a lattice of boundaries (the signs, the nakshatras, the tithis, the karanas, the yogas) or a single target, sampled at half the spacing over the greatest rate and never more than a day, unwrapped, each line narrowed by the shared solver; stations as the speed's sign changes; a synthetic looping planet as the retrograde test. The solver's narrowing moved from bisection to the ITP method with a floating-point guard: at most nine evaluations an event where bisection took twenty-seven, never more than a bisection and one. Measured against Teimeris's own searches: ingresses and tithi boundaries within 0.004 s, stations within 0.3 s; against the baseline's 280 geocentric panchanga transitions within 7.8 s (median 3.3 s, the baseline's own search). Design page revised. Next: the star table. |
| 2026-09-05 (sixteenth session) | `astro::houses`: the twenty-two catalogued house systems as one construction with the circles each picks, the auxiliary points, the sign-based systems in the zodiac in use, the four polar policies with the outcome reported. Within 4.8e-6° of Teimeris over 25 194 cusps and angles at ten latitudes (the adapter's `houses-table` binary), within 0.00021° of the baseline's 55 charts between 1800 and 2200. Design page `astro-house-systems.md`. Next: crossings and stations, the star table. |
| 2026-09-05 (tenth session) | The Bikram Sambat computation engine: `crates/siddhanta` (the text by verse, exact mean places, the sine table, both equations, the four steps, motion, precession, declination, the day's arc; 54 ns for the Sun, bit-identical), the `astro` seed (the boundary solver), the `time` seed (offset histories, Nepal's rows) with `core::time`, and in `crates/calendar` the `SolarModel`, the sankranti finder, the month-start rules as cited rows, the engine, the fit report and the table regenerated for 1700 to 2500 BS with a CI gate. Measured: the text's Sun at Kathmandu under Nepal's clock with the Dharmasindhu's punya-kala rule reproduces 1490 of 1512 official month lengths (98.5 %), 116 of 126 years exactly, every year total and every New Year, no drift; the eleven residual boundaries lie within 25 minutes of the rule's boundary. Findings: the baseline's seven-hour epoch shift and 0.705 cutoff nearly cancel to the civil day; the two ayana sankrantis are the whole difference; exact trigonometry changes one boundary; the tradition's day count changes none. Next: `crates/time` proper, then the port promotion with the drik model. |
| 2026-09-05 (ninth session) | `crates/calendar` built: the fixed day, Gregorian, Julian, mixed (1582, 1752, 1918) and ISO week with every day of −9999 to 9999 round-tripped and agreed with the `calendrical_calculations` oracle; Bikram Sambat over the baseline's table (1856 to 2457, official span stamped `Tabular`, the rest `Computed`) anchored on 13 April 1913; the source memo opened with the generator's findings (Surya Siddhanta at Kathmandu, Nepal's offset history, a fitted 0.705 cutoff, 87 % of month splits, drift within a day). Maintainer's mandate: compute Bikram Sambat from first principles for any year so Nepal's panchanga can use the SDK. Next: the Bikram Sambat engine (siddhanta Sun, drik through the port, rule rows, fit harness), then `crates/time`. |
