# ADR-0021: The reference-accuracy ephemeris path

Status: accepted (maintainer, 2026-09-04); extends ADR-0008, ADR-0009 and ADR-0013
Date: 2026-09-04
Question: Q31

## Context

ADR-0008 ships an analytic built-in ephemeris in three tiers whose best
accuracy is the theories' own (about an arcsecond). ADR-0009 makes the
SDK own the astronomy above raw positions, which means implementing frame
bias, precession, nutation, sidereal time, aberration and light
deflection ourselves. Three facts make a reference-accuracy path
permissively available: ERFA, the IAU's fundamental-astronomy routines,
is BSD-3-Clause; JPL DE data is public domain and its DAF/SPK file format
is specified to the byte; and a Chebyshev refit of a DE file over a
bounded range at a loosened tolerance is small.

## Decision

1. **The IAU routines in `astro` are a faithful Rust port of ERFA**:
   structure and operation order preserved so the C and the Rust read
   side by side; coefficient tables as constants; no allocation; every
   ported function tested against the C library over a wide input sweep
   at 1e-15 relative or bit equality; a provenance table (ERFA function,
   our function, test, version ported from); the BSD-3 notice in `NOTICE`.
   The C library is a dev-only oracle, never a dependency (ADR-0019).
   This lands in Phase 2, where those routines are built anyway.
2. **`ephemeris-de`**, a provider that reads JPL DE files directly:
   DAF/SPK types 2 and 3, endianness from the file's own field, the
   FTP-corruption check, bounds-checked reads with no `unsafe`, cycle
   detection on the summary list, a segment index built once, Clenshaw
   evaluation with a fixed operation order, centre chaining from segment
   metadata (Earth through the Earth-Moon barycentre) never hard-coded,
   and record access designed so a browser can fetch the few records a
   chart needs by HTTP range request. Target 0.001 arcsecond against
   JPL Horizons. Scheduled for v1.x; fuzzed as a parser of untrusted
   files.
3. **A fourth built-in tier, `reference`**: a Chebyshev refit from DE440
   over 1600 to 2400 produced by `ephemgen`, per-body blobs, about 1 MB
   for all bodies, target 0.005 arcsecond for the Sun, Moon and inner
   planets and 0.02 arcsecond for the outer planets, verified against the
   source file at ten times the sample density before it ships. Phase 3
   if the fitter meets the target, otherwise v1.x. If it misses, the
   degradation ladder is decided now: split per body so a panchanga
   consumer ships the Sun and Moon only; narrow the range to 1700 to
   2300; loosen the outer planets to 0.05 arcsecond; raise the budget to
   1.5 MB and say so. Never a looser figure under the published one.
4. **Validation is stage-isolated.** Teimeris exposes the same flags
   Swiss Ephemeris does (true position, no aberration, no deflection,
   J2000, no nutation, barycentric, equatorial, topocentric, sidereal), so
   the reduction chain is climbed one rung at a time with a reference at
   every rung; a discrepancy at a rung names its stage. JPL Horizons and
   CSPICE against DE440 are the final authority; agreement with Swiss
   Ephemeris proves consistency, not correctness. The sweep is every six
   hours across 1600 to 2400 for every body, with adversarial fixtures at
   segment boundaries, perigee and apogee, conjunctions, stations and both
   file endiannesses.

## Consequences

- `ephemgen` gains a DE refit mode; the size gates gain a fourth tier;
  the accuracy document gains rows per tier and for `ephemeris-de`.
- `NOTICE` gains ERFA; the provenance table is generated into the docs.
- The module catalogue gains `ephemeris-de` (v1.x) and marks `astro`'s
  IAU routines as ERFA-derived.
- The built-in ephemeris can be the test oracle for `astro` at reference
  accuracy, not only at arcsecond accuracy.

## Alternatives considered

The MPL-licensed Rust ERFA port and ANISE as dependencies (denied by
ADR-0019); only analytic tiers (an accuracy ceiling of about an
arcsecond, which is enough for parity through Teimeris but not for the
SDK's own claim); Swiss-style compression (thirty years of specialist
work; the `reference` tier accepts about five times Swiss's error at a
similar size and publishes the figure).

## Evidence

`01-research/platform/14-builtin-ephemeris.md`; the NAIF DAF and SPK
specifications; ERFA's licence; the Teimeris flag set in
`02-architecture/02-ephemeris-port.md`.
