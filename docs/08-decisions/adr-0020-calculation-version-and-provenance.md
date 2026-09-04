# ADR-0020: Calculation version and the provenance envelope

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q30

## Context

Consumers cache charts, store them in databases and compare them across
devices. The API version says nothing about whether a number moved: a
patch that fixes a rounding defect is API-compatible and still invalidates
every cached result computed before it. The envelope in
`02-architecture/05-data-model-identifiers.md` records versions and
hashes, but not the one integer a cache needs, and not several inputs
that decide an answer (which Delta T model, which leap-second table, which
time basis was applied when tzdb had no rule).

## Decision

- A **`calculation_version`** integer, independent of semantic versioning,
  bumps whenever any numeric output changes for identical input,
  including a bug fix. The changelog's **Numbers** section states, per
  bump, which outputs moved and by how much. The `f64` to `Nas` rounding
  rule, the boundary rule and every kernel table are part of it.
- The correct cache key, documented in every binding's guide, is
  `(input_hash, settings_hash, calculation_version)`.
- The envelope gains: `calculation_version`, `input_hash`, provider
  data-file hashes, the Delta T model, the leap-second table version, the
  tzdb version, `time_basis_applied` (for example "LMT, because tzdb has no
  rule before 1880 for this zone"), `deviation` when a classical model
  (Surya Siddhanta) answered, `time_uncertainty` for deep-time instants
  where Delta T is uncertain by hours, the calendar `resolution`
  (`tabular`, `computed`, `divergent`), the convention applied for an
  unattested request (an arbitrary D-N, a seed outside a conditional
  dasha's cycle), and `confidence` for rows shipped as `unverified`.
- Conformance fixtures assert the settings hash, so an engine cannot
  substitute a default and still pass; a fixture change that moves an
  expected value requires a calculation version bump.

## Consequences

- The results schema and the envelope in the data-model page are
  extended; `serial` emits the new fields in canonical JSON.
- The pull request template asks for the calculation version impact.
- The versioning contract in the API conventions gains the row "an
  API-compatible release can still move numbers; consumers cache on the
  calculation version".

## Alternatives considered

Bumping the minor version for numeric changes (conflates API shape with
answers and forces false bumps in both directions); leaving cache keys to
consumers (the exact failure the envelope exists to prevent).

## Evidence

The baseline engine's `calculationVersion` and `profileHash`, which this
generalises and makes public.
