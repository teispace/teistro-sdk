# Module catalogue

Status: `draft`, revised 2026-09-06 (`ffi` and `idl` built, rows
added); revised 2026-09-05 (what is built so far noted per
row; `time` consumes `calendar`, `calendar` consumes `siddhanta`);
revised 2026-09-04 after Q7, Q8 and Q10 (the `astro`
layer, the built-in ephemeris and Teistro Intl added) and after
ADR-0016, ADR-0017 and ADR-0021 (exact arithmetic in `core`, kernels and
tables for dasha, vargas, strength and rules, the ERFA-derived IAU
routines in `astro`, `ephemeris-de` and the `reference` tier).
Derived from the feature pages in `01-research/feature-universe/`. Each
module is a crate in the Rust workspace and a package (or a feature of a
family package) in every binding.

## L0 and L1

| module | contents | depends on | baseline source |
|---|---|---|---|
| `core` | keys and entity catalogue (grahas, rashis, nakshatras with padas, tithis, karanas, yogas, varas, vargas, dignities, states, ayanamshas, house systems, samvatsaras), angles with the canonical nanoarcsecond type and exact classification (`core::angle`, ADR-0016), exact rationals, Julian day and timescales, settings and profiles, result envelope with the calculation version (ADR-0020), errors, capabilities, registries, limits | none | `core` data, enums, types |
| `port-ephemeris` | the ephemeris port trait (positions required; the obliquity, Delta T, ayanamsha and rise-and-set overrides declared), the request over a grid, the columnar response with a status and a source per cell, frames packed to 32 bits, capabilities with content hashes, the horizon convention, the C vtable both ways, the `.se1` file scanner, the analytic test provider. Built 2026-09-05 (`03-design/ephemeris-port-and-adapters.md`) | `core` | `AstronomicalBackend` |
| `ephemeris-kit` | the provider conformance kit: eighteen checks under one published set of bounds, the report, the timing rows and the runner the kit binaries share; run against the test provider in CI and against the adapters by hand. Built 2026-09-05 | `port-ephemeris`, `astro` | |
| `port-timezone` | the zone database contract: version, zones, the offset at an instant with the abbreviation and the before-rules flag, the candidates of a civil time (one, a gap, an overlap), the offsets a zone applies today. Built 2026-09-05 | `core` | P0 | |
| `port-calendar`, `port-geo`, `port-intl-data`, `port-log` | as before | `core` | |
| `idl` | the API description (`idl/api.json`, `teistro-api/1`): the model, the naming rules, the C layout rules, the `TSRB` result blob encoder and decoder, the extractor over the boundary crates' Rust source with roles inferred from types and names, the SDK's own sources and catalogue kinds put through it, and the emitters (the C header today; Node, TypeScript, Dart next). Built 2026-09-06 (`03-design/ffi-abi-and-api-description.md`) | none (`syn` behind the `extract` feature) | the spike-2 extractor and emitters |
| `ffi` | the C ABI, the workspace's only `unsafe`: contexts from a profile, a JSON patch, a locale and the port's vtable; the last error with its message, field and hint; keys and ids; dates in every shipped calendar; civil times to instants with the zone metadata and the scale conversions; the locale engine over the embedded bundles; positions through the port completed into the requested frame, as a result blob with the steps and the provenance. Thirty-six entry points; `cargo xtask gen ffi`, `check-ffi`, and `check-c` for the binding's own C test. Built 2026-09-06 | `core`, `port-ephemeris`, `astro`, `calendar`, `time`, `intl`, `idl` | Teimeris's C ABI conventions |

## L1.5 astronomy

