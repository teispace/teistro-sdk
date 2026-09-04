# ADR-0008: A built-in analytic ephemeris ships in v1

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q7

## Context

The SDK must compute a correct chart with nothing but the SDK installed,
on every platform including wasm and mobile where ephemeris data files are
a size problem, and without inheriting any ephemeris licence.

## Decision

The SDK ships `ephemeris-builtin`, a module implementing the ephemeris
port from published analytic theories: VSOP87 for the Sun and planets,
ELP/MPP02 for the Moon, Pluto from Chebyshev coefficients fitted by the
SDK's own tool from a public-domain JPL kernel, mean and true nodes and
apogees, analytic speeds. It ships in three tiers (`compact`, `standard`,
`full`) as features and packages, with the tier stamped in every position.
It is developed in its own roadmap phase (Phase 3) and is removable by
tree-shaking when a consumer registers another provider. Corrections above
geometric positions are done by the `astro` layer, shared with every
provider.

## Consequences

- Zero-setup use of the SDK; a second, independent oracle for the
  astronomy layer's tests.
- An ingestion tool (`teistro ephemgen`) and generated coefficient tables
  with citations become part of the build.
- Asteroids, Chiron and Uranian points are not covered by the built-in
  provider; Teimeris or Swiss are needed for those.
- The accuracy per body, century and tier is published from the
  conformance run against Teimeris, never claimed.
- Q17 confirms the tiers to ship, the default tier and the data terms.

## Alternatives considered

No fallback in v1 (the first recommendation): rejected by the maintainer.
Compressing JPL data Swiss-style: more accurate but a far larger project;
possible later as a fourth tier.

## Evidence

`01-research/platform/14-builtin-ephemeris.md`; Astronomy Engine's
truncated-VSOP87 design and its measured 1-arcminute bound; Teimeris as
the oracle.
