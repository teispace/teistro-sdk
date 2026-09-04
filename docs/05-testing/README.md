# Testing

Status: `planned`, 2026-09-04. The strategy is in
`01-research/platform/09-testing-quality.md`; this directory will hold the
detailed plans and the generated reports.

## The bar

[`01-quality-bar.md`](01-quality-bar.md) is binding (ADR-0015): the gates
for correctness, robustness, performance, memory and code quality that a
module must pass to be done.

## Planned contents

| page | contents |
|---|---|
| `01-golden-vectors.md` | how vectors are exported from the baseline engine (a script in the baseline engine repository producing JSON per computation with settings and versions), from PyJHora, and hand-entered from JHora and PL printouts and classical texts; the fixture format with the settings hash asserted and a citation per expected value; the move of `fixtures/` into the separate CC0 conformance repository before Phase 1 exits (ADR-0022) |
| `02-conformance-harness.md` | the runner, the central tolerance file keyed by field and provider class, deliberate-difference registry, the cross-architecture hash matrix, generated `CONFORMANCE.md` |
| `03-property-tests.md` | invariants per module |
| `04-cross-binding-parity.md` | canonical JSON emission per binding, the diff gate |
| `05-provider-conformance-kit.md` | the kit shipped to consumers |
| `06-fuzzing.md` | targets, corpus, schedule |
| `07-benchmarks.md` | harness, baselines, results schema |
| `08-coverage.md` | floors per crate and the gate |
| `09-type-safety.md` | `trybuild` compile-fail tests, the strictness consumer projects per binding, the shared validator corpus (ADR-0023) |
| `perf/` | profiling findings, one file per investigation, so the same investigation is not repeated |
| `ACCURACY.md`, `CONFORMANCE.md`, `SIZES.md` | generated |

## Gate list (initial)

format, clippy, determinism lints, dependency policy (`cargo deny`),
containment build, unit, whole-table invariants, golden, property,
snapshot, compile-fail, parity, cross-architecture hashes, provider kit,
generated-artefact diff, dependency DAG, key catalogue consistency, pack
validation, docs examples, docs links, size per profile, wasm size delta,
install check per binding, benchmark regression, instruction-count
regression, mutation score, feature matrix, semver check, sanitizers,
Miri, fuzz smoke, version sites, changelog "Numbers" section present
with the calculation version impact, SBOM, licence headers.
