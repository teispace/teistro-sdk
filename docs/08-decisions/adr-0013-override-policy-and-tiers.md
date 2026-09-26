# ADR-0013: Provider override policy default and built-in ephemeris tiers

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q17, Q22, Q23

## Context

ADR-0009 lets a provider declare native implementations of computations
the SDK also owns; ADR-0008 ships the built-in ephemeris in tiers. Both
need defaults that decide charts.

## Decision

- The profile's provider override policy defaults to `prefer-native`: a
  declared native implementation (houses, events, crossings, ayanamsha,
  nodes) is used, so results agree exactly with Swiss-compatible tools
  through Teimeris. `sdk-only` is selectable for byte-identical results
  across providers. Both are gated to agree within the published bound,
  so the choice never moves a chart by more than that bound.
- The built-in ephemeris ships all three tiers (`compact` about 1
  arcminute, `standard` about 1 to 2 arcseconds, `full` the theories'
  accuracy) with `standard` as the default; the tier is stamped in every
  position. Coefficient tables generated from the published VSOP87 and
  ELP/MPP02 series are redistributed with citations in `NOTICE`; Pluto is
  fitted from a public-domain JPL kernel.
- Eclipses and the full fixed-star catalogue are v1.x; v1.0 carries the
  anchor stars and the nakshatra yogataras.

## Amendment (2026-09-26, Q40)

`prefer-native` governs the **completion's steps** — positions, frames,
crossings, events asked for directly — and not a modern chart's
conventions. A chart's day and zodiac are the SDK's over a modern engine,
because measured against the Swiss-based recording they are the closer:
the solver's sunrise 9.77 s at worst against the engine's own 32.39 s,
and the catalogue's nutated ayanamsha within 0.0086″ where the port's
mean value stands 18.46″ off. A **classical** astronomy still defines
both, since its sunrise and zodiac are the text's definitions with no SDK
answer to agree with (`03-design/classical-chart.md`). A modern provider's
native ayanamsha answers the mean basis alone.

## Consequences

- Two policies to test in the conformance harness for every override.
- Three tier builds in the size gates and the accuracy document.
- The blackout calendar in v1.0 does not depend on eclipse instants.

## Alternatives considered

`sdk-only` as the default (cross-provider identity over tool agreement);
one tier only; eclipses in v1.0.

## Evidence

`01-research/platform/13-astronomy-layer.md`, `14-builtin-ephemeris.md`.
