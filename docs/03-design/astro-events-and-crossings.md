# Astronomy: events and crossings

Status: `draft`, written 2026-09-05 when the ephemeris port was promoted
and the rise and set solver built; the crossings and stations sections
are planned for Phase 2 and say so. Derives from
`01-research/platform/13-astronomy-layer.md` (the solver rows and the
conformance targets), `02-architecture/02-ephemeris-port.md` (the rise
and set override), ADR-0013 (the override policy) and ADR-0021 (the
ERFA ports the solver stands on). The baseline engine's sunrise and the
55 fixture charts are the rank-2 reference; Teimeris's own search the
rank-2 oracle for the solver.

## 1. Purpose and scope

Every event search in the SDK is a root of one quantity of time: a
sankranti is the Sun's longitude reaching a sign boundary, a sunrise is a
body's altitude reaching a horizon convention, a station is a speed
reaching zero. This page settles the two solvers built so far, the
boundary solver and the rise and set solver, the horizon conventions,
the sidereal time and the obliquity they need, and the measurements
that hold them; crossings and stations follow in Phase 2 over the same
kernel.

## 2. Inputs, settings and ports

A rise or set needs a source of apparent geocentric equatorial positions
of date (`ApparentPositions`: the frame completion over a provider, or a
classical model), a place, a horizon convention and a Delta T model for
the sidereal time. The settings knobs read are `day.sunrise` (the
convention: `CENTRE_NO_REFRACTION`, `UPPER_LIMB_REFRACTION`,
`LOWER_LIMB_REFRACTION` or a custom altitude of the centre without
refraction), `time.delta_t`, and `provider.overrides` (a provider's own
rise and set search is used under `PREFER_NATIVE` when it declares the
`RISE_SET` override). Ports: the ephemeris port through the completion.

## 3. The data model

```rust
// the port (crates/port-ephemeris)
pub enum HorizonEventKind { Rise, Set, Transit, Antitransit }
pub enum DiscPoint { Centre, UpperLimb, LowerLimb }
pub enum Refraction { None, Standard }
pub struct Horizon { disc: DiscPoint, refraction: Refraction, altitude_deg: f64 }   // from_convention(SunriseConvention), key()
pub struct HorizonRequest { body, kind, place: Place, from: JulianDay<Ut1>, window_days, horizon }
trait EphemerisProvider { fn horizon_event(&self, &HorizonRequest) -> Result<Option<JulianDay<Ut1>>, ProviderError>; ... }

// the astronomy layer (crates/astro)
pub struct Apparent { ra_deg, dec_deg, distance_au }
pub trait ApparentPositions { fn apparent(&self, Body, JulianDay<Ut1>) -> Result<Apparent, Error>; fn describe(&self) -> String; }
pub struct Solver<'a> { sky: &'a dyn ApparentPositions, body, place, horizon, delta_t: DeltaTModel }
pub struct HorizonEvent { instant: JulianDay<Ut1>, method: Method /* Iterated | Scanned */, evaluations: u32 }
pub struct DayEvents { rise: Option<HorizonEvent>, set: Option<HorizonEvent>, above_at_midday: bool }   // arc()
pub struct Disc { semidiameter_deg, parallax_deg }                                                     // Disc::of(body, distance_au)
pub fn centre_altitude_deg(&Horizon, &Disc) -> f64
pub fn solve::next_crossing(angle, target, from, rate, tolerance, Caps) -> Result<Crossing, SolveError>
pub fn solve::first_zero(f, from, to, step, upward, tolerance, Caps) -> Result<Option<Crossing>, SolveError>
```

The horizon convention is the port's vocabulary because an adapter maps
it onto its engine's options (Teimeris: the disc centre, no refraction,
the lower limb and the three twilights; any other altitude is refused
by name). `Horizon::from_convention` reads the settings knob and
`Horizon::convention` writes it back for the stamp; the local day
(`time-and-timezone.md`, §3.3) carries the convention its arc was
reckoned by.

