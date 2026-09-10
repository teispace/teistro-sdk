# Architecture decision records

Status: living index, revised 2026-09-04 (records 0016 to 0023 added). One record per decision, numbered, never
edited after acceptance (a change is a new record that supersedes the old;
a record still `proposed` may be revised). A record moves from `proposed`
to `accepted` when the maintainer confirms it in `QUESTIONS.md`.

| ADR | title | status | question |
|---|---|---|---|
| [0001](adr-0001-core-language.md) | Rust core with a stable C ABI | accepted 2026-09-04 | Q1 |
| [0002](adr-0002-ephemeris-agnostic-port.md) | Ephemeris-agnostic through a capability-negotiated port; positions required, overrides optional | accepted 2026-09-04 (revised) | Q7, Q8, Q15 |
| [0003](adr-0003-keys-not-strings.md) | Engine emits keys; localisation is data packs | accepted 2026-09-04 | Q10 |
| [0004](adr-0004-binding-generation.md) | One API description, generated bindings, parity gate; generator chosen by the Phase 0 spike | accepted 2026-09-04 | Q2, Q3 |
| [0005](adr-0005-modularity.md) | Module families as crates and packages; profiles; size gates; v1.0 is baseline parity | accepted 2026-09-04; the v1-scope clause superseded by ADR-0025, the modularity decision unchanged | Q4 |
| [0006](adr-0006-licence.md) | Apache-2.0 open core; packs and adapters under their own terms | accepted 2026-09-04 | Q5 |
| [0007](adr-0007-binding-toolchain.md) | The binding toolchain: option A, a designed C ABI, an extracted API description and generators of our own; Diplomat rejected on the spike's measurements (no host callbacks in its JavaScript and Dart backends, accessor-only trees, wasm-only JavaScript) | accepted on the spike, 2026-09-05 | Q2 |
| [0008](adr-0008-builtin-ephemeris.md) | A built-in analytic ephemeris ships in v1 | accepted 2026-09-04; extended by 0021 | Q7 |
| [0009](adr-0009-sdk-native-astronomy.md) | The SDK owns the astronomy layer above raw positions | accepted 2026-09-04 | Q8, Q15 |
| [0010](adr-0010-teistro-intl.md) | Teistro Intl, one opinionated localisation standard | accepted 2026-09-04 | Q10 |
| [0011](adr-0011-precision-policy.md) | Precision policy | accepted 2026-09-04; amended by 0016 | Q13 |
| [0012](adr-0012-docs-site-fumadocs.md) | Documentation site on Fumadocs | accepted 2026-09-04 | Q11 |
| [0013](adr-0013-override-policy-and-tiers.md) | Provider override policy default; built-in ephemeris tiers; eclipses and stars in v1.x | accepted 2026-09-04; extended by 0021 | Q17, Q22, Q23 |
| [0014](adr-0014-rust-only-tooling.md) | Everything we author is Rust, including the tooling (`cargo xtask`) | accepted 2026-09-04 | Q25 |
| [0015](adr-0015-quality-bar.md) | The quality bar is gated: tests, benchmarks, memory and leak checks are part of done | accepted 2026-09-04 | maintainer instruction |
| [0016](adr-0016-exact-classification-and-periods.md) | Exact classification and period arithmetic: nanoarcsecond angles, integer classification, rational dasha spans (amends 0011) | accepted 2026-09-04 | Q26 |
| [0017](adr-0017-kernel-and-table.md) | Kernel and table: one kernel per family, systems as data rows, falsified before code, lazy dasha cursor | accepted 2026-09-04 | Q27 |
| [0018](adr-0018-evidence-ranks-and-mark-and-continue.md) | Evidence ranks, confidence marks, and "mark and continue" | accepted 2026-09-04 | Q28 |
| [0019](adr-0019-clean-room-and-licence-containment.md) | Clean room and licence containment: allow list, oracles unpublished, adapters outside, test-provider build | accepted 2026-09-04 | Q29 |
| [0020](adr-0020-calculation-version-and-provenance.md) | Calculation version and the provenance envelope | accepted 2026-09-04 | Q30 |
| [0021](adr-0021-reference-ephemeris-path.md) | The reference-accuracy ephemeris path: ERFA port, DE reader, `reference` tier (extends 0008, 0009, 0013) | accepted 2026-09-04 | Q31 |
| [0022](adr-0022-determinism-and-conformance-repository.md) | The determinism contract and the conformance repository (extends 0015) | accepted 2026-09-04 | Q32 |
| [0023](adr-0023-type-safety-in-every-binding.md) | Type safety is a correctness feature, in every binding: newtypes, generated typed surfaces, suggestions from the description, robust boundaries | accepted 2026-09-04 | Q33 |
| [0024](adr-0024-the-default-profile.md) | The default profile is `parashari-classical`, the texts as read, inheriting nothing else | accepted 2026-09-07 | Q34 |
| [0025](adr-0025-v1-is-the-whole-feature-universe.md) | v1.0 is the whole feature universe: Western and Hellenistic move from v1.x into v1 (supersedes ADR-0005's v1-scope clause) | accepted 2026-09-10 | Q4 (revised) |
| [0026](adr-0026-chart-geometry-and-the-first-party-renderer.md) | Chart layouts as cited rows a consumer can extend, geometry in Phase 4, and an optional first-party SVG renderer above the core | accepted 2026-09-10 | Q36 |
| [0027](adr-0027-the-lunar-theory-and-its-bija.md) | ELP2000-82B with a quadratic bija; tier bounds per body and per span; `reference` budget raised to 4 MB (amends 0013 and 0021) | accepted 2026-09-10 | Q37 |

## Template

```
# ADR-NNNN: title

Status: proposed | accepted | superseded by ADR-MMMM
Date:
Question: Qn

## Context
## Decision
## Consequences
## Alternatives considered
## Evidence
```
