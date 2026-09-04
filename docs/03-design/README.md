# Design

Status: `planned`, 2026-09-04. Per-module detailed designs are written in
Phase 1 onward, one page per module, after the Phase 0 decisions.

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

## Planned pages (in roadmap order)

| page | phase |
|---|---|
| `core-types-and-catalogue.md` | 1 |
| `settings-and-profiles.md` | 1 |
| `ephemeris-port-and-adapters.md` | 1 |
| `time-and-timezone.md` | 1 |
| `calendar-gregorian-julian.md`, `calendar-bikram-sambat.md`, `calendar-indian-lunisolar.md` | 1, 2 |
| `intl-engine-and-packs.md` (Teistro Intl) | 1 |
| `astro-timescales-and-frames.md`, `astro-ayanamsha-catalogue.md`, `astro-house-systems.md`, `astro-events-and-crossings.md` | 2 |
| `ephemeris-builtin.md` (theories, tiers, ingestion tool) | 3 |
| `chart-foundation.md`, `houses.md`, `vargas.md`, `state.md`, `aspect.md`, `points.md` | 4 |
| `panchanga-day.md` | 4 |
| `strength-shadbala.md`, `strength-ashtakavarga.md`, `strength-bhava-vimshopaka.md` | 3 |
| `dasha-registry.md` plus one page per seed kind | 3 |
| `rules-engine.md`, `rules-yoga-dosha-packs.md` | 4 |
| `interpret-composers.md` | 4 |
| `jaimini.md`, `kp.md`, `tajika.md`, `gochar.md`, `muhurta.md` | 5 |
| `matching.md`, `prashna.md`, `rectification.md`, `longevity.md`, `remedies.md`, `numerology.md`, `lalkitab.md`, `pakshi.md`, `namakarana.md`, `rashifal.md`, `research.md` | 6 |
| `serial-json-dossier-blob.md`, `chart-geometry.md` | 7 |
| `western-*.md`, `hellenistic-*.md` | v1.x |
