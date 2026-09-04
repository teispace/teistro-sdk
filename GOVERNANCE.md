# Governance

The long form is `docs/09-guidelines/07-open-source-governance.md`.

## Roles

| role | who | what |
|---|---|---|
| maintainer | Teispace, represented by the accounts in `CODEOWNERS` | final say on scope, releases and licence; approves RFCs and ADRs |
| core contributor | people with sustained merged work in an area | review rights on that area via `CODEOWNERS` |
| contributor | anyone | pull requests under the DCO |

## Decisions

- Small changes: a pull request with the fast check green.
- Significant changes: an RFC in `rfcs/`, open for comment, accepted by the
  maintainer, then an architecture decision record in `docs/08-decisions/`
  and the implementation.
- Accuracy disputes: an issue with reproducing data; the conformance
  harness decides; classical citations settle definitions.
- Decisions live in the repository, never only in chat.
  `docs/STATUS.md` is the public state of the project.

## Branches and reviews

`main` is protected: pull requests only, the fast check required, linear
history, no force pushes. Required reviews are enabled as soon as a second
maintainer account exists; until then the maintainer merges.

## Releases

Semantic versioning with the compatibility contract in
`docs/02-architecture/06-api-conventions.md`. A release is a tag; the
changelog leads with **Numbers**: which outputs moved and by how much.
