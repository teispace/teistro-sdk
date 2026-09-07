# Design

Status: `draft`, revised 2026-09-05. Per-module detailed designs are
written one page per module before the module's code. Five pages were
written in Phase 0 because their content is retrofit-hostile (ADR-0016,
ADR-0017): the kernels are falsified against the catalogue before any
code exists; the ephemeris port and Teistro Intl pages came from spikes 3
and 4; the five Phase 1 foundation pages (core types and the catalogue,
settings and profiles, time and time zones, the arithmetic calendars,
Bikram Sambat) were written at Phase 1's start, the Surya Siddhanta page
with its crate for the Bikram Sambat engine, and the events page with
the rise and set solver when the port was promoted.

## What a design page contains

1. Purpose and scope, with the research page it derives from.
2. Inputs, settings knobs it reads, ports and capabilities it needs.
3. The data model: types, keys emitted, result shape, provenance.
4. Algorithms with citations, named variants, and the default per profile.
5. The API: C ABI entry points, ergonomic shape, batch forms.
6. Errors and degenerate states.
7. Performance budget and the benchmark that measures it.
8. Tests: unit examples from texts, golden-vector sets, property tests,
   parity coverage.
9. Localisation: namespaces and keys it introduces.
10. Open questions, linked to `QUESTIONS.md`.

## Drafted

| page | what it settles |
|---|---|
| [`exact-arithmetic.md`](exact-arithmetic.md) | nanoarcsecond angles, integer classification, rational periods (ADR-0016) |
| [`dasha-kernels.md`](dasha-kernels.md) | the udu and rashi kernels, 56 systems as rows with confidence marks, the lazy cursor |
| [`varga-kernel.md`](varga-kernel.md) | one table-driven evaluator for every divisional chart and variant |
| [`strength-schemes.md`](strength-schemes.md) | bala schemes with group membership as data, 18 components |
| [`rules-engine.md`](rules-engine.md) | the predicate algebra v2: references, table lookups, classifying outcomes, cancellation |
| [`ephemeris-port-and-adapters.md`](ephemeris-port-and-adapters.md) | positions required and overrides declared (the obliquity, Delta T, the ayanamsha, rise and set), the frame and its bits, columns instants outermost, the C vtable, frame completion by policy, the adapter rules, the kit's fifteen checks and bounds with the measured values of both engines; built |
| [`astro-events-and-crossings.md`](astro-events-and-crossings.md) | the boundary solver and the rise and set solver over ERFA-ported sidereal time and obliquity, the horizon conventions and the event altitude, polar days as reported absences, the measurements against Teimeris and the baseline's fixtures; crossings and stations planned |
| [`intl-engine-and-packs.md`](intl-engine-and-packs.md) | the source conventions, the stable `MessageFormat 2` grammar and the SDK's functions with the types they imply, selection on entities and contexts, the validation gates with the catalogue as authority, the `.tpack` container and the locale bundle, the typed accessors for TypeScript, Dart and Rust, the `teistro-intl` command line; built (`crates/intl`) |
| [`core-types-and-catalogue.md`](core-types-and-catalogue.md) | forty kinds with their keys and ids, the catalogue sources and generator, the three rules for what an attribute is, the quantity newtypes, closed unions per binding, the envelope and the status codes, registries and limits |
| [`settings-and-profiles.md`](settings-and-profiles.md) | the knob inventory as typed groups, profiles as patches over a cited root, resolution and coherence rules, the canonical form and hash, the five shipped profiles, per-request patches |
| [`time-and-timezone.md`](time-and-timezone.md) | scales and instants, Delta T as the IERS table then a model with an uncertainty on every value, the leap-second table, zone resolution with replay-safe metadata and DST policies over the embedded tzdb, local mean time, the sunrise-anchored day with the polar policies, ghati-pala as exact integer arithmetic; built |
| [`calendar-gregorian-julian.md`](calendar-gregorian-julian.md) | the fixed day and the Julian day, the four arithmetic calendars over Reingold and Dershowitz, the mixed transition, ISO weeks, exhaustive and differential tests |
| [`calendar-bikram-sambat.md`](calendar-bikram-sambat.md) | the official table and the computed extension from the SDK's own engine, the month-start rules as rows with the punya-kala rule chosen by measurement (98.5 % of the official month lengths, every New Year), tabular, computed and divergent resolution, the generated table and its gate, era numbers by new-year rules, the source memo |
| [`siddhanta.md`](siddhanta.md) | the Surya Siddhanta as a computation: the text's numbers by verse, exact integer mean places, the sine table, the manda and sighra equations and the four steps, daily motion, precession, declination and the day's arc; the classical path bit-identical everywhere |
| [`astro-timescales-and-frames.md`](astro-timescales-and-frames.md) | the branded time scales, precession as a catalogue of models (Vondrák 2011 the default, IAU 2006, IAU 1976, Newcomb) over the ERFA ports with the obliquity each is consistent with, nutation, the frame bias, the completion steps built and the ones designed (centre, corrections, equinox, topocentric) |
| [`astro-house-systems.md`](astro-house-systems.md) | the twenty-two house systems as one construction with twenty-two choices of circles, the auxiliary points, the sign-based systems in a sidereal zodiac, the polar behaviour of each system and the four policies; within 5e-6° of Teimeris at ten latitudes and 0.0002° of the baseline's 55 charts; built |
| [`chart-bhava-chalit.md`](chart-bhava-chalit.md) | the falsification pass over the four bhava chalit methods, generated by `cargo xtask chalit` and held by `check-chalit`: which one the recording engine actually computes (Vehlow, on all 55 charts, whatever its label says), and how often each pair puts a graha in a different house (21.8% for the two a Jyotisha application chooses between; 50.5% for the two that share their cusps exactly); measured |
| [`astro-ayanamsha-catalogue.md`](astro-ayanamsha-catalogue.md) | the forty-seven ayanamshas as definitions (epoch and value, frame, or anchor) with their sources, the construction that carries a value to any date and the fitted-model correction, mean against nutated, custom definitions, the twelve anchored members refused until the star table; every epoch-defined member within 1e-7″ of Teimeris over 1044 rows; built |
| [`astro-star-table.md`](astro-star-table.md) | the star table: 128 catalogue members with SIMBAD astrometry and its bibcodes, the apparent, mean and geometric places over the SDK's own Earth ephemeris and frame bias, the twelve star-anchored ayanamshas; bit for bit with Teimeris on the same astrometry; built |
| [`astro-planetary-phenomena.md`](astro-planetary-phenomena.md) | elongation, phase, the apparent disc and parallax, the visual magnitude under the Astronomical Almanac's models, the equation of time; visibility and the heliacal phenomena under three named criteria (the Surya Siddhanta's degrees of time, the combustion orb, Ptolemy's arcus visionis); built |
| [`ffi-abi-and-api-description.md`](ffi-abi-and-api-description.md) | the C ABI as built (the size handshake, the last error, owned and lent memory, the panic guard, the JSON settings patch), the `TSRB` result blob and its schemas, the API description with roles inferred from types and names and the `api:` metadata line, the layout rules and the header's assertions, the thirty-three entry points, the gates; built (`crates/ffi`, `crates/idl`) |

