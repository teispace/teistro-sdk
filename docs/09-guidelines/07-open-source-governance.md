# Open-source governance

Status: `accepted`, 2026-09-04. The SDK is public from its first commit
and built to be contributed to. These are the working rules; the files
they name live in the repository root.

## Roles

| role | who | what |
|---|---|---|
| maintainer | Teispace, represented by the accounts in `CODEOWNERS` | final say on scope, releases and licence; approves RFCs and ADRs |
| core contributor | people with sustained merged work | review rights on their areas via `CODEOWNERS` |
| contributor | anyone | pull requests under the DCO |

## Decision process

- Small changes: a pull request with the fast check green and, once a
  second maintainer account exists, one review.
- Significant changes (new module, API shape, default, binding, licence of
  a pack, dependency policy): an RFC in `rfcs/` using the template, open
  for comment, accepted by the maintainer, then an ADR and the
  implementation. Rejected RFCs are merged as rejected so the reasoning
  survives.
- Accuracy disputes: an issue with the reproducing data; the conformance
  harness decides; classical citations settle definitions.

## Contributor agreement

The Developer Certificate of Origin (Q18): a `Signed-off-by` line on every
commit, checked on pull requests. No CLA.

## Files

`LICENSE` (Apache-2.0), `NOTICE`, `DCO`, `CONTRIBUTING.md`,
`CODE_OF_CONDUCT.md` (Contributor Covenant 2.1), `SECURITY.md` (private
vulnerability reporting, three-day acknowledgement, ninety-day fix
target), `GOVERNANCE.md`, `CODEOWNERS`, `CHANGELOG.md`, issue templates
(bug, accuracy report, feature, documentation) and the pull request
template under `.github/`, `rfcs/0000-template.md`.

## Repository settings

Public; issues and discussions on; pull requests by squash or rebase only;
branches deleted on merge; `main` protected with the `fast-check` status
required, linear history, no force pushes or deletions; private
vulnerability reporting on; required reviews to be enabled when a second
maintainer account exists.

## Releases

Semantic versioning with the compatibility contract; a release is a tag;
the changelog leads with **Numbers**; artefacts are signed and carry
provenance and an SBOM; every binding's package installs from its artefact
in a throwaway project before it is published; the docs site deploys from
the tag.

## Communication

Issues, discussions and pull requests on GitHub; decisions in the
repository, never only in chat; `docs/STATUS.md` is the public state of
the project.
