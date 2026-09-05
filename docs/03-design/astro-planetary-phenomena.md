# Astronomy: planetary phenomena and the equation of time

Status: `draft`, written 2026-09-05 when `astro::phenomena` and the
equation of time were built; §4's visibility and heliacal phenomena
(`astro::visibility`) added the same day. Derives from
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
surface. Whether a body near the Sun can be seen at dawn or dusk, and
the days it appears and disappears (the heliacal rising and setting, the
tradition's asta and udaya), under a named criterion. And the equation
of time: the difference between the sundial and the clock that the
panchanga's velantara states and local apparent time needs. The
horizontal transform and the twilights are the rise and set page's;
eclipses are v1.x.

## 2. Inputs, settings and ports

A body and a TT instant over the frame completion (`Completion`), which
asks the provider for the body and the Sun apparent and geocentric and,
when the provider answers heliocentric requests, for the body from the
Sun at the retarded instant; or a `Geometry` the caller supplies. The
equation of time takes a UT1 instant, a source of apparent positions
(`ApparentPositions`, which the completion implements) and the Delta T
model for the sidereal time. Visibility takes the completion, a place,
a criterion, the horizon convention the risings and settings are taken
under and the Delta T model. No settings knob is read: the phenomena are
geometry, and the criterion is named in the call (the chart layer's knob
when it exists). Port: the ephemeris port through the completion; a
provider whose distances are not in astronomical units (a classical
astronomy's mean distances) is refused by name for the phenomena, and
answers visibility, which needs longitudes and risings alone.

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

// visibility
pub enum Motion { Direct, Retrograde }                    // Motion::of_speed(rate)
pub enum Side { East, West }                              // the morning sky, the evening sky; Side::of_elongation(body − Sun)
pub struct Pair { direct, retrograde }                    // a body's two thresholds, degrees
pub struct Table { moon, mercury, venus, mars, jupiter, saturn, uranus, neptune, pluto: Option<Pair> }
pub enum Thresholds { SuryaSiddhanta, Ptolemy, Custom(Table) }   // .of(body, motion), .of_star(star), Thresholds::never_sets_heliacally(star)
pub enum Criterion { TimeDegrees { thresholds }, Longitude { thresholds }, ArcusVisionis { thresholds } }
//   Criterion::SURYA_SIDDHANTA, COMBUSTION_ORB, PTOLEMY; .key() "TIME_DEGREES/SURYA_SIDDHANTA"
pub struct Visibility { body, day_start, instant, side, motion, measure_deg, threshold_deg, visible, evaluations }
pub enum HeliacalKind { MorningFirst, MorningLast, EveningFirst, EveningLast }   // .side(), .appears()
pub struct HeliacalEvent { kind, day: Visibility }        // the first day seen, or the last
pub struct Heliacal<'a, P> { completion, place, criterion, horizon, delta_t }
//   .state(body, day_start), .events(body, from, to), .next(body, from, window_days), .day_start(at)
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

**Visibility and the heliacal phenomena** (`visibility`). Three
criteria, each a convention the caller names, because the tradition and
the astronomers disagree and both are legitimate. *Degrees of time*, the
Surya Siddhanta's (IX.2 to 11 and X.1): at sunrise, for a body west of
the Sun in longitude (the morning sky, `Side::East`), the interval in
oblique ascension between the body's rising and the Sun's; at sunset,
for a body east of it (the evening sky, `Side::West`), between the Sun's
setting and the body's; one degree of time is four minutes of sidereal
rotation (the text's "respirations divided by sixty"), and the SDK reads
the interval off the rise and set solver's two instants at the rate
360.985647° a day, which is the text's definition with exact geometry in
place of its rising-time tables and its horizon correction for latitude
(vii.8; the text's own arithmetic is the siddhanta page's open item).
The thresholds are the text's (IX.6 to 8, X.1): Jupiter 11, Saturn 15,
Mars 17; Venus 10 direct and 8 retrograde; Mercury 14 direct and 12
retrograde; the Moon 12; and the classes of the asterisms' junction
stars and the named stars (IX.12 to 15: thirteen, fourteen, fifteen,
seventeen and twenty-one; IX.18 names the six stars the Sun's rays never
extinguish at the text's latitude), carried as data for the star search
to come. *Longitude*, the tradition's combustion orb (asta): the
difference of ecliptic longitude at the same instant against the same
numbers, which the tradition reads as degrees of longitude (C44; the
retrograde variants of the tradition's table are IX.7 to 8, closing
C17). *Arcus visionis*, the classical astronomers' measure: the Sun's
depression below the horizon at the deepest twilight the body is up in,
its own rising or setting, or the Sun's antitransit when the body is
still up at that hour, so a body far from the Sun is seen and the
criterion says nothing false at opposition; the thresholds are Ptolemy's
(Almagest XIII.7 to 9, as Burgess quotes them under IX.9: Saturn 14°,
Jupiter 12°45′, Mars 14°30′, Venus 5°40′, Mercury 11°30′), or a caller's
table (Schoch's, a photometric model's). The body's motion at the
instant (the rate's sign) selects the threshold. The state of a local
mean day is the measure against the threshold; the heliacal events are
the days the state changes, scanned day by day: seen today and not
yesterday is a first on today's side (`MorningFirst`, the heliacal
rising; `EveningFirst`), seen yesterday and not today a last on
yesterday's (`MorningLast`; `EveningLast`, the heliacal setting). A
change of side alone is no event: seen on both days it is the
opposition, on neither the conjunction's passage. Every criterion runs
over any provider through the completion and the rise and set solver,
the classical astronomy included.

**The equation of time.** Apparent solar time less mean solar time: the
Greenwich apparent sidereal time (the SDK's own, IAU 2006 with the 2000B
nutation, `gst06b`) less the Sun's apparent right ascension is
the true Sun's hour angle at Greenwich; less the time since midnight
plus twelve hours is the mean Sun's; the difference folded to a half
turn, at four minutes a degree. Positive when the sundial is ahead:
about +16.4 minutes in early November and −14.2 in mid-February.

## 5. The API

`phenomena::phenomena(&completion, body, tt)` for a provider's
geometry; `Phenomena::from_geometry(body, &geometry, tt)` for one the
caller has (a recorded one, another ephemeris's); `Geometry` and
`EclipticPosition::new` to build it; `Phenomena::apparent_diameter_deg`;
`sky::equation_of_time_seconds(&sky, ut1, delta_t)`.
`Heliacal::new(&completion, place, criterion, horizon, delta_t)` with
`state(body, day_start)` for one day, `events(body, from, to)` for every
event whose day begins inside the window and `next(body, from,
window_days)` for the first; `day_start(at)` names the local mean day an
instant falls in (`sky::local_mean_midnight`); `Criterion::SURYA_SIDDHANTA`,
`COMBUSTION_ORB` and `PTOLEMY` are the three built-in conventions,
`Thresholds::Custom(Table)` a caller's. C ABI and bindings arrive with
the chart layer: the record as six numbers and two options, the equation
of time as seconds, a criterion as its two enumerations and an optional
table.

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

Visibility: the Sun itself is `INVALID_ARG` (name the body seen near
it); a point without light `UNSUPPORTED`; a body the criterion's table
does not place `UNSUPPORTED` naming the table and suggesting a custom
one; a day without a sunrise or a sunset (a polar day or night), or a
body with no rising or setting within half a day of the reference,
`LIMIT` naming the day and the latitude, since there is no dawn or dusk
to read at; a window that does not run forward `INVALID_ARG`. A body
seen on both sides of a change of side (the opposition) or on neither
(the conjunction) is no event.

## 7. Performance budget

| operation | budget | measured (release, Apple Silicon, one session on 2026-09-05) |
|---|---:|---:|
| the phenomena of a planet over the test provider | 20 µs | 0.89 µs (two position requests, the arithmetic under a tenth of that) |
| the equation of time over the test provider | 10 µs | 3.71 µs (the sidereal time with its nutation, Delta T, one position) |
| the visibility state of one day over the test provider | 100 µs | 51.0 µs by degrees of time, 72.0 µs by the arcus visionis (ten to twenty position reads: the Sun's rise and set, the body's rising or setting, the body and the Sun in one request; the arcus visionis adds the Sun's antitransit and one altitude) |

The arithmetic is a few hundred nanoseconds; the cost is the two or
three position requests through the completion, and for a heliacal scan
the day-by-day readings, about fifteen a day. Rows compare within a
table: the machine's state moves every row by tens of per cent between
sessions (`astro-events-and-crossings.md` §7).

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
  within 0.0003 s through 2030 (bound 0.001 s); from 2050 the engine's
  sidereal time steps by 1.9″ where its long-term branch takes over
  (`05-testing/02-engine-findings.md`, F1), so its equation of time is
  0.127 s from the one its own Sun implies; the SDK's GAST is continuous
  and the comparison is held at 0.2 s there.
- Visibility (`visibility::tests`): the text's thresholds verse by
  verse, Ptolemy's, a custom table; the star classes cover the
  twenty-seven junction stars (four at thirteen, nine at fourteen, seven
  at fifteen, four at seventeen, three at twenty-one) and the four named
  stars, and the six that never set are IX.18's; the sides, motions and
  kinds read their signs; over the test sky, whose Mercury always outruns
  the Sun, every criterion finds alternating morning-last and
  evening-first events, each on a day seen with the neighbouring day not,
  `next` agreeing with `events`; the refusals name their reasons, a
  polar midsummer among them.
- By hand over the engine's positions (the adapter's
  `tests/visibility.rs`), the three criteria against the day the
  engine's photometric visibility model names, at Kathmandu: Venus's
  heliacal rising of June 2020 one to two days earlier than the model
  (degrees of time 9.01° against 8; longitude 8.21°; arcus 5.96° against
  5.67°), its heliacal setting of May 2020 within a day, Jupiter's
  heliacal rising of February 2021 a day later by degrees of time, three
  days earlier by longitude and seven later by Ptolemy's 12°45′ (the
  model sees Jupiter at a smaller depression); held within ten days. The
  criteria are different definitions; the days differ by their nature.
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
3. Closed: **heliacal phenomena** compute under three named criteria
   (§4). Open from it: a photometric criterion (Schaefer's model, which
   the engine implements) as a fourth convention, needing the extinction
   and sky-brightness formulae from their primary sources; the stars'
   heliacal search, whose thresholds the text supplies (IX.12 to 15) and
   which waits for a star as a source of apparent positions to the rise
   and set solver; and the Moon's first crescent, which the text treats
   by the same rule (X.1) and modern astronomy by Yallop's or Odeh's
   criteria over the crescent's width.