## Planned pages (in roadmap order)

| page | phase |
|---|---|
| `calendar-indian-lunisolar.md` | 2 |
| `astro-star-table.md` (crossings and stations: the next revision of `astro-events-and-crossings.md`) | 2 |
| `ephemeris-builtin.md` (theories, tiers, ingestion tool) | 3 |
| `chart-foundation.md`, `houses.md`, `state.md`, `aspect.md`, `points.md` (vargas: `varga-kernel.md`) | 4 |
| `panchanga-day.md` | 4 |
| `strength-ashtakavarga.md`, `strength-bhava-vimshopaka.md` (Shadbala: see `strength-schemes.md`) | 5 |
| `dasha-registry.md` (the registry and plug-in interface over `dasha-kernels.md`) | 5 |
| `rules-yoga-dosha-packs.md` (the engine: `rules-engine.md`) | 6 |
| `interpret-composers.md` | 4 |
| `jaimini.md`, `kp.md`, `tajika.md`, `gochar.md`, `muhurta.md` | 5 |
| `matching.md`, `prashna.md`, `rectification.md`, `longevity.md`, `remedies.md`, `numerology.md`, `lalkitab.md`, `pakshi.md`, `namakarana.md`, `rashifal.md`, `research.md` | 6 |
| `serial-json-dossier-blob.md`, `chart-geometry.md` | 7 |
| `western-*.md`, `hellenistic-*.md` | v1.x |
