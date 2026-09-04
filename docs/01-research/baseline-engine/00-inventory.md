# baseline engine: inventory

Status: `research`, 2026-09-04. From a full read of
the baseline engine repository (private) at version 0.6.120 (Node 24,
Yarn 4, NestJS 11, seven engine packages). This is the minimum bar for the
SDK and the source of its first golden vectors.

## Engine packages and what they hold

| package | depends on | contents |
|---|---|---|
| the baseline `core` package | none | `AstronomicalBackend` seam with a Node implementation over `sweph` 2.10.3 (files 1800–2399, Moshier fallback flagged per position, sidereal, speed, optional topocentric), a siddhanta router (`drik`, `surya` with a custom Surya Siddhanta engine and bija option), 47 ayanamshas, decimal.js precision boundaries, timezone resolver (geo-tz, date-fns-tz, LMT, manual, DST gap throws, ambiguity earlier, replay-safe metadata), sunrise service with modes, ghati-pala (civil and proportional), BS calendar (table, official or computed source), entity registry with four-language names (grahas, rashis, nakshatras with padas and akshars, tithis, karanas, yogas, varas, vargas, dignities, states, deities, ayanamshas, samvatsars, avakhada), locale and numeral services, calendar service with sankranti-aware era numbers |
| the baseline `interpret` package | core | 38 state-interpretation tables, yoga and dosha interpretation records merged with collision checks, a namespaced facade, milan resolver and verdict composer, graha-bhava composer (108 cells plus lagna, condition modifiers, conjunction synthesis), dasha narrative composer, day narrative, muhurta narrative, namakarana note, tara chakra, yoga timing composers |
| the baseline `chart` package | core, interpret | foundation-then-slices pipeline, 22 house systems with degeneracy checks, Bhava-Chalit (measured Vehlow), vargas, dignities, avasthas, aspects, upagrahas, special lagnas, 562 yoga and 62 dosha rules evaluated by one algebraic evaluator, Shadbala, Ashtakavarga (classical, zero, transfer ekadhipatya), Bhava Bala, 18 dasha systems with depth knobs and spatial or temporal balance, dasha timeline, Jaimini slice, Ayurdaya, Maraka, longevity windows, birth timing (Ishtakaal, bhayat, bhabhoga), transit service, serialiser 1.4.0 with presets and date-free persistence, gochar sidecar, dossier composer |
| the baseline `panchanga` package | core, chart | daily panchanga (root-found tithi, nakshatra, yoga, karana transitions; vara; lunar month; Rahu kaal, Yamaganda, Gulika; choghadiya; hora; Abhijit; Panchaka; Disha shool; Chandra niwas; five muhurta yogas; Moon sign; Sun ayana), timing service with next and previous change finders, muhurta finder, blackout service, window assembler, lagna and karaka evaluators, natal scorer, event rules, permitted nakshatra map |
| the baseline `predictive` package | core, chart | gochar kundali overlays, transit aspects, Sade Sati detection, transit strength via Ashtakavarga, phala with vedha, transit calendar, Tajika (whole-sign solar return) |
| the baseline `jataka` package | core, chart | KP (249 sub-lords, four levels, significators, ruling planets, Placidus with equal fallback, ayanamsha warning), milan (Ashta Koot with cancellations, extended Rajju, Vedha, Mahendra, Stree Deergha, Mangal match, marriage doshas, quick milan, akshar resolver), rectification (Bayesian cascade with Tattwa prior and dasha-boundary stage, refinement, HPD intervals, hold-out), prashna, Lal Kitab, numerology (Pythagorean, Chaldean), namakarana (phonetic matcher, validation, offline corpus of 25,096 lines, LLM path assembler), Pancha Pakshi, remedies (functional benefics, gemstone safety, priorities, four-language reasons), research (batch, synastry, composites, statistics) |
| the baseline `rashifal-core` package | core, chart, panchanga, predictive | transit context per period (snapshot near 6 AM local at the midpoint, ingresses, stations), per-rashi overlays with favourable houses, vedha and Sade Sati, scoring with planet weights and normalisation, life areas, lucky elements, prompt and fallback constants |

## Settings snapshot the baseline engine persists per chart

ayanamsaId, zodiacMode, houseSystem, siddhanta, nodeType, sunriseMode,
dashaBalanceMethod, charaKarakaSystem (7 or 8), ekadhipatyaMethod,
useTopocentric, suryaBija, plus the version triple (engine 1.8.0, Swiss
2.10.03, serializer 1.4.0). The SDK's settings model must be a superset.

## Guards and conventions worth keeping

- Package DAG enforced by a cycle checker; browser-safety boundary enforced
  by a script (`sweph`, geo-tz and Node built-ins only behind `core/node`).
- Every interpretation record has four languages and citations, enforced.
- Results carry versions and the ephemeris source per position.
- Settings are snapshotted and hashed into a content hash that makes chart
  persistence idempotent.
- The persisted dossier is date-free; date-aware views are derived at read
  time; transits are recomputed from a natal sidecar.

## Scale figures (for sizing the port)

| item | count |
|---|---:|
| yoga rules | 562 |
| dosha rules | 62 |
| dasha systems | 18 |
| house systems offered | 22 |
| ayanamshas | 47 |
| state interpretation tables | 38 |
| entity data files in core | 16, the largest 1,757 lines (nakshatras) |
| name corpus (namakarana) | 25,096 lines |
| chart section routes | 38 |
