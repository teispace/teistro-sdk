# ADR-0002: The SDK is ephemeris-agnostic through a capability-negotiated port

Status: accepted (maintainer, 2026-09-04), in the revised form below
(Q7 and Q8 changed the required surface and the fallback)
Date: 2026-09-04
Question: Q7, Q8, Q15

## Context

Swiss Ephemeris is AGPL or commercially licensed; Teimeris is private-use
under a Professional Licence; consumers may have other ephemerides. The baseline engine
couples its engine to `sweph` through global state and twelve setter call
sites. The maintainer decided that the SDK must be complete from v1 (the
"master key"), including working with no external ephemeris at all.

## Decision

The SDK defines an ephemeris port whose only required operation is
`positions` (a grid of instants and bodies, returned in a declared frame
with speeds); everything above raw positions is computed by the SDK's own
`astro` layer (ADR-0009). A provider may declare native overrides (nodes
and apsides, Delta T, nutation, sidereal time, ayanamsha, houses, rise and
set, crossings, stations, eclipses, stars) which the SDK uses when the
profile's override policy allows, holding them to a published agreement
with its own implementation. The SDK links no external ephemeris; it ships
its own built-in analytic provider as a removable module (ADR-0008), plus
adapters for Teimeris and Swiss as separate packages. Frames travel on the
request. Every result stamps the provider identity, tier, frame and which
implementation computed each part.

## Consequences

- The SDK works with nothing else installed, and its licence is
  unencumbered by ephemeris licensing.
- Provider adapters are small (positions plus optional overrides).
- The SDK carries the astronomy layer's implementation and conformance
  burden (Phase 2), which is what makes "same behaviour everywhere" true.
- A conformance kit ships for adapters and for the built-in provider.
- Teimeris exports a C vtable so every binding gets the zero-cost native
  path (Q15).

## Alternatives considered

Requiring providers to compute houses, events and ayanamshas (the first
draft): rejected because a chart would change meaning with the provider.
Bundling Teimeris or Swiss: rejected on licence grounds. No built-in
provider: rejected by the maintainer (Q7).

## Evidence

`01-research/platform/03-ephemeris-abstraction.md`, `13-astronomy-layer.md`,
`14-builtin-ephemeris.md`; the baseline engine's `AstronomicalBackend`; Teimeris's
headers and migration measurements.
