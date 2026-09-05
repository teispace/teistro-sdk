# Astronomy: time scales and frames

Status: `draft`, written 2026-09-05 when the precession models were
built for the ayanamsha catalogue; the completion steps still to come
(equinox, centre, corrections) are designed here and say so. Derives
from `01-research/platform/13-astronomy-layer.md` (the time scale,
precession, nutation, frame and completion rows with their conformance
targets), `02-architecture/02-ephemeris-port.md` and
`03-design/ephemeris-port-and-adapters.md` (the frame and the completion
built so far), `03-design/time-and-timezone.md` (Delta T and the scales)
and ADR-0021 (the ERFA ports). Teimeris is the rank-2 oracle for every
model; the IAU resolutions and the cited papers are rank 1.

## 1. Purpose and scope

Every position the SDK hands out is in a frame: a centre, an equinox, a
coordinate system, a zodiac and a set of corrections. A provider returns
one frame natively and the completion turns it into the one a caller
asks for, step by step, each step stamped with who did it. This page
settles the models behind those steps: the time scales the astronomy
runs in, precession as a catalogue of models, nutation, the obliquity,
the frame bias, and the steps that remain (equinox, centre, corrections)
so that a J2000 geometric provider (a JPL kernel, the built-in ephemeris)
completes to the canonical apparent frame of date.

## 2. Inputs, settings and ports

Instants in UT1 or TT (`time-and-timezone.md`: UTC goes through the
leap-second table, UT1 to TT through the Delta T model); the ephemeris
port's `Frame`; the `frame.*` knobs (zodiac, ayanamsha, basis, centre,
positions apparent or true); the override policy (ADR-0013). No new
knob for the precession or nutation model yet: the SDK's defaults are
the ones below, and a model knob arrives when a consumer needs another
(§10).

## 3. The data model

```rust
pub enum PrecessionModel { Vondrak2011 /* default */, Iau2006, Iau1976, Newcomb }
pub type Matrix3 = [[f64; 3]; 3];   // row-major rotation, ERFA's sense: p_date = R · p_J2000
pub type Vector3 = [f64; 3];
pub struct Obliquity { mean_deg, true_deg, nutation_lon_deg, nutation_obl_deg }   // the port's record
```

Time scales as built: `JulianDay<Utc>`, `JulianDay<Ut1>`, `JulianDay<Tt>`
branded per scale so a UT1 value cannot be passed as TT; two-part dates
(`split()`) into every ported routine so an instant keeps its
sub-millisecond resolution. Frames as built: the port's `Frame` with
its bits (centre, equinox, coordinates, zodiac with the catalogue id,
corrections).

## 4. Algorithms

**Precession** (`astro::precession`). Four models over the ported
routines, each giving the matrix from the mean equator and equinox of
J2000.0 to the mean of a date, without the frame bias, and the mean
obliquity it is consistent with:

| model | matrix | obliquity | validity | use |
|---|---|---|---|---|
| Vondrák, Capitaine and Wallace (2011), the default | `ltp` from the ecliptic and equator poles (`ltpecl`, `ltpequ`) | the paper's own series (`ltpeps`) | ±200 000 years | the ayanamshas, whose epochs reach the first millennium BCE; the engines' default |
| IAU 2006 (P03) | `bp06`'s precession matrix | `obl06` | a few millennia | the modern short-term model; the obliquity record |
| IAU 1976 (Lieske) | the 323 Euler rotation of ζ, z, θ | `obl80` | ±2 centuries | the fitted-constant correction (Lahiri) |
| Newcomb (Kinoshita 1975) | the same rotation, tropical millennia from B1850 | Newcomb's, referred to 1850 | historical | the fitted-constant correction (Fagan and Bradley, Raman, Krishnamurti) |

