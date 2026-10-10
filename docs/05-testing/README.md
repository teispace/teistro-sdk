# Testing

Status: `planned`, 2026-09-04; the quality bar is accepted and the
golden-vectors page is drafted. The strategy is in
`01-research/platform/09-testing-quality.md`; this directory will hold the
detailed plans and the generated reports. The corpus itself is in
`fixtures/`, a pinned submodule of
[`teispace/teistro-conformance`](https://github.com/teispace/teistro-conformance)
(see [`01-golden-vectors.md`](01-golden-vectors.md)).

## The bar

[`01-quality-bar.md`](01-quality-bar.md) is binding (ADR-0015): the gates
for correctness, robustness, performance, memory and code quality that a
module must pass to be done.

## Planned contents

| page | contents |
|---|---|
| [`01-golden-vectors.md`](01-golden-vectors.md) | **drafted**: the sources by rank, the spike 1 result (115 baseline fixtures over 55 charts, the searched boundary instants, the settings-profile variants, the tree-as-rows format, ten baseline conventions for the deliberate-difference registry), how the harness will consume the corpus, and the move of `fixtures/` into the separate CC0 conformance repository before Phase 1 exits (ADR-0022) |
| [`02-engine-findings.md`](02-engine-findings.md) | **active**: the register of discrepancies traced to the reference engine, each measured, filed upstream with a reproduction and assigned, with the bound the SDK holds it at meanwhile |
| [`ACCURACY.md`](ACCURACY.md) | **generated** by `cargo xtask accuracy` from [`accuracy-rows.yaml`](accuracy-rows.yaml) and the measurement tests, held by `check-accuracy`: every area of the astronomy layer with its target, what CI measures against the recorded engine tables on every run, the by-hand measurements with their dates, and the evidence (the Phase 2 exit artefact) |
| `03-conformance-harness.md` | the runner, the central tolerance file keyed by field and provider class, deliberate-difference registry, the cross-architecture hash matrix, generated `CONFORMANCE.md` |
| `04-property-tests.md` | invariants per module |
| `05-cross-binding-parity.md` | canonical JSON emission per binding, the diff gate |
| `06-provider-conformance-kit.md` | the kit shipped to consumers |
| `07-fuzzing.md` | targets, corpus, schedule |
| `08-benchmarks.md` | harness, baselines, results schema |
| `08-coverage.md` | floors per crate and the gate |
| `09-type-safety.md` | `trybuild` compile-fail tests, the strictness consumer projects per binding, the shared validator corpus (ADR-0023) |
| `perf/` | profiling findings, one file per investigation, so the same investigation is not repeated |
| `ACCURACY.md`, `CONFORMANCE.md`, `SIZES.md` | generated |

## The gates

The gates are not listed here, because a list beside the workflows goes
stale. [`01-quality-bar.md`](01-quality-bar.md) marks each requirement as
enforced, naming its gate, or as not yet enforced and taken up in the
hardening pass; `.github/workflows/fast-check.yml` and `verify.yml` are
the authority on what runs.
