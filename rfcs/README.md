# RFCs

Significant changes to Teistro SDK start here: a new module, a change to
an API shape, a change to a default (which decides someone's chart), a new
binding, a pack licence, a dependency policy. Small changes go straight to
a pull request.

## Process

1. Copy `0000-template.md` to `NNNN-short-title.md` with the next number.
2. Open a pull request. Discussion happens on the pull request.
3. The maintainer accepts, rejects or asks for revision. An accepted RFC is
   merged and gets an architecture decision record in
   `docs/08-decisions/`; the implementation follows in separate pull
   requests that link both.
4. A rejected RFC is merged too, marked rejected, so the reasoning is not
   lost and the question is not re-opened without new evidence.

## What an RFC must contain

Motivation with the defect or need that prompted it; the design in enough
detail to implement; alternatives and why they were not chosen; numbers
where the change claims performance, accuracy or size; migration and
compatibility consequences under the versioning contract; open questions.
