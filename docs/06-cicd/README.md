# CI/CD

Status: `built`, 2026-09-06 (the docs site's deploy is still planned).
The research is in `01-research/platform/11-cicd-release.md`.

## What runs today

| workflow | when | what |
|---|---|---|
| `fast-check` | every push to `main`, every pull request | format, lint, the dependency policy, the workspace's tests, and every gate the Rust toolchain alone can run (`check-docs`, `check-fixtures`, `check-catalogue`, `check-calendars`, `check-time`, `check-accuracy`, `check-intl`, `check-ffi`, `check-lints`); on a pull request, that every commit is signed off |
| `hash-matrix` | nightly and on demand | `cargo xtask hashes` on Linux x86-64, Linux aarch64 and macOS aarch64; the two Linux runs compared value by value (a difference fails the job) and macOS reported against them (a difference is published, not failed) |
| `verify` | nightly, on demand, on a tag | the bindings' own gates (`check-c`, `check-node`, `check-dart`, `check-parity`) and `check-package` on all five platforms |
| `release` | a `v*` tag, or a dispatch that publishes nothing | five platforms built, merged, staged and published to npm, pub.dev and the release page |

`cargo xtask hashes` walks a fixed scenario through the calendars, the
astronomy, the house systems and the classical model, and hashes every
value's bits per section. Every value is hashed as its bits, because a
difference of one unit in the last place is a difference: the point is to
find out whether the same source computes the same numbers on another
machine, not to decide how close is close enough. The report names the
section, so a difference says which layer moved rather than only that one
did, and `cargo xtask compare-hashes` says how many values moved and by
how many places. That is Phase 1's exit criterion, "hash-identical across
x86-64 and aarch64", which the first run met; what the wider matrix
measures is in `05-testing/01-quality-bar.md`.

The gates that need another toolchain run by hand and in `verify`, which
is the nightly matrix: `check-c` needs a C compiler, `check-node` a Node
and a TypeScript, `check-dart` a Dart, `check-parity` both, and
`check-package` all three. A missing toolchain skips its own gate and
names what it wanted; a skip is not a pass, and the release's own run has
every toolchain installed.

## Planned contents

| page | contents | state |
|---|---|---|
| [`01-pipelines.md`](01-pipelines.md) | the four workflows, their triggers and the cost policy | built |
| [`02-build-matrix.md`](02-build-matrix.md) | the five platforms, what each produces, and how the packages are proved | built |
| [`03-release-process.md`](03-release-process.md) | one version, cutting a release, what the tag starts, provenance, what a consumer installs | built |
| `04-local-verify.md` | a `cargo xtask verify` that runs the whole matrix locally, and the Linux container that reproduces it | planned |
| `05-docs-deploy.md` | site build and deploy | planned |

## Principles

- CI is a release gate; the fast check runs on every push, the full matrix
  nightly and on tags (cost control); pull-request jobs are path-filtered
  (a locale-only change does not run the differential sweep) and the full
  gate always runs before merge to `main`.
- The fast check carries `cargo deny check` and, once an adapter exists,
  the containment build with the test provider only (ADR-0019).
- The nightly carries the cross-architecture determinism matrix comparing
  output hashes (ADR-0022), mutation testing, the full conformance run and
  the differential sweep against Teimeris and the recorded oracles.
- A skip is not a pass: the summary names what did not run.
- Every gate is proven red once and the run is recorded.
- Artefacts are installed into throwaway projects and run before they are
  published.
