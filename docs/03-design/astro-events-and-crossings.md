# Astronomy: events and crossings

Status: `draft`, written 2026-09-05 when the ephemeris port was promoted
and the rise and set solver built; revised the same day when the
crossings and stations kernel was built over the boundary solver and the
solver's refinement moved from bisection to the ITP method. Derives from
`01-research/platform/13-astronomy-layer.md` (the solver rows and the
conformance targets), `02-architecture/02-ephemeris-port.md` (the rise
and set override), ADR-0013 (the override policy) and ADR-0021 (the
ERFA ports the solver stands on). The baseline engine's sunrise and
panchanga transitions in the 55 fixture charts are the rank-2 reference;
Teimeris's own searches the rank-2 oracle for the solvers.

## 1. Purpose and scope

Every event search in the SDK is a root of one quantity of time: a
sankranti is the Sun's longitude reaching a sign boundary, a sunrise is a
body's altitude reaching a horizon convention, a tithi's end is the
Moon's elongation from the Sun reaching a multiple of twelve degrees, a
station is a speed reaching zero. This page settles the three solvers
built so far, the boundary solver, the rise and set solver and the
crossings and stations kernel, the horizon conventions, the sidereal
time and the obliquity they need, and the measurements that hold them.

## 2. Inputs, settings and ports

A rise or set needs a source of apparent geocentric equatorial positions
of date (`ApparentPositions`: the frame completion over a provider, or a
classical model), a place, a horizon convention and a Delta T model for
the sidereal time. A crossing or a station needs a source of longitudes
and their speeds in one frame (`Longitudes`: the completion in a frame,
with an observer for a topocentric one, or a classical model), the
quantity searched, the lattice of boundaries and a tolerance. The
settings knobs read are `day.sunrise` (the convention:
`CENTRE_NO_REFRACTION`, `UPPER_LIMB_REFRACTION`, `LOWER_LIMB_REFRACTION`
or a custom altitude of the centre without refraction), `time.delta_t`,
`provider.overrides` (a provider's own rise and set search is used under
`PREFER_NATIVE` when it declares the `RISE_SET` override), and, through
the frame a search runs in, the zodiac and the ayanamsha (a nakshatra
lattice is sidereal; a tithi lattice is an elongation and needs neither).
Ports: the ephemeris port through the completion.

## 3. The data model

