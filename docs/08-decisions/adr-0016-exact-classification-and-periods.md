# ADR-0016: Exact classification and period arithmetic

Status: accepted (maintainer, 2026-09-04); amends ADR-0011
Date: 2026-09-04
Question: Q26

## Context

ADR-0011 fixes `f64` for every computation and a rounding contract at
serialisation. Two results cannot be made identical across platforms
under that policy alone, and both are classifications rather than
astronomy:

1. **Which division a longitude falls in.** A sign, nakshatra, pada,
   varga part or KP sub-lord computed as `floor(longitude / width)` is
   decided by the last unit in the last place whenever the longitude sits
   at a boundary, and the last unit is exactly where x86-64, aarch64 and
   wasm32 differ (fused multiply-add contraction, extended intermediates).
   The baseline engine reached for decimal arithmetic at these boundaries
   for the same reason; that removes the platform variance but not the
   cost.
2. **Whether dasha children cover their parent.** Five levels of
   `parent × child_years / total_years` in `f64` leave gaps and overlaps
   of nanoseconds at every boundary; an instant that falls in a gap has no
   period. The numerical error is irrelevant (microseconds over a century);
   the structural defect is a result with no answer, once a year, never
   reproducible.

## Decision

Four representations, each chosen for its layer. Astronomy stays as
ADR-0011 says.

| layer | representation | why |
|---|---|---|
| astronomy: series evaluation, positions, cusps, rise and set, root finding | `f64` with ADR-0011's hygiene | the source data and every reference are `f64`; more digits would be invented |
| angles as data: every longitude, latitude and cusp once computed | `i64` nanoarcseconds (`Nas`; 1 circle = 1 296 000 000 000 000) | `f64` resolution at 360 degrees is about 0.3 nanoarcseconds, so nothing is lost and nothing is fabricated; `i64` holds about 7 000 circles |
| classification: sign, nakshatra, pada, varga part, KP sub, koota lookups | exact integer arithmetic: `index = (nas × divisions) / CIRCLE` in `i128`, no floating-point division anywhere on the path | deterministic by construction; works for any divisor, including KP's 249, which does not divide the circle constant |
| period arithmetic: dasha spans and boundaries | exact rationals over `i128` (`Ratio`), stored as fractions of the parent span, materialised to an instant once at presentation with documented rounding | `sum(children) == parent` becomes an identity, so gaps are unrepresentable |

Rules that make the layers meet:

- `core::angle` owns `Nas`, `Ratio` and the classification primitives and
  is the only place that converts `f64` to `Nas` (round half to even).
  Every classification the SDK reports is computed from the `Nas` value
  it serialises, so a consumer who classifies from the serialised integer
  gets the SDK's answer.
- Boundaries are half-open and lower-inclusive: `[start, end)`. The rule
  is documented, pinned by conformance fixtures and part of the
  calculation version (ADR-0020).
- Serialised angles carry the exact integer (`lon_nas`) and the decimal
  degrees derived from it; the 1e-9-degree figure in ADR-0011 is a
  derived presentation, no longer the canonical value.
- A lint forbids `f64` division in any classification path; a `trybuild`
  test proves the float path is not expressible where the type system
  can make it so.

## Consequences

- Property tests assert an exact partition of the circle for every
  divisor from 1 to 360 and every varga definition, and
  `sum(children) == parent` at every depth for every dasha system.
- `dasha_at(instant, depth)` compares by `i128` cross-multiplication and
  never materialises a timestamp; the design is in
  `03-design/dasha-kernels.md`.
- Pack validation rejects a dasha definition whose denominators could
  overflow `i128` at the supported depth (five levels of a 120-year cycle
  need about 2.5e10, far inside the bound).
- The ADR-0011 rounding contract now reads: angles serialised as `Nas`
  plus derived degrees, instants to the millisecond, scores to stated
  decimals.

## Alternatives considered

Decimal arithmetic throughout (the baseline engine's tool: solves
precision, which is not the problem, and only removes platform variance
if every operation is decimal, which series evaluation never is);
`f64` with an epsilon at boundaries (an epsilon moves the boundary, it
does not fix it); microarcseconds (discards real `f64` precision);
double-double in the general path (nothing measured needs it; ADR-0011
keeps it as a per-kernel exception).

## Evidence

`01-research/platform/13-astronomy-layer.md` (the `f64` resolution
argument); the baseline engine's decimal-at-boundaries practice; the
dasha design page's invariant list; Teimeris's measured cost of disabling
contraction, which is what makes integer classification the cheap path.