The event altitude is the disc point's target altitude, less the
refraction at the horizon, less the semidiameter for the upper limb
(plus it for the lower), plus the horizontal parallax, since the
observer sees the body lower than the Earth's centre does. The standard
refraction is the almanac's 34 arcminutes at sea level (*Astronomical
Almanac*; Meeus, chapter 15); the semidiameter comes from the body's
radius (the Sun's IAU 2015 nominal radius, the others from Archinal et
al. 2018) and the distance; the parallax from the WGS 84 equatorial
radius and the distance. The observer's height is not applied: the
almanacs and the panchanga reckon from sea level, the baseline did the
same, and a dip is a custom altitude when a caller wants one.

## 4. Algorithms

**Sidereal time and the obliquity** (`sky`). Greenwich apparent sidereal
time is the IAU 2000 mean sidereal time plus the equation of the
equinoxes with the IAU 2000B nutation, from the ported `gmst00` and
`ee00b` with the UT1 and TT instants distinguished (ERFA's `gst00b`
takes one date for both); local apparent sidereal time adds the
longitude. The obliquity record is the IAU 2006 mean obliquity and the
IAU 2000B nutation. Every ported routine is in `astro::iau` with the
provenance table ADR-0021 requires and is tested against the reference
values of ERFA's own test program.

