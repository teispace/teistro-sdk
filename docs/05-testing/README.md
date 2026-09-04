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
| `01-golden-vectors.md` | how vectors are exported from the baseline engine (a script in the baseline engine repository producing JSON per computation with settings and versions), from PyJHora, and hand-entered from JHora and PL printouts and classical texts; the fixture format; tolerance policy per field |
| `02-conformance-harness.md` | the runner, per-module tolerance tables, deliberate-difference registry, generated `CONFORMANCE.md` |
| `03-property-tests.md` | invariants per module |
| `04-cross-binding-parity.md` | canonical JSON emission per binding, the diff gate |
| `05-provider-conformance-kit.md` | the kit shipped to consumers |
| `06-fuzzing.md` | targets, corpus, schedule |
| `07-benchmarks.md` | harness, baselines, results schema |
| `08-coverage.md` | floors per crate and the gate |
| `ACCURACY.md`, `CONFORMANCE.md`, `SIZES.md` | generated |

## Gate list (initial)

format, clippy, unit, golden, property, snapshot, parity, provider kit,
generated-artefact diff, dependency DAG, key catalogue consistency, pack
validation, docs examples, docs links, size per profile, install check per
binding, benchmark regression, sanitizers, Miri, fuzz smoke, version sites,
changelog "Numbers" section present, SBOM, licence headers.
