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

## Amendment (2026-11-14, SR-100)

The port asks the ayanamsha **on a basis**: `ayanamsha_deg(jd, scale,
ayanamsha, basis)`, and the C vtable's `ayanamsha` callback takes
`basis` (`TS_AYANAMSHA_BASIS_MEAN` or `TS_AYANAMSHA_BASIS_TRUE`; vtable
version 5, C ABI 2). An engine answers each basis from its own nutation,
or says `UNSUPPORTED`; nothing is left to an engine's default and the SDK
never adds its own nutation to a native value. Under `PREFER_NATIVE` a
declined basis is the catalogue's, stamped `ayanamsha:SDK`; under
`NATIVE_ONLY` it refuses the step. The previous amendment's last sentence
no longer holds: a modern provider now answers the true basis, which
every drik profile uses (`03-design/settings-and-profiles.md`). A chart's
zodiac over a modern engine is still the SDK's catalogue, on the frame's
basis.

**The positions path.** A sidereal `Frame` names its ayanamsha and no
basis, so a modern engine asked for one whole answered on its own: the
Teimeris adapter sets its sidereal mode and flag and answers the mean
basis whatever `frame.ayanamsha_basis` said, and the per-step choice was
skipped (the boundary audit of 2026-11-14, A1, B2 and A6). The
completion no longer asks a modern engine for a sidereal frame whole: it
asks the frame's tropical twin, natively and whole where the engine can,
and shifts it by the ayanamsha step, which asks the basis and keeps the
policy. A classical sky, which has no nutation, still answers its own
sidereal frame. Carrying the basis in the frame itself (A6's frame bits)
would let an engine answer it whole again; until then the shift is the
one route.

## Consequences

- Two policies to test in the conformance harness for every override.
- Three tier builds in the size gates and the accuracy document.
- The blackout calendar in v1.0 does not depend on eclipse instants.

## Alternatives considered

`sdk-only` as the default (cross-provider identity over tool agreement);
one tier only; eclipses in v1.0.

## Evidence

`01-research/platform/13-astronomy-layer.md`, `14-builtin-ephemeris.md`.
