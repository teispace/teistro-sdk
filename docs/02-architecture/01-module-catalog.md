# Module catalogue

Status: `draft`, revised 2026-09-04 after Q7, Q8 and Q10: the `astro`
layer, the built-in ephemeris and Teistro Intl are added; houses, points,
panchanga and gochar depend on `astro`, not on provider capabilities.
Derived from the feature pages in `01-research/feature-universe/`. Each
module is a crate in the Rust workspace and a package (or a feature of a
family package) in every binding.

## L0 and L1

| module | contents | depends on | baseline source |
|---|---|---|---|
| `core` | keys and entity catalogue (grahas, rashis, nakshatras with padas, tithis, karanas, yogas, varas, vargas, dignities, states, ayanamshas, house systems, samvatsaras), angles, Julian day and timescales, settings and profiles, result envelope and provenance, errors, capabilities, registries, limits | none | `core` data, enums, types |
| `port-ephemeris` | the ephemeris port trait (positions required; overrides optional), request and response types, capability descriptor, provider conformance kit | `core` | `AstronomicalBackend` |
| `port-calendar`, `port-timezone`, `port-geo`, `port-intl-data`, `port-log` | as before | `core` | |

## L1.5 astronomy

| module | contents | depends on | tier | notes |
|---|---|---|---|---|
| `astro` | timescales and Delta T models, sidereal time, precession (11 models), nutation (5), obliquity, frame bias, frame completion (light-time, aberration, deflection, nutation), topocentric parallax, coordinate transforms and refraction, the ayanamsha catalogue (47 plus custom), house systems (all), rise/set/transit solver, crossings and stations search, equation of time, star table (anchors and yogataras), eclipses (v1.x) | `core`, `port-ephemeris` | P0 | see `01-research/platform/13-astronomy-layer.md`; conformance against Teimeris |
| `ephemeris-builtin` | analytic ephemeris implementing the port: VSOP87 planets, ELP/MPP02 Moon, fitted Pluto, nodes and apogees, analytic speeds; tiers `compact`, `standard`, `full`; generated tables with citations | `core`, `port-ephemeris` | P0 (own phase) | see `14-builtin-ephemeris.md`; removable by tree-shaking |
| `siddhanta` | Surya Siddhanta positions with bija, as a provider | `core`, `port-ephemeris` | P0 | The baseline engine |

## L2 domain modules

| module | contents | depends on | tier | baseline source |
|---|---|---|---|---|
| `calendar` | Gregorian, Julian, mixed, ISO week, Bikram Sambat (table plus computed), Indian lunisolar (Amanta, Purnimanta, adhika, kshaya, solar months), eras and samvatsara; later Nepal Sambat, Saka, regional solar, Hijri, Hebrew, Persian, Chinese | `port-calendar`, `astro` (lunisolar) | P0 | `BsCalendarService`, `CalendarService` |
| `time` | civil to instant resolution, DST policies, LMT, ghati-pala, sunrise-anchored day, planetary hours | `port-timezone`, `astro` | P0 | timezone resolver, ghati-pala, birth timing |
| `chart` | birth data, foundation (positions in both frames with speeds, declinations, RA; cusps; frames), chart kinds | `astro`, `time` | P0 | `ChartFoundation` |
| `houses` | placement policy, Bhava-Chalit variants (Sripati, Vehlow, Porphyry, KP), placements by span, whole sign, equal, degeneracy states | `chart`, `astro` | P0 | `HouseService` |
| `vargas` | standard and extended vargas with named variants, custom D-N, mixed, vargottama, varga change search | `chart`, `astro` (crossings) | P0 | `VargaService` |
| `state` | dignities, relationships, combustion, retrogression, war, gandanta, avasthas, marana karaka sthana | `chart`, `vargas` | P0 | dignity, avastha services |
| `aspect` | graha drishti, sphuta drishti, rashi drishti, conjunction logic, Tajika aspects, orb-based aspects (shared engine) | `chart` | P0 | aspect services |
| `points` | special lagnas, upagrahas, Bhrigu bindu, yogi points, sphutas, bhagas, 64th navamsa and 22nd drekkana lords, sahamas and lots | `chart`, `time`, `astro` | P0 | upagraha, special lagna services |
| `strength` | Shadbala, Ishta and Kashta, Bhava Bala, Ashtakavarga, Vimshopaka, Vaiseshikamsa, Tajika balas | `chart`, `houses`, `vargas`, `state`, `aspect` | P0 | strength services |
| `dasha` | registry over four seed kinds; the 18 baseline systems; tree, timeline, active chain, balance methods, year lengths, depth control; plug-in interface | `chart`, `state` | P0 | dasha engines |
| `rules` | rule engine, rule pack format and loader, yoga and dosha packs, strength and cancellation, timing relations, batch search | `chart`, `houses`, `vargas`, `state`, `aspect`, `strength`, `dasha` | P0 | yoga and dosha evaluator |
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
| `intl` | Teistro Intl: pack engine, MF2 subset, formatting functions bound to SDK types, numerals, transliteration, fallback chains, runtime overrides, the `teistro-intl` CLI as a library | `core`, `port-intl-data`, `calendar` | P0 | registry names, locale service, numeral formatter, phonetic matcher |
| `serial` | canonical JSON, dossier text with presets and date-free persistence, gochar sidecar, binary blob, content hashes, chart layout geometry, dasha timeline shapes | all it serialises | P0 | chart serialiser, layout code |

## Profiles (shipping sets)

| profile | modules |
|---|---|
| `panchanga` | core, ports, astro, ephemeris-builtin (standard tier), calendar, time, panchanga, intl, serial |
| `kundali` | `panchanga` plus chart, houses, vargas, state, aspect, points, strength, dasha, rules, jaimini, interpret |
| `baseline-parity` | everything P0 |
| `full` | everything |

A consumer registering Teimeris drops `ephemeris-builtin` from their
build; the size report shows the difference.

## Rules for adding a module

`09-guidelines/03-adding-a-module.md`.