**The rise and set solver** (`rise_set`). Meeus's method (chapter 15)
iterated to convergence: from one reading of the sky at the start of the
window (right ascension, declination, distance, hour angle) the hour
angle at which the centre stands at the event altitude gives a first
instant, and each pass corrects it by the altitude error over the
altitude's rate, `(h − h₀) / (360.985647 cos δ cos φ sin H)` days,
until the correction is under `TOLERANCE_DAYS` (1e-7, under a hundredth
of a second). A transit iterates the hour angle to zero, an antitransit
to 180°. An event that settles just before the window is retried a
sidereal day on; one that settles outside it is reported absent. Where
the rate factor `cos δ cos φ sin H` is under 1e-3 (a grazing event near
the polar circles) or the iteration has not settled in twelve passes,
the solver scans the window in ten-minute steps for the sign change of
`h − h₀` (a rise upward, a set downward; a transit the hour angle's) and
bisects it through the shared solver's `first_zero`, and says so
(`Method::Scanned`). Every loop has a cap; an unmet cap is
`NOT_CONVERGED` naming the event, the body, the place and the instant.

**The day** (`Solver::day`). The rise inside the day that begins at a
local mean midnight, then the set that follows that rise (the day's arc,
which past the polar circles may end after the next civil midnight, as
at Fairbanks on the solstice), or the set inside the day when there is
no rise; and whether the body stood above the event altitude at the
day's middle, which names a day without an arc as a polar day or a polar
night. The drik solar model of the calendar crate (`DrikSun`) maps this
onto `DayLight::{Arc, AlwaysUp, NeverUp}`, so the Bikram Sambat engine,
the local day and ghati-pala run over modern astronomy exactly as over
the Surya Siddhanta.

**The boundary solver** (`solve`). `next_crossing`: a jump toward the
target by the quantity's mean rate, a bracket in which the signed gap
changes sign, a bisection to the tolerance (37 steps from a day to a
microsecond), every loop capped. `first_zero`: the first sign change of
a quantity inside a window, stepped then bisected, absence reported
rather than guessed. Both share one bisection.

**Crossings and stations** (Phase 2). A bracketing scan with a step
bounded by the fastest body's speed and the feature size, then a
refinement to a stated tolerance; single body, composite angle
`a·lon(A) + b·lon(B)` (tithi, yoga, karana, nakshatra lattices, returns,
lunations, ingresses), speed-zero for stations; the two step-size
hazards Teimeris recorded (a 40-day step swallowing Mercury's retrograde
arc; a Pluto pair 24 days apart) become tests. The kernel is
`first_zero` over the composite quantity; the design of the step rule
and the event kinds is this page's next revision.

## 5. The API

`Solver::new(sky, body, place, horizon, delta_t)`, `Solver::event(kind,
from, window_days)`, `Solver::day(midnight)`, `Solver::describe()`;
`Completion` implements `ApparentPositions` so a provider's positions
feed the solver through the port; `DrikSun` in `calendar::solar` wraps
both for the calendars and the local day. The port's `horizon_event` is
the native override, with a vtable slot and a kit check. C ABI: the
generic event entry points follow with `chart`; the horizon convention
crosses as three small enumerations and an altitude.

## 6. Errors and degenerate states

A polar day or night is `None` from `event` and a `DayEvents` without an
arc, never an error. A zero or negative window: `INVALID_ARG` naming
`window_days`. A source that cannot answer (an instant outside the
provider's data): the source's error. A scan that does not converge:
`NOT_CONVERGED` (never seen; every search has a cap). A horizon
convention an engine cannot search under (a custom altitude that is not
a twilight): the adapter's `Unsupported`, and the SDK's solver answers
under `PREFER_NATIVE`.

## 7. Performance budget

| operation | budget | measured (release, Apple Silicon) |
|---|---:|---:|
| the obliquity and nutation (IAU 2006, 2000B) | 2 µs | 0.79 µs |
| local apparent sidereal time | 2 µs | 1.16 µs |
| Delta T from the table | 50 ns | 7 ns |
| a grid of 100 instants by 8 bodies completed to equatorial with the SDK's obliquity, over the test provider | 0.5 µs per cell | 311 µs, 0.36 µs per cell above the provider's own 18 µs |
| a day (rise, set, midday) over the test provider | 50 µs | 26 µs, about ten readings of the sky |
| the same over Teimeris | 100 µs | ten engine calls at a few microseconds each |

The budgets are held by `crates/astro/benches/astro.rs`
(`cargo bench -p teistro-astro --bench astro`).

## 8. Tests

- Every ERFA port against the reference values of `t_erfa_c.c` at the
  tolerances the reference program uses; the 77-term nutation table
  checked row by row against the C source when ported.
- A fixed star on the equator rises and sets a quarter of a sidereal day
  from its transit; a circumpolar star never sets and a southern one
  never rises at Tromsø; a grazing star's rise and set are found, and
  the scan lands where the iteration does; the test provider's Sun gives
  a Kathmandu day under every convention with the conventions in their
  order (the upper limb with refraction first, then the lower limb, then
  the centre on the geometric horizon) and a civil twilight before them.
- The conformance kit's `override_rise_set_geometric` and
  `override_rise_set_refracted` checks compare a provider's own search
  with the solver over the same provider at three sea-level places on
  the kit's instants; measured against Teimeris: 0.13 s under the
  geometric convention (bound 1 s) and 7.3 s under the almanac's, at
  Reykjavík (bound 10 s; cruxes C34).
- The Teimeris adapter's fixture test (`tests/fixtures.rs`, by hand with
  the engine present) reproduces the baseline's sunrise and sunset for
  the 52 non-polar charts within 2.5 s below 60° of latitude and 9.8 s
  at Fairbanks on the solstice, and names the three charts whose
  baseline block is a day early (`fixtures/README.md`, convention
  twelve; C35).

## 9. Localisation

None: the solver emits instants and states; the convention's names are
the settings' keys.

## 10. Open questions

1. A settable atmosphere (pressure, temperature, a refraction model such
   as Bennett's) beside the almanac's 34 arcminutes, so a chart can ask
   for the engines' convention by name (C34).
2. The Moon's rise and set: the semidiameter from the mean radius against
   the eclipse convention (k = 0.2725), a difference under an arcsecond;
   to be pinned when the panchanga day computes moonrise.
3. The step rule and event kinds of the crossings and stations kernel
   (Phase 2).
