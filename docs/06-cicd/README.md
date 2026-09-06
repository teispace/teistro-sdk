# CI/CD

Status: `planned`, 2026-09-04; the cross-architecture hash matrix was
built 2026-09-06. The research is in
`01-research/platform/11-cicd-release.md`.

## What runs today

| workflow | when | what |
|---|---|---|
| `fast-check` | every push to `main`, every pull request | format, lint, the dependency policy, the workspace's tests, and every gate the Rust toolchain alone can run (`check-docs`, `check-fixtures`, `check-catalogue`, `check-calendars`, `check-time`, `check-accuracy`, `check-intl`, `check-ffi`); on a pull request, that every commit is signed off |
| `hash-matrix` | nightly and on demand | `cargo xtask hashes` on Linux x86-64, Linux aarch64 and macOS aarch64, and the three digests compared |

`cargo xtask hashes` walks a fixed scenario through the calendars, the
astronomy, the house systems and the classical model, and hashes every
value's bits per section. Every value is hashed as its bits, because a
difference of one unit in the last place is a difference: the point is to
find out whether the same source computes the same numbers on another
machine, not to decide how close is close enough. The report names the
section, so a difference says which layer moved rather than only that one
did. That is Phase 1's exit criterion, "hash-identical across x86-64 and
aarch64".

The gates that need another toolchain (`check-c`, `check-node`,
`check-dart`, `check-parity`) run by hand today and belong to the nightly
matrix when it is built.

## Planned contents

| page | contents |
|---|---|
| `01-pipelines.md` | fast check, full verify, release, docs, scheduled; triggers and cost policy |
| `02-build-matrix.md` | per binding and platform: toolchains, cross-compilation, artefacts |
| `03-release-process.md` | version bump tool, changelog "Numbers" rule, tagging, signing, provenance, SBOM, publishing or hand-over per the licence decision |
| `04-local-verify.md` | `cargo xtask verify` modes and the Linux container (a Dockerfile that runs the same task) that reproduces the Linux matrix; `cargo xtask ci` runs locally exactly what CI runs |
| `05-docs-deploy.md` | site build and deploy |

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
