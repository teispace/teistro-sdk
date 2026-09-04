# CI/CD

Status: `planned`, 2026-09-04. The research is in
`01-research/platform/11-cicd-release.md`.

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
