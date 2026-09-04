# ADR-0011: Precision policy

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q13

## Context

The baseline engine uses decimal.js at some boundaries; astronomical libraries use
`f64`; the maintainer asked for the highest precision and accuracy we can
deliver.

## Decision

All computation in IEEE-754 `f64` with the numerical hygiene that lets
`f64` deliver its full 15 to 16 significant digits: split Julian days
(integer day plus fraction) for sub-millisecond instants, one angle
normalisation routine, compensated summation where a long series is
measured to lose bits, iteration to convergence with caps, no fast-math,
a stated floating-point contraction policy, and a rounding contract applied
only at serialisation (longitudes to 1e-9 degrees, instants to the
millisecond, scores to stated decimals). Bit-identity with Teimeris is
gated where achievable by construction; elsewhere the agreement is
measured and published. Extended precision (double-double) is adopted in a
kernel only when a measurement shows `f64` fails a target.

## Consequences

- No decimal library; deterministic results across platforms to stated
  tolerances, with a cross-platform golden test.
- Tolerances are part of the results schema and the accuracy document.

## Alternatives considered

Decimal arithmetic (slow, no astronomical library uses it, and it does
not improve accuracy of series evaluation); `f32` anywhere (never).

## Evidence

`01-research/platform/13-astronomy-layer.md`; Teimeris's floating-point
policy and its measured cost of disabling contraction.
