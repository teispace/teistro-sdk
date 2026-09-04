# ADR-0018: Evidence ranks and "mark and continue"

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q28

## Context

The catalogue is far larger than what any one implementation verifies.
Filling tables from the nearest available source produces rows that look
sourced and are not, and a plausible number that is silently wrong is the
worst defect an astrology engine can ship. Two examples found while
writing the design pages: a received text for a sixty-year dasha whose
verses sum to sixty-nine years, and a third-party implementation carrying
antardasha permutation tables for Narayana dasha that no rule in the
baseline engine generates.

## Decision

**Evidence has rank.** Rank 1: primary texts and observational references.
Rank 2: the baseline engine and Teimeris, ours and validated. Rank 3:
third-party implementations and their documentation. A disagreement
between rank 3 and rank 2 is a question, entered in
`01-research/feature-universe/19-verification-cruxes.md`; only rank 1
corrects rank 2. `CLEAN_ROOM.md` lists the sources by rank.

**Every table row carries a confidence mark.**

| mark | meaning | ships |
|---|---|---|
| V | verified: parameters read from the baseline engine's source or a primary text, cited to file or verse | yes |
| T | traditional: shape and values well attested and self-consistent, no primary text consulted yet | no, until a citation lands |
| S | shape only: the kernel fields are identified, the values are not | no |

**Mark and continue.** An unsourced or unverified variant is a registered
identifier with no implementation. Requesting it returns `UNSUPPORTED`
with the reason `unsourced`; it never falls back to another variant. A
row that ships with a known but unverified source carries
`confidence: unverified`, surfaced in the result envelope. Absent is
safe; silently defaulted is not.

Where a classical convention has to be chosen for the arithmetic to
terminate (a seed outside a conditional dasha's cycle, an unattested
divisional chart), the choice is a named field in the row, and the result
carries a flag saying the choice was applied.

## Consequences

- Pack and table validation refuse a row without a `sources` field;
  the generated reference marks V, T and S per row.
- The cruxes page is a living register with an owner per item; a crux is
  closed by a citation, never by a vote.
- Principles 4 and 11 in `00-vision/02-principles.md` gain this rule.

## Alternatives considered

Shipping T rows as defaults with a warning (the warning is ignored and
the number is cached); taking a third-party table as authority because it
is the only one available (it may be a school, a misreading, or right,
and nothing distinguishes the three until a text is read).

## Evidence

The two cruxes above; the baseline engine's practice of compile-time
citations on every rule, which this extends to tables.
