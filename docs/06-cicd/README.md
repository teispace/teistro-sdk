# CI/CD

Status: `planned`, 2026-09-04. The research is in
`01-research/platform/11-cicd-release.md`.

## Planned contents

| page | contents |
|---|---|
| `01-pipelines.md` | fast check, full verify, release, docs, scheduled; triggers and cost policy |
| `02-build-matrix.md` | per binding and platform: toolchains, cross-compilation, artefacts |
| `03-release-process.md` | version bump tool, changelog "Numbers" rule, tagging, signing, provenance, SBOM, publishing or hand-over per the licence decision |
| `04-local-verify.md` | `cargo xtask verify` modes and the Linux container (a Dockerfile that runs the same task) that reproduces the Linux matrix |
| `05-docs-deploy.md` | site build and deploy |

## Principles

- CI is a release gate; the fast check runs on every push, the full matrix
  nightly and on tags (cost control).
- A skip is not a pass: the summary names what did not run.
- Every gate is proven red once and the run is recorded.
- Artefacts are installed into throwaway projects and run before they are
  published.
