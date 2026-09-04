# Scope

Status: `accepted`, 2026-09-04. The minimum bar is baseline parity, and Q4 is
decided: v1.0 is baseline parity, with the Western and Hellenistic module
families designed in Phase 0 and shipped in v1.x (ADR-0005).

## The bar: replace the baseline engine's `packages/` folder

Everything the baseline engine's seven engine packages compute must be computable through
the SDK, with a migration path for its persisted artefacts. The inventory is
in `01-research/baseline-engine/00-inventory.md` and the parity checklist in
`02-parity-requirements.md`. In summary, v1.0 must provide:

| area | what the baseline engine has today |
|---|---|
| foundation | entity registry with four-language names, angles and JD utilities, decimal boundaries, sunrise with modes, ghati-pala, timezone resolution with LMT and replay-safe metadata, BS calendar, dual siddhanta (drik, Surya Siddhanta) |
| chart | foundation-then-slices pipeline, 22 house systems, vargas, dignities, avasthas, aspects, upagrahas, special lagnas, 562 yogas and 62 doshas as data, Shadbala, Ashtakavarga, Bhava Bala, 18 dasha systems with depth and balance options, Jaimini (karakas, arudhas, three pairs), Ayurdaya, Maraka, longevity windows, chart serialiser with presets and a gochar sidecar |
| panchanga | daily panchanga with root-found transitions, hora, choghadiya, Rahu kaal and kin, Abhijit, Panchaka, Disha shool, muhurta yogas, muhurta search with blackouts and event rules |
| predictive | gochar overlays, transit aspects, Sade Sati, transit strength via Ashtakavarga, phala with vedha, transit calendar, Tajika |
| jataka | KP, Ashta Koot milan with extended checks and marriage doshas, quick milan from name, Bayesian rectification, prashna, Lal Kitab, numerology, namakarana with phonetic matching, Pancha Pakshi, remedies, research (synastry, composites, statistics) |
| interpret | 38 state tables, yoga and dosha texts, deterministic composers for milan, placements, dasha, day, muhurta, namakarana, tara chakra, yoga timing |
| rashifal | transit context with ingresses and stations, per-rashi overlays, scoring |
| localisation | ne, en, sa, hi throughout |

## In scope for v1.0

1. The foundation and ports: context, settings and profiles, ephemeris port
   with capabilities (positions required, overrides optional), calendar
   port with Gregorian, Julian, mixed, ISO week, BS and the Indian
   lunisolar calendar, timezone port with embedded tzdb, intl-data port
   with the four core packs.
2. The astronomy layer inside the SDK: timescales, Delta T, precession,
   nutation, sidereal time, frame completion, topocentric correction, the
   full ayanamsha catalogue, every house system, rise and set, crossings and
   stations (Q8, ADR-0009).
3. The built-in analytic ephemeris in three tiers, so the SDK works with
   nothing else installed (Q7, ADR-0008).
2. Every module above, re-derived from the baseline engine behaviour with the
   variants named and the known defects fixed.
3. Bindings: Node native, wasm, Dart/Flutter, Python, Rust; C and C++
   headers as the base (Q3).
4. Golden-vector conformance against the baseline engine and, where available,
   against JHora and Parashara's Light exports.
5. The docs site, the API reference, and the guidelines for adding a
   language, a calendar and a module.
6. CI/CD producing installable packages for every binding.

## Designed in v1.0, shipped in v1.x

- Western foundations: tropical zodiac end to end, aspect model with orbs and
  applying/separating, midpoints and dials, secondary progressions, solar arc
  and primary directions, solar and lunar returns, synastry and composite
  (Davison), fixed stars and parans, Arabic parts, declinations, relocation
  and astro-lines.
- Hellenistic and medieval: sect, lots, essential dignity tables and almutens,
  profections, zodiacal releasing, firdaria, decennials, planetary hours,
  horary considerations.
- Additional calendars: Nepal Sambat, Saka national, Tamil, Malayalam,
  Bengali, Hijri, Hebrew, Persian, Chinese.
- Additional dasha systems from the JHora and PyJHora catalogue beyond
  the baseline engine's eighteen.
- Eclipses and the full fixed-star catalogue in the astronomy layer (Q22).
- Additional bindings: Java, Swift, Kotlin.

## Out of scope

- Any external ephemeris: Teimeris, Swiss and others are adapters
  published separately; the SDK ships only its own analytic provider.
- User interface, chart drawing beyond geometry helpers, persistence, HTTP.
- LLM prompting and generation (the baseline engine's AI layer stays in the baseline engine; the SDK
  provides the deterministic dossier serialisation it consumes).
- Vastu, palmistry, tarot and other non-computational divination.
- Chinese BaZi and Zi Wei Dou Shu, Tibetan and Burmese systems: researched in
  `01-research/feature-universe/17-other-traditions.md` for the sake of the
  abstractions, scheduled for v2 at the earliest.
