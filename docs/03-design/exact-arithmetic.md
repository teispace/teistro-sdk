# Exact arithmetic: angles as data, classification, periods

Status: `draft`, 2026-09-04. Design for ADR-0016. Lives in `core::angle`
and `core::ratio`; Phase 1.

## Purpose

Two families of result must be identical on every platform and must
never disagree with what the SDK serialised: which division of the circle
a longitude falls in, and where a dasha period begins and ends. Both are
integer problems once the astronomy has produced an `f64`. This page
defines the types, the single conversion point and the invariants.

## Types

```rust
/// A canonical angle: nanoarcseconds, 0 ..< CIRCLE. Exact, ordered, hashable.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nas(i64);

impl Nas {
    pub const CIRCLE: i64 = 1_296_000_000_000_000;       // 360 × 3600 × 1e9
    pub const PER_DEGREE: i64 = 3_600_000_000_000;

    /// The only conversion from floating point in the workspace: normalise
    /// to [0, 360), scale, round half to even. Lives here and nowhere else.
    pub fn from_degrees(deg: f64) -> Nas;
    pub fn to_degrees(self) -> f64;                       // presentation only

    /// Index of the division a longitude falls in, for any divisor. Exact:
    /// cross-multiplication in i128, no floating-point division, no
    /// divisibility requirement (249 does not divide CIRCLE).
    pub const fn division_index(self, divisions: u32) -> u32 {
        ((self.0 as i128 * divisions as i128) / Self::CIRCLE as i128) as u32
    }
    pub const fn sign(self) -> u8        { self.division_index(12) as u8 }
    pub const fn nakshatra(self) -> u8   { self.division_index(27) as u8 }
    pub const fn pada(self) -> u8        { self.division_index(108) as u8 }
    pub const fn in_sign(self) -> Nas    { Nas(self.0 % (Self::CIRCLE / 12)) }
    /// Part index within a sign for a varga of N equal parts.
    pub const fn part(self, n: u32) -> u32 {
        ((self.in_sign().0 as i128 * n as i128) / (Self::CIRCLE / 12) as i128) as u32
    }
}

/// An exact non-negative rational over i128, always in lowest terms.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ratio { num: i128, den: i128 }
```

Why nanoarcseconds: at 360 degrees the spacing of `f64` values is about
0.3 nanoarcseconds, so the integer keeps every bit the astronomy
produced and invents none; `i64` holds about 7 000 circles. Why
`division_index` by cross-multiplication: the circle constant is
2^17 × 3^4 × 5^12, so widths such as 360/249 or 30/7 are not
representable, and a division by them in floating point is decided by
the last bit, which is the bit that differs between x86-64, aarch64 and
wasm32.

## The one conversion point

Every longitude, latitude, cusp and derived point becomes a `Nas` once,
when the astronomy layer hands it to the chart foundation. From there,
every classification reads the `Nas`. The foundation serialises the
`Nas` as `lon_nas` beside `lon_deg` (derived from it), so anyone who
recomputes a sign from the serialised value gets the SDK's answer.
`to_degrees` exists for presentation and for feeding a value back into an
`f64` computation (a house cusp into a house-span calculation), never for
classification.

## Boundaries

Half-open, lower-inclusive: a longitude of exactly 30 degrees is in
Taurus, exactly 13°20′ is in Bharani. Documented here, pinned by
conformance fixtures at exact boundaries, and part of the calculation
version (ADR-0020). Unequal spans (D30, nakshatra spans with Abhijit)
use a cumulative span table in `Nas`, compared by integer comparison.

## Period arithmetic

A dasha period is a fraction of its parent's span:

```rust
pub struct Period { pub lord: Lord, pub start: Ratio, pub end: Ratio }  // fractions of the parent span
```

Children are produced from the parent's fraction list and the system's
year table, so `end` of the last child equals `1` exactly and
`sum(children) == parent` is an identity. `dasha_at(instant, depth)`
converts the instant to a fraction of the root span once and compares by
`i128` cross-multiplication at each level. Conversion to a Julian day
happens once, at presentation, with the rounding stated in the results
schema (to the millisecond). Denominators: five levels of a 120-year
cycle multiply to about 2.5e10; the pack validator refuses a definition
whose worst-case denominator at the supported depth would overflow
`i128` with a nanosecond-scale span, which none of the catalogued systems
approaches. If a profile shows allocation from `Ratio` reduction, the fix
is a packed small-denominator fast path, not floating point.

## What stays `f64`

Series evaluation, cusps, sunrise, root finding, speeds, declinations,
every quantity the astronomy layer computes, under ADR-0011's hygiene.
`Nas` is the boundary between computing a number and asking which bin it
is in.

## Enforcement

- A lint (`xtask check-lints`) refuses `f64` division and `floor` in any
  path under `core::angle::classify` or in any module marked
  `classification`; the type system keeps `Nas` free of `From<f64>`.
- Property tests: for every divisor 1 to 360 and every varga definition
  the indices partition `[0, CIRCLE)` with no gap or overlap and are
  monotonic; a value at a boundary lands on the lower-inclusive side; for
  every dasha system and depth 1 to 6, children sum to their parent
  exactly and every instant in a period is found by `dasha_at`.
- Determinism: the classification fixtures run in the cross-architecture
  hash matrix (ADR-0022).

## Open questions

None open; the rounding rule and the boundary rule are decided. The
small-denominator fast path is a measurement in Phase 5 (a six-level
Vimshottari tree benchmark).
