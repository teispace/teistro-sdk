# Governance

The long form is `docs/09-guidelines/07-open-source-governance.md`.

## Roles

| role | who | what |
|---|---|---|
| maintainer | Teispace, represented by the accounts in `CODEOWNERS` | final say on scope, releases and licence; approves RFCs and ADRs |
| core contributor | people with sustained merged work in an area | review rights on that area via `CODEOWNERS` |
| contributor | anyone | pull requests under the DCO |
| tradition reviewer | a practitioner of a tradition (Parashari, Jaimini, KP, Nepali practice, a South Indian tradition, Western, and so on), credited in the repository | review authority over rule packs, table rows, citations and fixtures for that tradition; a crux for that tradition is closed only with their sign-off; recruited as the traditions ship |

## Decisions

- Small changes: a pull request with the fast check green.
- Significant changes: an RFC in `rfcs/`, open for comment, accepted by the
  maintainer, then an architecture decision record in `docs/08-decisions/`
  and the implementation.
- Accuracy disputes: an issue with reproducing data; the conformance
  harness decides; classical citations settle definitions.
- Doctrine: the project does not arbitrate it. Where classical
  authorities differ, every documented convention ships as a selectable
  variant and the result records which ran; "which is correct" is out of
  scope, "which text says what" is in scope and needs a citation
  (chapter and verse). A rule or table change without one is not merged.
- Evidence has rank (`CLEAN_ROOM.md`, ADR-0018): a third-party
  implementation's disagreement with the baseline engine is a question on
  the cruxes page, not a correction.
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