Vondrák and IAU 2006 agree to 4.3 mas over a century from J2000.0 and
their obliquities within 0.002° two millennia away; the other seven
models Swiss offers (Laskar 1986, Williams 1994, Simon 1994, Bretagnon
2003, Owen 1990, IAU 2000, Williams with Laskar's obliquity) are
registered in the research and arrive with a model knob (§10).

**Nutation.** IAU 2000B (77 terms, `nut00b`) as built: 1 mas against
IAU 2000A in the modern era. IAU 2000A (1365 terms, `nut00a`) and its
P03 adjustment (`nut06a`) are the next ports, for the star table and
for a provider that asks for them.

**The obliquity record** (`sky::obliquity`). The IAU 2006 mean
obliquity and the IAU 2000B nutation, the true obliquity their sum: the
record the completion rotates with under `sdk-only`, and what the kit
holds a provider's override to within 0.01″.

**The frame bias.** The ICRS to J2000.0 mean rotation (`bi00`, `bp06`'s
bias matrix, `ltpb`), needed when a provider returns ICRS or GCRS
positions (a JPL kernel); the precession models above exclude it so
that J2000 mean positions precess without it.

**The completion steps** (`astro::completion`, `ephemeris-port-and-adapters.md`
§5). Built: coordinates (the rotation between the ecliptic and the
equator through the obliquity record or the provider's), the zodiac
(the sidereal shift through the provider's ayanamsha or the SDK's
catalogue, applied while the columns are ecliptic). Designed, for a
J2000 geometric provider:

1. *centre*: barycentric or heliocentric to geocentric by subtracting
   the Earth's barycentric position (ERFA's `epv00`, or the provider's
   Earth), with the light-time iteration (the position at `t − d/c`,
   two or three passes to convergence);
2. *corrections*: annual aberration from the Earth's barycentric
   velocity (`ab`), gravitational deflection by the Sun (`ld`, `ldsun`),
   so that `GEOMETRIC` becomes `APPARENT`; the inverse selections
   (`TRUE`, no aberration) as the `frame.positions` knob;
3. *equinox*: J2000 to of date by the precession matrix of the model in
   force and the nutation matrix (IAU 2000B or 2000A), the frame bias
   first when the native frame is ICRS;
4. *topocentric*: the observer's geocentric position (WGS84) and the
   parallax, for `frame.centre = TOPOCENTRIC`.

Each is stamped as the steps built so far are, and the kit's
`completion_native` and `completion_sdk` checks grow a row each.

## 5. The API

Rust: `precession::{PrecessionModel, matrix, to_date, to_j2000, between,
mean_obliquity_rad, mean_obliquity_deg, equatorial_to_ecliptic,
ecliptic_to_equatorial}`; `iau::{p06, ltp, vector}` for the ported
routines; `sky::obliquity`; `Completion::with_precession`. C ABI and
bindings: the frame is already the port's; a precession model knob is
a settings field when it arrives, never a positional argument.

## 6. Errors and degenerate states

| situation | outcome |
|---|---|
| a completion step not built (equinox, centre, corrections) | `UNSUPPORTED`, naming the step, with the hint to ask the provider for a frame it returns natively |
| a native-only policy over a provider without the override | `PolicyRefused`, naming the step and the policy |
| an instant far outside a model's validity | the value is computed and the provenance names the model; the Delta T uncertainty grows with the distance (`time-and-timezone.md`) |

## 7. Performance budget

| operation | budget | measured (`cargo bench -p teistro-astro`, Apple M-series, 2026-09-05) |
|---|---:|---:|
| the obliquity and nutation (IAU 2006, 2000B) | 2 µs | 0.79 µs |
| the precession matrix, Vondrák 2011 | 2 µs | 142 ns |
| the precession matrix, IAU 2006 | 1 µs | 36 ns |

## 8. Tests

- Every ported routine against ERFA's reference values (`iau::p06`,
  `iau::ltp`, `iau::vector`): the tolerances the reference program uses.
- Vondrák's obliquity series against IAU 2006: a microarcsecond at
  J2000.0, 0.05″ within a millennium; the two models' matrices 4.3 mas
  apart over a century.
- Every model the identity at J2000.0, orthogonal, round-tripping, and
  1.396° of general precession over a century.
- The ayanamsha catalogue against Teimeris over 1044 rows
  (`astro-ayanamsha-catalogue.md`, §8), which exercises the Vondrák
  matrices and obliquity at epochs from −700 to 2500.

## 9. Localisation

None.

## 10. Open questions

1. **A precession and nutation model knob.** The defaults are Vondrák
   2011 and IAU 2000B; the research row registers eleven precession and
   five nutation models. A `frame.precession` and `frame.nutation` knob
   arrives with the first consumer who needs another model (JPL Horizons
   compatibility, or a historical reconstruction), each model ported and
   held to Teimeris.
2. **The remaining completion steps** (centre, corrections, equinox,
   topocentric) arrive with the built-in ephemeris (Phase 3), which is
   the first provider that returns a geometric J2000 frame; the kit's
   two completion checks then compare the SDK's chain with an engine's
   apparent output at 1e-6″ when fed identical geometric vectors.
3. **IAU 2000A.** Ported when the star table needs it; until then the
   1 mas of IAU 2000B is inside every chart's tolerance.
