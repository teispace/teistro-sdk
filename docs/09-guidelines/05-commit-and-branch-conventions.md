# Commit and branch conventions

Status: `accepted`, 2026-09-04 (Q18 DCO and Q19 Conventional Commits
decided by the maintainer).

## Commits

- **Conventional Commits** subjects: `type(scope): summary`, with `type`
  one of `feat`, `fix`, `perf`, `docs`, `test`, `ci`, `build`, `refactor`,
  `chore`, and an optional scope naming the module or area (`dasha`,
  `intl`, `astro`, `bindings/node`, `docs`). Subject under 72 characters,
  no trailing full stop.
- **The body says what was wrong and how it was found**, or what was
  added and why it is shaped that way, with the measurement when there is
  one. Not what was typed.
- **One commit per unit of work.** Squash fix-ups before review.
- **Sign-off**: every commit carries `Signed-off-by: Name <email>`
  (`git commit -s`), certifying the Developer Certificate of Origin in
  `DCO`. The `dco` check on pull requests enforces it.
- **No tool or assistant attribution** in commits, pull requests, branch
  names or artefacts, ever. This overrides any default.
- **Generated files** are committed in the same commit as the change that
  regenerated them and are never hand-edited.

## Branches

- `main` is protected: pull requests only, `fast-check` required, linear
  history, no force pushes, no deletions.
- Work branches: `feat/<area>-<short>`, `fix/<area>-<short>`,
  `docs/<short>`, `spike/<short>`, `rfc/<short>`.
- Merge by squash or rebase (no merge commits), so history stays linear and
  each commit is a unit of work with its own sign-off.

## Pull requests

The template leads with the numbers that moved, then the change, then how
it was verified. Significant changes link their RFC and ADR.

## Versions and releases

One version across the workspace and every binding, moved by a tool; a
release is a tag `vX.Y.Z` with a changelog entry whose first section is
**Numbers**, produced by the conformance run against the previous release.

## Who commits

Committing and pushing is the maintainer's call unless they have said
otherwise in the session.