| module | contents | depends on | tier | notes |
|---|---|---|---|---|
| `astro` | timescales and Delta T models, sidereal time, precession (11 models), nutation (5), obliquity, frame bias, frame completion (light-time, aberration, deflection, nutation), topocentric parallax, coordinate transforms and refraction, the ayanamsha catalogue (47 plus custom), house systems (all) with the polar policy, the shared boundary solver (one root finder with a per-use tolerance), the event-scan kernel (bracket then refine; step from the quantity's rate; scored searches excluded), rise/set/transit, crossings and stations, visibility and heliacal phenomena (degrees of time, combustion orb, arcus visionis), equation of time, star table (anchors and yogataras), eclipses (v1.x). The IAU routines are a faithful port of ERFA (BSD-3) with a provenance table (ADR-0021). Built so far: Delta T and the scales, the ERFA ports (sidereal time, obliquity, nutation 2000B, IAU 2006 and Vondrák 2011 precession, the frame bias, the vector primitives), the obliquity record, frame completion (coordinates, zodiac), the boundary solver, the rise and set solver, precession as a catalogue of four models, the ayanamsha catalogue, the twenty-two house systems with the polar policy, the crossings and stations kernel, the star table with the Earth ephemeris and the apparent-place corrections it stands on, the twelve star-anchored ayanamshas over it, the planetary phenomena (elongation, phase, disc, parallax, magnitude) and the equation of time (`03-design/astro-timescales-and-frames.md`, `astro-ayanamsha-catalogue.md`, `astro-house-systems.md`, `astro-events-and-crossings.md`, `astro-star-table.md`, `astro-planetary-phenomena.md`) | `core`, `port-ephemeris` | P0 | see `01-research/platform/13-astronomy-layer.md`; conformance against Teimeris and the C ERFA library |
| `ephemeris-builtin` | analytic ephemeris implementing the port: VSOP87 planets, ELP/MPP02 Moon, fitted Pluto, nodes and apogees, analytic speeds; tiers `compact`, `standard`, `full`, and `reference` (a Chebyshev refit from DE440 over 1600 to 2400, per-body blobs, ADR-0021); generated tables with citations | `core`, `port-ephemeris` | P0 (own phase); `reference` Phase 3 or v1.x | see `14-builtin-ephemeris.md`; removable by tree-shaking |
| `ephemeris-de` (v1.x) | a provider reading JPL DE files directly: DAF/SPK types 2 and 3, Clenshaw evaluation, centre chaining from segment metadata, record access suited to HTTP range requests; 0.001 arcsecond against Horizons | `core`, `port-ephemeris` | P1 | ADR-0021; fuzzed as a parser of untrusted files |
| `siddhanta` | the Surya Siddhanta as a computation: mean places in exact integer arithmetic, the sine table, the manda and sighra equations and the four steps, the daily motions by the text's rules, latitudes, precession, declination and the day's arc, the Lagna from the oblique ascensions, with a bija overlay; `SiddhantaProvider` presents it behind the ephemeris port as a classical astronomy (`03-design/siddhanta.md`) | `core`, `astro`, `port-ephemeris` | P0 | the text (Burgess, 1860, with his worked example of 1860 as test vectors); the baseline engine for structure only |

Built so far (2026-09-05): in `astro`, Delta T and the UT1 to TT
conversions (moved here from `time`), the ERFA-ported IAU routines
(`iau`: the Earth rotation angle, mean and apparent sidereal time, the
1980 and 2006 obliquities, the 2000B nutation, the fundamental
arguments, the equation of the equinoxes, the refraction constants), the
obliquity record and the ecliptic and equatorial rotations (`sky`), frame
completion over the port (`completion`), the shared boundary solver
(`solve::next_crossing` and `first_zero`) and the rise and set solver
(`rise_set`); `siddhanta` as the model (its provider adapter is still to
come). The precession and nutation model families, the ayanamsha
catalogue, house systems, crossings and stations are Phase 2.

## L2 domain modules

| module | contents | depends on | tier | baseline source |
|---|---|---|---|---|
| `calendar` | Gregorian, Julian, mixed, ISO week, Bikram Sambat (the official table plus the SDK's computed extension, with the solar-calendar engine: a `SolarModel` as the Surya Siddhanta or the drik Sun through the port, the sankranti finder, the month-start rules as rows, the measurement), Indian lunisolar (Amanta, Purnimanta, adhika, kshaya, solar months), eras and samvatsara; later Nepal Sambat, Saka, regional solar, Hijri, Hebrew, Persian, Chinese | `port-calendar`, `astro`, `port-ephemeris`, `siddhanta` | P0 | `BsCalendarService`, `CalendarService` |
| `time` | civil to instant resolution with replay-safe metadata under the DST policies and the `SUNRISE` fallback for a birth without a time, local mean time, the UTC conversions over `astro`'s Delta T, the leap-second table, the sunrise-anchored day from a solar model under its sunrise convention with the polar policies, ghati-pala, the planetary hours under both reckonings, DUT1 from a provider that declares it. Built 2026-09-05 (`03-design/time-and-timezone.md`); the embedded tzdb is `jiff`'s bundled data, never the host's | `port-timezone`, `calendar`, `astro` | P0 | timezone resolver, ghati-pala, birth timing |
| `chart` | birth data, foundation (positions in both frames with speeds, declinations, RA; cusps; frames), chart kinds | `astro`, `time` | P0 | `ChartFoundation` |
| `houses` | placement policy, Bhava-Chalit variants (Sripati, Vehlow, Porphyry, KP), placements by span, whole sign, equal, degeneracy states | `chart`, `astro` | P0 | `HouseService` |
| `vargas` | one table-driven evaluator (`03-design/varga-kernel.md`): standard and extended vargas and every named variant as rows, arbitrary D-N under a recorded convention, the mixed-chart axis, vargottama, varga change search | `chart`, `astro` (crossings) | P0 | `VargaService` |
| `state` | dignities, relationships, combustion, retrogression, war, gandanta, avasthas, marana karaka sthana | `chart`, `vargas` | P0 | dignity, avastha services |
| `aspect` | graha drishti, sphuta drishti, rashi drishti, conjunction logic, Tajika aspects, orb-based aspects (shared engine) | `chart` | P0 | aspect services |
| `points` | special lagnas, upagrahas, Bhrigu bindu, yogi points, sphutas, bhagas, 64th navamsa and 22nd drekkana lords, sahamas and lots | `chart`, `time`, `astro` | P0 | upagraha, special lagna services |
| `strength` | bala schemes as tables with group membership as data (`03-design/strength-schemes.md`): Shadbala (18 components, scheme `parashari-baseline` first), Ishta and Kashta, Bhava Bala, Ashtakavarga, Vimshopaka, Vaiseshikamsa, Tajika balas | `chart`, `houses`, `vargas`, `state`, `aspect` | P0 | strength services |
| `dasha` | the udu and rashi kernels, the Kalachakra kernel, the scale decorator and compositions (`03-design/dasha-kernels.md`); the 18 baseline systems as verified rows, the rest as rows with confidence marks; the lazy cursor (`dasha_at`, range iteration, search) and explicit materialisation; balance methods, year-length table, applicability through `rules`; plug-in interface as a row schema, a trait for the exceptions | `chart`, `state` | P0 | dasha engines |
| `rules` | the predicate algebra v2 (`03-design/rules-engine.md`): reference subjects over every derived point, table lookups over cited tables, classifying and grading outcomes, first-class cancellation with a net status, traces; the pack format and loader; yoga and dosha packs; timing relations; batch search | `chart`, `houses`, `vargas`, `state`, `aspect`, `strength`, `dasha`, `points` | P0 | yoga and dosha evaluator |
| `jaimini` | karakas, arudhas, argala, karakamsa, three pairs, Jaimini yogas | `chart`, `state`, `aspect`, `dasha` | P0 | Jaimini slice |
| `kp` | sub-lord tables to five levels, significators, ruling planets, KP horary numbers, KP profile binding | `chart`, `houses` | P0 | KP module |
| `tajika` | annual and monthly charts, Muntha, year lord, Tajika yogas, sahamas, Mudda and Patyayini | `chart`, `aspect`, `strength`, `dasha`, `astro` | P0 | Tajika service |
| `panchanga` | day model, limbs with transitions, derived timings, choghadiya, hora, muhurta yogas, Panchaka and kin, tarabalam, chandrabalam, festival rule-pack hooks | `chart`, `time`, `calendar`, `astro` | P0 | panchanga package |
| `muhurta` | activity rules, blackouts, shuddhi scoring, natal scoring, lagna and karaka evaluation, window assembly, ranked search, instant evaluation, published-days data | `panchanga`, `chart`, `rules` | P0 | muhurta services |
| `gochar` | transit snapshots and overlays, vedha, Sade Sati, transit strength, phala, ingresses, stations, hit lists, transit calendar | `chart`, `strength`, `dasha`, `astro` | P0 | predictive gochar |
| `prashna` | query chart analysis, number-based prashna, void-of-course, scoring schemes, timing, KP horary | `chart`, `kp`, `aspect`, `rules` | P0 | prashna module |
| `matching` | Ashta Koota, Dasha Koota, Mangal matching, marriage doshas, quick match, synastry, composites | `chart`, `state`, `rules` | P0 | milan, research |
| `rectification` | candidate grid, stage interface, Tattwa prior, dasha-boundary stage, refinement, intervals, hold-out; later stages | `chart`, `dasha`, `houses`, `time` | P0 | rectification module |
| `longevity` | Ayurdaya methods, Maraka, windows, Balarishta, disclaimer flag | `chart`, `strength`, `dasha`, `gochar` | P0 | Ayurdaya, Maraka |
| `remedies` | functional classification, gemstone safety, remedy packs, priorities | `chart`, `state`, `dasha`, `rules` | P0 | remedy module |
| `numerology` | Pythagorean, Chaldean, name compatibility | `core`, `intl` (transliteration) | P0 | numerology |
| `lalkitab` | house-number chart, sleeping planets, rinas, Varshkundali, yogas, remedies | `chart` | P0 | lalkitab |
| `pakshi` | Pancha Pakshi cycles | `panchanga` | P0 | pancha-pakshi |
| `namakarana` | akshar from nakshatra pada, name validation, corpus suggestions, numerology link | `chart`, `intl` | P0 | namakarana |
| `rashifal` | period context, overlays, scoring, life areas, lucky elements | `gochar`, `panchanga` | P0 | rashifal-core |
| `research` | batch computation, statistics, rule search over sets | everything above | P0 | research |
| `western` (v1.x) | points catalogue, aspects with orbs, midpoints, harmonics, progressions, directions, returns, synastry, fixed stars, Arabic parts, mapping geometry, declinations | `chart`, `aspect`, `gochar`, `astro` | P1 | none |
| `hellenistic` (v1.x) | sect, dignity tables, lots, profections, zodiacal releasing, firdaria, horary considerations | `chart`, `state`, `dasha` | P1 | none |

## L3 presentation

| module | contents | depends on | tier | baseline source |
|---|---|---|---|---|
| `interpret` | state-key derivation from results, rule interpretation lookup, composers as narrative plans, report section catalogue | all L2 it interprets | P0 | interpret package |
| `intl` | present: Teistro Intl, the stable MF2 grammar in full, the functions bound to SDK types (`:entity` on the catalogue, `:zodiac`, `:dms`, `:list`, `:msg`), numerals and grouping per locale, fallback chains with provenance, validation with the catalogue as authority, `.tpack` packs and `.tbundle` locale bundles, typed accessors for TypeScript, Dart and Rust, the `teistro-intl` CLI as a library; the runtime API (packs loaded after construction, overrides, the report), the calendar-aware `:date` family with `:ghati` and `:duration`, `migrate baseline`; to come: transliteration, XLIFF, twelve-hour clocks | `core` (then `port-intl-data`, `calendar`) | P0 | registry names, locale service, numeral formatter, phonetic matcher |
| `serial` | canonical JSON, dossier text with presets and date-free persistence, gochar sidecar, binary blob, content hashes, chart layout geometry, dasha timeline shapes | all it serialises | P0 | chart serialiser, layout code |

## Profiles (shipping sets)

| profile | modules |
|---|---|
| `calendar` | core, calendar, intl (date namespaces) |
| `panchanga` | core, ports, astro, ephemeris-builtin (standard tier, Sun and Moon), calendar, time, panchanga, intl, serial |
| `kundali` | `panchanga` plus chart, houses, vargas, state, aspect, points, strength, dasha, rules, jaimini, interpret |
| `baseline-parity` | everything P0 |
| `full` | everything |

A consumer registering Teimeris drops `ephemeris-builtin` from their
build; the size report shows the difference. Budgets per profile are in
`09-performance-architecture.md`; dependency assertions (`panchanga` never
depends on `chart`; `numerology` on nothing astronomical) are gated.

## Rules for adding a module

`09-guidelines/03-adding-a-module.md`.
