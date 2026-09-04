# ADR-0009: The SDK owns the astronomy layer above raw positions

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q8, Q15

## Context

Providers differ in what they compute and how; if the SDK left houses,
sidereal longitudes, sunrise and crossings to providers, a chart's meaning
would change with the provider. The maintainer decided the SDK must be
complete from v1.

## Decision

The `astro` crate implements, in the SDK: timescales and Delta T models,
sidereal time, precession and nutation models with matching obliquity,
frame bias, frame completion (light-time, aberration, deflection,
nutation) from any declared provider frame to the canonical apparent
geocentric ecliptic of date, topocentric correction, coordinate transforms
and refraction, the full ayanamsha catalogue (47 plus custom) including
star-anchored ones over an SDK star table, every house system Swiss and
Teimeris offer plus the Vedic Bhava-Chalit variants with stated polar
behaviour, the rise/set/transit solver, crossings and stations search, the
equation of time, and (v1.x) eclipses and the full star catalogue. Every
row is gated against Teimeris with a published agreement. Providers may
override any of these natively; the profile's override policy decides
(Q23), and overrides are gated to agree within the published bound.

## Consequences

- Phase 2 of the roadmap is this layer; everything above depends on it.
- The SDK implements house systems and ayanamshas from published
  definitions, never from Swiss's AGPL source.
- Teimeris is the oracle; Teimeris is updated as needed to expose what
  the tests and the vtable adapter need (Q15).
- Cross-provider byte-identical results are possible with `sdk-only`.

## Alternatives considered

Provider-computed houses and events with SDK fallbacks (the first draft).

## Evidence

`01-research/platform/13-astronomy-layer.md`; Teimeris's `DEFERRED.md`
model inventory (11 precession, 5 nutation, 5 Delta T, 4 sidereal-time, 3
frame-bias models, verified against upstream).
