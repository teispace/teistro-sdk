# Astronomy: planetary phenomena and the equation of time

Status: `draft`, written 2026-09-05 when `astro::phenomena` and the
equation of time were built. Derives from
`01-research/platform/13-astronomy-layer.md` (the phenomena in the
Phase 2 list and the equation-of-time row), `07-roadmap/00-roadmap.md`
(Phase 2: elongation, magnitude, apparent diameter), the cruxes register
(C19, apparent disc diameters for the planetary war and combustion) and
`astro-events-and-crossings.md` (the disc and the radii the rise and set
solver already carries). Teimeris's `phenomena` and `equation_of_time`
are the rank-2 oracle; the magnitude models are the Astronomical
Almanac's, cited below.

## 1. Purpose and scope

What a body looks like from the Earth: how far it stands from the Sun
(combustion, the morning and evening star, heliacal phenomena), how much
of its disc is lit and at what angle (the phase), how large it appears
(the planetary war's rule of the larger disc, C19), how bright it is,
and, for the Moon, how much its place shifts for an observer on the
surface. And the equation of time: the difference between the sundial
and the clock that the panchanga's velantara states and local apparent
time needs. The horizontal transform and the twilights are the rise and
set page's; eclipses are v1.x.

## 2. Inputs, settings and ports

A body and a TT instant over the frame completion (`Completion`), which
asks the provider for the body and the Sun apparent and geocentric and,
when the provider answers heliocentric requests, for the body from the
Sun at the retarded instant; or a `Geometry` the caller supplies. The
equation of time takes a UT1 instant, a source of apparent positions
(`ApparentPositions`, which the completion implements) and the Delta T
model for the sidereal time. No settings knob is read: the phenomena are
geometry. Port: the ephemeris port through the completion; a provider
whose distances are not in astronomical units (a classical astronomy's
mean distances) is refused by name.

## 3. The data model

```rust
pub struct EclipticPosition { lon_deg, lat_deg, dist_au }                       // ecliptic of date
pub struct Geometry { body: EclipticPosition, sun: EclipticPosition, body_from_sun: Option<EclipticPosition> }
pub enum HeliocentricLeg { Provider, FromGeocentric }                             // where the Sun-to-body leg came from
pub struct Phase { angle_deg, illuminated_fraction }
pub struct Phenomena { body, elongation_deg, phase: Option<Phase>, disc: Disc, magnitude: Option<f64>, heliocentric_leg }
//   Phenomena::from_geometry(body, &geometry, tt), phenomena.apparent_diameter_deg()
pub fn phenomena(&Completion, Body, JulianDay<Tt>) -> Result<Phenomena, Error>
pub fn sky::equation_of_time_seconds(&dyn ApparentPositions, JulianDay<Ut1>, DeltaTModel) -> Result<f64, Error>
```

`Disc` (the semidiameter and the horizontal parallax from the IAU radii
and the WGS 84 Earth) is the rise and set solver's, shared. The phase is
`None` where there is none: the Sun, and the lunar points, which have no
disc. The magnitude is `None` for a point and where a fit does not reach
(Venus beyond a phase angle of 179°, as its authors state). The
heliocentric leg says whether the phase angle came from the provider's
heliocentric position or from the difference of the two apparent
vectors, which ignores the Sun's own light time and is within a few
arcseconds of phase angle.

## 4. Algorithms

**The geometry.** The elongation is the angle between the apparent
directions of the body and the Sun from the observer. The phase angle is
measured at the body between the Sun and the observer, so the body is
taken where the light now arriving left it: its heliocentric position at
the instant less the light time (`distance × 499.005 s / au`), which a
provider that answers heliocentric requests gives (Teimeris does; the
test provider does not, and the difference of the geocentric vectors
stands in). The illuminated fraction is `(1 + cos i) / 2`. The disc is
`Disc::of(body, distance)`: the semidiameter from the IAU radii (the
Sun's IAU 2015 nominal 695 700 km; Archinal et al. 2018 for the rest)
and the horizontal parallax from the WGS 84 equatorial radius.

**The magnitudes.** Every model shares the distance term
`5 log₁₀(r Δ)` (the body's distances from the Sun and the observer, au).
Mercury, Venus, Mars, Jupiter, Saturn and Uranus follow Mallama and
Hilton (2018), *Computing apparent planetary magnitudes for The
Astronomical Almanac*, Astronomy and Computing 25, 10, with the
Almanac's choices: Mercury's sixth-order polynomial; Venus's two
branches meeting at 163.7° and unfitted beyond 179°; Mars without the
rotational and orbital terms (tenths of a magnitude within hours, and no
better than 0.1 against JPL Horizons without them); Jupiter's quadratic
(its phase angle never passes 12°); Saturn with the tilt of its rings to
the Earth and to the Sun averaged (the ring pole of Meeus, chapter 45,
referred to the ecliptic of date at the retarded instant), nearly a
magnitude between edge-on and open; Uranus with the sub-Earth-latitude
term folded into its mean, −0.05. Neptune is a step of the calendar:
−6.89 before 1980, −7.00 after 2000 and a straight line between, at the
slope that keeps the curve continuous. Pluto is the IAU 1986 polynomial
with its phase terms at zero, −1.00. The Moon is Allen's (1976)
`−21.62 + 0.026 |i| + 4 × 10⁻⁹ i⁴` with the Earth-Moon distance in Earth
radii up to a phase angle of 147.1385465°, and Samaha's cubic in
`180° − i` beyond, the stitch being where the two agree, because Allen's
expression goes wrong for the thin crescent that first visibility turns
on. The Sun is −26.86 at one astronomical unit, scaled by the square of
its disc's ratio to the disc at one unit (the radius cancels).

**The equation of time.** Apparent solar time less mean solar time: the
Greenwich apparent sidereal time (the SDK's own, IAU 2000 with the 2000B
equation of the equinoxes) less the Sun's apparent right ascension is
the true Sun's hour angle at Greenwich; less the time since midnight
plus twelve hours is the mean Sun's; the difference folded to a half
turn, at four minutes a degree. Positive when the sundial is ahead:
about +16.4 minutes in early November and −14.2 in mid-February.

## 5. The API

`phenomena::phenomena(&completion, body, tt)` for a provider's
geometry; `Phenomena::from_geometry(body, &geometry, tt)` for one the
caller has (a recorded one, another ephemeris's); `Geometry` and
`EclipticPosition::new` to build it; `Phenomena::apparent_diameter_deg`;
`sky::equation_of_time_seconds(&sky, ut1, delta_t)`. C ABI and bindings
arrive with the chart layer: the record as six numbers and two options,
the equation of time as seconds.

## 6. Errors and degenerate states

A provider whose distances are not astronomical units: `UNSUPPORTED`
naming the provider and the unit. A body the provider does not carry:
the completion's error. A non-finite position or a negative distance in
a supplied geometry: `INVALID_ARG` naming `body`, `sun` or
`body_from_sun`. The Sun: no phase, elongation zero, the magnitude by
distance alone. A point: no phase, no disc, no magnitude, an elongation.
Venus within a degree of the Sun's far side: a phase and no magnitude. An
observer inside a body (a distance under the radius) is not a case the
port produces; `Disc::of` clamps the semidiameter at 90°.

## 7. Performance budget

| operation | budget | measured (release, Apple Silicon) |
|---|---:|---:|
| the phenomena of a planet over the test provider | 20 µs | 0.72 µs (two position requests, the arithmetic under a tenth of that) |
| the equation of time over the test provider | 10 µs | 2.96 µs (the sidereal time with its nutation, Delta T, one position) |

The arithmetic is a few hundred nanoseconds; the cost is the two or
three position requests through the completion.

## 8. Tests

- The Moon full at opposition (phase angle zero, lit, −12.7), half lit
  at quadrature, faint and on Samaha's branch a day after new; Venus half
  lit at −4.4 at greatest elongation, unfitted at inferior conjunction and
  on its crescent branch just outside; Jupiter −2.7 at opposition, Saturn
  brighter with open rings than edge-on, Neptune's 0.11 step between 1970
  and now, Pluto at 13.9; the Sun's inverse square and no phase; a node's
  elongation with no disc, phase or magnitude; a supplied heliocentric leg
  taken as given; a bad position refused by name. Over the test provider
  every body's quantities are in range with the leg from the geocentric
  vectors.
- Against Teimeris (`fixtures/teimeris/pheno.json`, the adapter's
  `pheno-table` binary; `crates/astro/tests/teimeris_phenomena.rs`): for
  eleven bodies at sixteen instants over two centuries, the SDK's
  arithmetic over the engine's own geometry reproduces its phase angle
  and elongation within 1e-9°, its illuminated fraction within 1e-12, its
  magnitudes to the rounding of the same formulae (under 1e-6; bound
  0.002 for the Sun's disc, whose radius cancels in the ratio); the
  apparent diameters to the rounding for every body but the Sun, whose
  older 696 000 km radius in the engine is 0.84″ of disc (bound 1″). The
  engine's horizontal parallax is compared for the Moon alone, which is
  the one body it reports it for, and it reads it from a distance up to
  40 km from the one its disc uses where the SDK reads both from the
  apparent distance: 0.32″ of the Moon's 3400″ (bound 0.5″). The engine
  writes zero for a point's magnitude where the SDK writes none. The
  equation of time from the engine's Sun with the SDK's sidereal time
  within 0.0003 s through 2030 (bound 0.001 s); beyond 2030 the engine's
  own equation of time and the Sun it places at a UT1 instant take
  different Delta T extrapolations, 0.12 s apart (1.9″ of hour angle),
  where the SDK's two are one construction; held at 0.2 s there.
- By hand over the engine's positions through the completion (the
  adapter's `tests/phenomena.rs`, ten bodies at six instants): the phase
  angles within 1.6e-12° and the elongations within 2.7e-13°, the
  heliocentric leg the engine's own at every instant; the magnitudes
  within 1.4e-7; the Sun's diameter 0.84″ and the Moon's parallax 0.32″
  apart as above; the equation of time over every day of the year 2000
  within 2 µs of the engine's.

## 9. Localisation

None: numbers, and the body keys.

## 10. Open questions

1. **Mars's rotational terms.** The Almanac's full Mars model adds terms
   for the sub-Earth longitude and the orbital longitude; they need the
   planet's rotation, which the SDK does not yet carry, and change the
   brightness by tenths of a magnitude within hours (the Almanac itself
   omits them for the same reason).
2. **Asteroids and the H-G system.** Chiron, Ceres and the rest take
   the H-G phase law with per-body elements; they arrive with the bodies.
3. **Heliacal phenomena.** First and last visibility from the magnitude,
   the altitude and the twilight; a rise and set page revision.
