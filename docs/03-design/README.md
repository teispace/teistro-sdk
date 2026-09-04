# Design

Status: `planned`, revised 2026-09-05. Per-module detailed designs are
written in Phase 1 onward, one page per module. Five pages were written
in Phase 0 because their content is retrofit-hostile (ADR-0016, ADR-0017):
the kernels are falsified against the catalogue before any code exists;
a sixth, the ephemeris port, was written from spike 3's measurements and
a seventh, Teistro Intl, from spike 4's.

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

## Drafted in Phase 0

| page | what it settles |
|---|---|
| [`exact-arithmetic.md`](exact-arithmetic.md) | nanoarcsecond angles, integer classification, rational periods (ADR-0016) |
| [`dasha-kernels.md`](dasha-kernels.md) | the udu and rashi kernels, 56 systems as rows with confidence marks, the lazy cursor |
| [`varga-kernel.md`](varga-kernel.md) | one table-driven evaluator for every divisional chart and variant |
| [`strength-schemes.md`](strength-schemes.md) | bala schemes with group membership as data, 18 components |
| [`rules-engine.md`](rules-engine.md) | the predicate algebra v2: references, table lookups, classifying outcomes, cancellation |
| [`ephemeris-port-and-adapters.md`](ephemeris-port-and-adapters.md) | positions required and overrides declared, the frame and its bits, columns instants outermost, the C vtable, frame completion by policy, the adapter rules, the kit's thirteen checks and bounds, Delta T as a table plus a model (spike 3) |
| [`intl-engine-and-packs.md`](intl-engine-and-packs.md) | the source conventions, the stable `MessageFormat 2` grammar and the SDK's functions with the types they imply, selection on entities and contexts, the validation gates, the `.tpack` container, the typed accessors for TypeScript and Dart (spike 4) |

## Planned pages (in roadmap order)

| page | phase |
|---|---|
| `core-types-and-catalogue.md` | 1 |
| `settings-and-profiles.md` | 1 |
| `time-and-timezone.md` | 1 |
| `calendar-gregorian-julian.md`, `calendar-bikram-sambat.md`, `calendar-indian-lunisolar.md` | 1, 2 |
| `astro-timescales-and-frames.md`, `astro-ayanamsha-catalogue.md`, `astro-house-systems.md`, `astro-events-and-crossings.md` | 2 |
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