```rust
// the port (crates/port-ephemeris)
pub enum HorizonEventKind { Rise, Set, Transit, Antitransit }
pub enum DiscPoint { Centre, UpperLimb, LowerLimb }
pub enum Refraction { None, Standard }
pub struct Horizon { disc: DiscPoint, refraction: Refraction, altitude_deg: f64 }   // from_convention(SunriseConvention), key()
pub struct HorizonRequest { body, kind, place: Place, from: JulianDay<Ut1>, window_days, horizon }
trait EphemerisProvider { fn horizon_event(&self, &HorizonRequest) -> Result<Option<JulianDay<Ut1>>, ProviderError>; ... }

// the astronomy layer (crates/astro): rise and set
pub struct Apparent { ra_deg, dec_deg, distance_au }
pub trait ApparentPositions { fn apparent(&self, Body, JulianDay<Ut1>) -> Result<Apparent, Error>; fn describe(&self) -> String; }
pub struct Solver<'a> { sky: &'a dyn ApparentPositions, body, place, horizon, delta_t: DeltaTModel }
pub struct HorizonEvent { instant: JulianDay<Ut1>, method: Method /* Iterated | Scanned */, evaluations: u32 }
pub struct DayEvents { rise: Option<HorizonEvent>, set: Option<HorizonEvent>, above_at_midday: bool }   // arc()
pub struct Disc { semidiameter_deg, parallax_deg }                                                     // Disc::of(body, distance_au)
pub fn centre_altitude_deg(&Horizon, &Disc) -> f64

// the boundary solver
pub struct Caps { bracket_steps: u32, refinements: u32 }                                               // DEFAULT: 64 and 64
pub struct Crossing { instant: f64, width: f64, evaluations: u32 }
pub fn solve::next_crossing(angle, target, from, rate, tolerance, Caps) -> Result<Crossing, SolveError>
pub fn solve::first_zero(f, from, to, step, upward, tolerance, Caps) -> Result<Option<Crossing>, SolveError>
pub fn solve::refine(f, lo, hi, tolerance, Caps) -> Result<Crossing, SolveError>                      // f(lo) < 0 <= f(hi)

// crossings and stations (astro::events)
pub trait Longitudes { fn longitude_and_speed(&self, Body, JulianDay<Ut1>) -> Result<(f64, f64), Error>; fn describe(&self) -> String; }
pub struct FrameLongitudes<'c, P> { completion, frame, observer: Option<Place> }                     // Completion::longitudes(frame).with_observer(place)
pub enum Quantity { Longitude(Body), Speed(Body), Composite { a, first, b, second } }                 // ELONGATION, MOON_PLUS_SUN, separation(a, b)
pub struct Lattice { origin_deg, step_deg }                                                          // SIGNS, NAKSHATRAS, TITHIS, KARANAS, YOGAS, single(target)
pub enum Direction { Rising, Falling }
pub struct Event { instant: JulianDay<Ut1>, boundary_deg, direction, evaluations }
pub enum StationKind { Retrograde, Direct }
pub struct Station { instant: JulianDay<Ut1>, longitude_deg, kind, evaluations }
pub struct Search<'s, S: Longitudes + ?Sized> { source, quantity, lattice, tolerance_days, step_days: Option<f64>, caps }
pub fn events::stations(&dyn Longitudes, Body, from, to, tolerance_days) -> Result<Vec<Station>, Error>
pub fn events::greatest_rate(Body) -> f64                                                            // degrees a day, the table below
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

A quantity is a longitude (a sign ingress, a nakshatra), a composite
`a·lon(A) + b·lon(B)` (the tithi and the karana are the elongation,
Moon − Sun; the yoga is Moon + Sun; a separation for an aspect is
A − B; a return is a single longitude against a single target) or a
speed (a station). A lattice is an origin and a spacing: the signs
(30°), the nakshatras and the yogas (360/27°), the tithis (12°), the
karanas (6°), or a single target with no spacing. An event says which
boundary was reached and which way: `Rising` when the quantity was
increasing through it, `Falling` when decreasing, so a retrograde
re-entry into a sign is the falling crossing of the boundary the body
rose through days before, and a search's rising events less its falling
ones equal the boundaries the quantity advanced past net.

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
narrows it through the shared solver's `first_zero`, and says so
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

**The boundary solver** (`solve`). Three entry points over one
narrowing. `next_crossing`: a jump toward the target by the quantity's
mean rate, a bracket in which the signed gap changes sign, the
narrowing to the tolerance, every loop capped. `first_zero`: the first
sign change of a quantity inside a window, stepped then narrowed,
absence reported rather than guessed. `refine`: a bracket the caller
found (`f(lo) < 0 ≤ f(hi)`), narrowed. The narrowing is the ITP method
(interpolate, truncate, project; Oliveira and Takahashi, *ACM
Transactions on Mathematical Software* 47, 2021): each step takes the
secant's root, pulls it toward the midpoint by a tenth of the width
times the width's ratio to the first width (so the pull shrinks with
the square of the width and the unit of time does not matter), and
confines it to the interval a bisection would have reached by that
step. A smooth curve converges superlinearly, a day-wide bracket to a
tenth of a millisecond in six or seven evaluations; no curve costs more
than a bisection's count plus one (a day to a microsecond in 38), which
is what the caps assume. The method needs no derivative, so the same
code serves a tabular classical model and a modern ephemeris, and the
found instant is the middle of a bracket no wider than the tolerance, as
it was under bisection.

**Crossings** (`events::Search`). A bracketing scan then the shared
narrowing. The quantity is sampled from the window's start at a step of
half the lattice spacing over the quantity's greatest rate, capped at a
day: `min(spacing / (2 · rate), 1 day)`, where the rate is the sum of
the bodies' greatest rates weighted by the composite's coefficients
(the table: Sun 1.03, Moon 15.5, Mercury 2.3, Venus 1.3, Mars 0.9,
Jupiter 0.3, Saturn 0.15, Uranus 0.07, Neptune and Pluto 0.05, the mean
node and apogee 0.12, the true node and the osculating apogee 0.6
degrees a day; any body added later 2.3). Between two samples the
quantity moves less than half a spacing, so it passes each lattice line
at most once and no boundary can be crossed and recrossed unseen:
Mercury's retrograde arc, which a forty-day step swallowed in an engine's
record, lasts three weeks against a step of a day; a single target is a
spacing of a whole circle and the daily cap. The samples are unwrapped
into a continuous curve (each step adds the wrapped difference), the
lattice lines strictly above the earlier sample and up to the later one
are listed (the far sample owns a line met exactly), and each is
narrowed by `refine` over the signed distance to the line along the
same unwrapped curve, so the bracket's ends carry the very values the
lattice test saw and a line met at a sample still brackets. The
tolerance is 1e-7 days (under a hundredth of a second); the steps are
capped at two million samples (five and a half thousand years at a
day's step), beyond which the search is `NOT_CONVERGED` naming the cap.
`next_within` is the first event of a window.

**Stations** (`events::stations`). The speed's sign changes inside the
window, found by `first_zero` at a step of a day in both directions
(downward: `Retrograde`, the body about to run back; upward: `Direct`),
merged in time order, each carrying the longitude where the body stood.
A speed is arcseconds a day near a station, so the instant is soft by
construction: the engines that publish stations differ among themselves
by minutes there, and the SDK's number is the zero of the speed its
source reports.

## 5. The API

`Solver::new(sky, body, place, horizon, delta_t)`, `Solver::event(kind,
from, window_days)`, `Solver::day(midnight)`, `Solver::describe()`;
`Completion` implements `ApparentPositions` so a provider's positions
feed the solver through the port; `DrikSun` in `calendar::solar` wraps
both for the calendars and the local day. The port's `horizon_event` is
the native override, with a vtable slot and a kit check.

`Completion::longitudes(frame)` (`.with_observer(place)` for a
topocentric frame) is the source of longitudes; `Search::new(&source,
quantity, lattice)` with `with_tolerance_days` and `with_step_days` to
override the defaults, `between(from, to)` for every event of a window
in time order and `next_within(from, window_days)` for the first;
`stations(&source, body, from, to, tolerance_days)`;
`Completion::crossings(&request)` for the same search under the override
policy, a provider that declares the `CROSSINGS` override answering with
its own search under `PREFER_NATIVE` and `NATIVE_ONLY` and the kernel
otherwise (a request the provider refuses, a topocentric frame, falls to
the kernel under `PREFER_NATIVE`), the result saying which
(`Crossings { events, implementation }`). The vocabulary (`Quantity`,
`Lattice`, `Direction`, `Event`, `CrossingRequest`) is the port's
(`teistro_port_ephemeris::crossing`), re-exported here. `Quantity::Longitude(body)`,
`Quantity::ELONGATION`, `Quantity::MOON_PLUS_SUN`,
`Quantity::separation(a, b)`, `Quantity::Speed(body)`; `Lattice::SIGNS`,
`NAKSHATRAS`, `TITHIS`, `KARANAS`, `YOGAS`, `Lattice::single(target)`.
Every event carries how many evaluations placed it. C ABI: the generic
event entry points follow with `chart`; the horizon convention crosses
as three small enumerations and an altitude, a quantity and a lattice as
their tagged records.

## 6. Errors and degenerate states

A polar day or night is `None` from `event` and a `DayEvents` without an
arc, never an error. A zero or negative window: `INVALID_ARG` naming
`window_days`; a search window that does not run forward: `INVALID_ARG`
naming `to`; a tolerance, step or lattice that is not a positive finite
number: `INVALID_ARG` naming the field. A source that cannot answer (an
instant outside the provider's data, a body it does not carry): the
source's error. A scan that does not converge or exceeds the sample cap:
`NOT_CONVERGED` (never seen; every search has a cap). A horizon
convention an engine cannot search under (a custom altitude that is not
a twilight): the adapter's `Unsupported`, and the SDK's solver answers
under `PREFER_NATIVE`. A window with no crossing or no station is an
empty list, not an error.

## 7. Performance budget

| operation | budget | measured (release, Apple Silicon) |
|---|---:|---:|
| the obliquity and nutation (IAU 2006, 2000B) | 2 µs | 0.79 µs |
| local apparent sidereal time | 2 µs | 1.16 µs |
| Delta T from the table | 50 ns | 7 ns |
| a grid of 100 instants by 8 bodies completed to equatorial with the SDK's obliquity, over the test provider | 0.5 µs per cell | 311 µs, 0.36 µs per cell above the provider's own 18 µs |
| a day (rise, set, midday) over the test provider | 50 µs | 26 µs, about ten readings of the sky |
| the same over Teimeris | 100 µs | ten engine calls at a few microseconds each |
| the Sun's twelve ingresses of a year over the test provider | 500 µs | 160 µs: 366 samples and at most nine evaluations an event |
| the thirty tithis of a lunation over the test provider | 1 ms | 233 µs: 77 samples of two bodies and at most nine evaluations an event |

The budgets are held by `crates/astro/benches/astro.rs`
(`cargo bench -p teistro-astro --bench astro`). The narrowing's cost is
what moved: the same events took twenty-four to thirty evaluations each
under bisection.

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
- The solver: a sine from a day-wide bracket to a tenth of a millisecond
  in at most nine evaluations, a step function in at most the
  bisection's thirty plus one, a bracket without a sign change refused,
  and a property test of perturbed motions whose gap changes sign inside
  the tolerance around every answer.
- The kernel over the test provider: the Sun's twelve ingresses of a
  year in order at the boundaries and the tithis of a lunation twelve
  degrees apart; the lattice lines a curve passes, forward and back,
  with a line met exactly counted once; the step rule; refusals by name.
  Over a synthetic looping planet (half a degree a day with a twelve
  degree epicycle of a hundred days): eight stations in 400 days
  alternating retrograde and direct at zero speed, ten sign crossings of
  which two are falling, the net advance of six boundaries the mean
  motion predicts, and a single target giving the same instants as its
  lattice line.
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
- The Teimeris adapter's crossings test (`tests/crossings.rs`, by hand
  with the engine present) holds the kernel over Teimeris's positions to
  the engine's own searches over the year from J2000: the Sun's twelve
  ingresses within 0.001 s, Mercury's fourteen (its retrograde
  re-entries among them) within 0.004 s, the twenty-nine tithi boundaries
  of a lunation within 0.004 s, and Mercury's twelve and Mars's two
  stations of two years within 0.3 s (bound 1 s for a crossing, 600 s for
  a station). Against the baseline's panchanga day in the 55 charts (the
  280 tithi, nakshatra, yoga and karana ends inside the days, which the
  baseline reckoned geocentrically in Lahiri's zodiac; `fixtures/README.md`,
  convention one): within 7.8 s, median 3.3 s. The positions are the same
  engine lineage's, so the residual is the baseline's own search, which
  stops within 0.001° of the boundary (seven seconds of the Moon's
  elongation), not the kernel's; the band on the fixtures'
  `panchanga_day.*.end_jd` is set from that spread when the harness
  lands.

## 9. Localisation

None: the solvers emit instants and states; the convention's names are
the settings' keys.

## 10. Open questions

1. A settable atmosphere (pressure, temperature, a refraction model such
   as Bennett's) beside the almanac's 34 arcminutes, so a chart can ask
   for the engines' convention by name (C34).
2. The Moon's rise and set: the semidiameter from the mean radius against
   the eclipse convention (k = 0.2725), a difference under an arcsecond;
   to be pinned when the panchanga day computes moonrise.
3. One request for both bodies of a composite quantity, halving the
   completion's overhead per sample. (The `CROSSINGS` override exists:
   Teimeris's own search answers under `PREFER_NATIVE`, held to the kernel
   by the kit's two crossings checks within 0.004 s; a native stations
   search of its own is not needed, a station being a crossing of the
   speed.)
