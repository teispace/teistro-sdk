# ADR-0017: Kernel and table

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q27

## Context

The catalogue in `01-research/` lists more than fifty dasha systems,
arbitrary divisional charts with named variant schemes, twenty house
systems, eighteen strength components under several conventions, and
hundreds of rules. Written as one implementation per variant this is
unmaintainable by a small team, and the three reference codebases show it:
the baseline engine has eighteen dasha engine classes, PyJHora one module
per system, VedAstro over a thousand lines for Vimshottari alone. The
baseline engine's own code shows the alternative from the inside: its
Padanadhamsa engine differs from its Chara engine in one expression (the
start sign), Niryana Shoola from Shoola in one expression, and its
proportional nakshatra dasha builder is one function parameterised by
sequence, total, start nakshatra and count direction that already serves
six systems.

## Decision

For every family of variants the SDK implements **one kernel** and
expresses the variants as **rows of data** over it: udu (nakshatra-seeded)
dashas, rashi (sign-progression) dashas, Kalachakra, the cycle-scale
decorator, divisional charts, house cusps and bhava spans, bala schemes,
rules, panchanga limbs, boundary solving, event scanning, aspect models,
compatibility scoring. A system that needs code keyed to its own
identifier is a defect in the kernel, not a special case.

Discipline that goes with it:

1. **Falsify before building.** Before a kernel is coded, every variant
   the catalogue names is written as a row and the schema is corrected
   wherever a variant refuses to fit. The passes for dashas, vargas and
   balas are in `03-design/` (`dasha-kernels.md`, `varga-kernel.md`,
   `strength-schemes.md`); the rules kernel is `rules-engine.md`.
2. **Invariants over the whole table**, in one test, not per row:
   totals derived and asserted exactly, sequences that visit every sign
   once, maps that yield a valid lord for every seed, tables that cover
   the circle.
3. **Confidence marks on rows** (ADR-0018): a row ships only when it is
   verified against the baseline engine or a primary text.
4. **The tree layer over the dasha kernels is lazy**: a cursor exposes
   roots and children; `dasha_at(instant, depth)` walks one branch in
   O(children × depth) with no allocation after warm-up; ranges and
   searches prune; materialisation takes an explicit depth and window.
   The baseline engine defaults to three levels because a materialised
   five-level tree (nine to the fifth nodes per system) was slow; with a
   cursor, depth is free.
5. **Kill criterion.** If a kernel accumulates a third "compute this
   parameter from the chart" escape hatch, it has become an interpreter
   and is redesigned as one rather than defended as a table.

## Consequences

- Adding a dasha system, a varga scheme or a bala convention is adding a
  row with a citation; the plug-in interface for these registries is the
  row schema first and a Rust trait only for systems the schema cannot
  express (Patyayini, whose periods come from planetary strengths, is the
  known case).
- A defect fixed in a kernel is fixed for every row at once; the test
  surface is the kernel's parameter space plus per-row data checks.
- Binary size stays flat as the catalogue grows; rows are data.
- The generated reference documentation is rendered from the tables.

## Alternatives considered

A class or module per system (the pattern of all three reference
codebases; a defect in a shared step is fixed N times or N-1 times);
macros that generate per-system code (unreadable, invisible to
documentation tools, and still N implementations).

## Evidence

The baseline engine's dasha engines and its proportional nakshatra builder
(`dasha-base.engine.ts`, `countToReference`); PyJHora's
`custom_divisional_chart`, which independently arrived at the varga
parameterisation in `03-design/varga-kernel.md`; the three-codebase
comparison recorded in `01-research/competitive-analysis/`.
