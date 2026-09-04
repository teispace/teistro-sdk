# Modularity and tree-shaking

Status: `research`, 2026-09-04. Feeds ADR-0005 and
`02-architecture/01-module-catalog.md`.

## What "ship only what you use" means per platform

| platform | code | data |
|---|---|---|
| Rust | one crate per module in a Cargo workspace; features for optional sub-capabilities; the linker drops unused code; `no_std` core possible | baked packs selected by feature; blob packs at runtime |
| Node native | one native addon per module family is too many binaries; instead one addon built per **profile** (a named set of modules) plus the JavaScript side split into ESM subpath packages (`@teistro/chart`, `@teistro/panchanga`) with `sideEffects: false` so bundlers shake the JS; the native library is built with the modules the profile needs | packs as separate npm packages (`@teistro/locale-ne`) loaded at runtime |
| WebAssembly | a wasm binary cannot be tree-shaken by a bundler; the answer is **multiple prebuilt wasm variants per profile** (`core+panchanga`, `core+chart`, `full`) plus a build tool (`teistro-build`) that produces a custom wasm for an exact module set, the way `icu4x-datagen` produces exact data; `wasm-opt` and `panic=abort` for size; lazy instantiation of the wasm module | packs as fetched blobs |
| Dart and Flutter | Dart tree-shakes unused Dart code; the native library follows the profile; Flutter plugin builds the native library from source with features set by the consumer's pubspec configuration | packs as assets |
| Python | wheels per profile, or one wheel with the full library (size matters less on servers) | packs as package data or runtime files |
| Java | one JAR with the full library, FFM bindings | packs as resources |

## Module granularity

Too fine and the dependency graph and the release matrix explode; too
coarse and nothing is shakable. The research suggests **module families**
as the shipping unit (eight to twelve families) with **cargo features**
inside a family for optional pieces:

| family | contains | optional features inside |
|---|---|---|
| `core` | types, keys, angles, time, settings, errors, ports, entity catalogue | none |
| `astro` | the astronomy layer above raw positions | model variants |
| `ephemeris-builtin` | the analytic provider | tiers `compact`, `standard`, `full` |
| `calendar` | Gregorian, Julian, BS, Indian lunisolar, eras, tz provider | each extra calendar |
| `chart` | positions, houses, vargas, dignities, state, aspects, points | Surya Siddhanta model, extended vargas |
| `strength` | Shadbala, Ashtakavarga, Bhava bala, Vimshopaka | Tajika balas |
| `dasha` | registry and the 18 systems | each additional system |
| `rules` | rule engine, yoga and dosha packs | additional packs |
| `panchanga` | day, limbs, timings, muhurta, blackouts | festivals rule pack, saait |
| `predictive` | gochar, transits, Sade Sati, calendar, Tajika, hit lists | Western predictive later |
| `jataka` | KP, matching, prashna, rectification, longevity, remedies, numerology, Lal Kitab, Pancha Pakshi, namakarana, research | each as a feature |
| `interpret` | state and rule interpretation, composers | each composer |
| `intl` | Teistro Intl engine, MF2 subset, numerals, transliteration | each script |
| `serial` | JSON, dossier text, blob, chart geometry | |
| `western`, `hellenistic` (v1.x) | | |

Profiles are named sets of families: `panchanga-only`, `kundali`, `full`,
`baseline-parity`. Each profile is a build target with its own size budget in
the gate.

## Dependency rules

- A DAG, enforced by a gate (the baseline engine and Teimeris both have one).
- `core` depends on nothing; `intl` depends on `core` only; no computation
  module depends on `intl` or `interpret`; `serial` depends on everything
  it serialises but nothing depends on `serial`.
- Every module declares which ports it needs and which capabilities of each.
- Cross-module data passes through `core` types only.

## Extension without modification

The plug-in points (each is a registry in `core` with a stable interface):
dasha systems, house systems, varga schemes, calendars, timezone data,
ephemeris providers, geo providers, locale packs, interpretation packs, rule
packs, remedy packs, muhurta activity rules, matching tables, points
catalogues. Registration happens at context creation; the context is the
unit of configuration, never a global.

## Measuring it

A size gate per profile per platform (compressed bytes), a "what did I ship"
report generated from the build, and an install check that runs each
profile in a throwaway project, as Teimeris's `install_check.py` does.
