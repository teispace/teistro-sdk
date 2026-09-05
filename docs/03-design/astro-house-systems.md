# Astronomy: house systems

Status: `draft`, written 2026-09-05 when `astro::houses` was built and
measured against the baseline's fixtures and Teimeris. Derives from
`01-research/platform/13-astronomy-layer.md` (the house systems row: every
system Swiss and Teimeris offer, the polar behaviour stated and gated,
1e-6° against Teimeris at ten latitudes), `03-design/core-types-and-catalogue.md`
(the twenty-two catalogued systems with their Swiss letter and degeneracy),
`03-design/settings-and-profiles.md` (the `houses.*` knobs) and
`03-design/astro-events-and-crossings.md` (sidereal time and the obliquity
the cusps stand on). The published definitions of each system are the
rank-1 sources named per row; the baseline's 55 charts (all systems, over
the Swiss Ephemeris) and Teimeris's recorded table are the rank-2
references.

## 1. Purpose and scope

A chart's houses are twelve cusps on the ecliptic of date from the
observer's meridian, latitude and the obliquity, each system a different
division of the sky; and a handful of auxiliary points every system
shares. This page settles the twenty-two catalogued systems as one
construction with twenty-two choices of circles, the auxiliary points,
the sign-based systems in a sidereal zodiac, the polar behaviour of each
system and the policy that decides it, and the measurements that hold
them. The placement of bodies into houses (which depends on a body's
latitude for the systems that divide semi-arcs or the prime vertical),
cusp speeds and the Gauquelin sectors are Phase 4's `houses` crate over
this layer (§10).

## 2. Inputs, settings and ports

The right ascension of the meridian (local apparent sidereal time, from
`sky::sidereal_time_deg`), the geographic latitude, the true obliquity of
date (`sky::obliquity`), the ayanamsha of a sidereal chart (zero for a
tropical one) and, for the Sunshine system alone, the Sun's declination.
The settings knobs are `houses.placement_system`, `houses.chalit_system`
(a second system for the Vedic bhava chalit), `houses.module_overrides`
and `houses.polar_policy` (`ERROR`, `FALLBACK_WHOLE_SIGN`,
`FALLBACK_PORPHYRY`, `CLAMP`); the profiles choose them. No port: the
computation is the SDK's own, and a provider's `HOUSES` override is a
kit row when a provider declares one.

## 3. The data model

```rust
pub struct Input { armc_deg, latitude_deg, obliquity_deg, sun_declination_deg: Option<f64>, sidereal_offset_deg }
pub struct Houses { system: HouseSystem, cusps: [f64; 12] /* house 1 first, tropical of date */, angles: Angles, outcome: Outcome }
pub struct Angles { ascendant_deg, midheaven_deg, armc_deg, vertex_deg, equatorial_ascendant_deg,
                    co_ascendant_koch_deg, co_ascendant_munkasey_deg, polar_ascendant_deg }
pub enum Outcome { Defined, Substituted { asked: HouseSystem }, Clamped { asked_latitude_deg } }
pub struct ChartFrame { sidereal_offset_deg, sun_declination_deg: Option<f64> }   // what a chart brings beside instant and place
```

Cusps are tropical longitudes of date; a sidereal chart subtracts its
ayanamsha from every cusp and angle alike. The sign-based systems (whole
sign, equal from 0° Aries) take their signs in the zodiac in use and
return the tropical longitudes of the sidereal sign boundaries, so the
subtraction leaves exact multiples of 30° (`sidereal_offset_deg`). The
provenance of a result names the system used and the outcome.

## 4. Algorithms

**One construction.** The ecliptic point where a great circle of pole
height f, whose ascending intersection with the equator is at right
ascension x, crosses the ecliptic of obliquity ε:

    λ = atan2(sin x, cos ε cos x − sin ε tan f)

The horizon (f the latitude φ, x the ARMC + 90°) gives the ascendant; an
hour circle (f = 0) at x = ARMC gives the midheaven and, at other right
ascensions, the meridian houses. Inside the polar circle the midheaven
can sink below the horizon and the formula's rising point is the
descendant: the signed distance from the midheaven to the ascendant is
then negative and the ascendant is turned by 180°, as every engine does.
Values within 1e-10° of a cardinal point snap to it, so latitude 0 and
ARMC 0 give an ascendant of exactly 90°.

**The systems**, with the circles each picks for cusps 11, 12, 2 and 3
(cusps 4 to 9 opposite 10, 11, 12, 1, 2, 3 unless said otherwise):

| system | letter | construction | source |
|---|---|---|---|
| `WHOLE_SIGN` | W | the eastern ascendant's sign in the zodiac in use, thirty degrees each | Hellenistic; the Vedic rashi chart |
| `EQUAL` | A | thirty degrees from the eastern ascendant | |
| `EQUAL_MC` | D | thirty degrees with the midheaven as cusp 10 | |
| `EQUAL_ARIES` | N | the signs from 0° Aries of the zodiac in use | |
| `VEHLOW` | V | equal houses with the ascendant at the middle of the first | Vehlow |
| `PORPHYRY` | O | the quadrants between the eastern ascendant and the midheaven trisected | Porphyry of Tyre |
| `SRIPATI` | S | Porphyry's sectors with the cusps at their middles: the Porphyry cusps are the bhava madhyas, the reported cusps the sandhis | Sripati, *Siddhantasekhara*; the Vedic bhava chalit |
| `REGIOMONTANUS` | R | great circles through the north and south points dividing the equator into 30° arcs: pole heights atan(tan φ / 2) and atan(tan φ cos 30°) at ARMC + 30°, 60°, 120°, 150° | Regiomontanus |
| `CAMPANUS` | C | the same circles dividing the prime vertical: pole heights asin(sin φ / 2) and asin(√3 sin φ / 2) at ARMC + 90° ∓ atan(√3 / cos φ), ∓ atan(1 / (√3 cos φ)) | Campanus |
| `TOPOCENTRIC` | T | the ascendant's construction at ARMC + 30°, 60°, 120°, 150° with the latitude's tangent scaled by 1/3, 2/3, 2/3, 1/3 | Polich and Page, 1961 |
| `ALCABITIUS` | B | the eastern ascendant's semi-diurnal arc (acos(−tan φ tan δ)) and semi-nocturnal arc trisected on the equator, the points carried to the ecliptic along hour circles | Alcabitius |
| `KOCH` | K | the midheaven's semi-arcs trisected in time and the ascendants at those times: with sin a = sin(MC) sin ε / cos φ, c = atan(tan φ / cos a), ad = asin(sin c sin a) / 3, the horizon's construction at ARMC + 30° − 2ad, 60° − ad, 120° + ad, 150° + 2ad; undefined inside the polar circle | Koch, 1971 |
| `PLACIDUS` | P | the ecliptic point that has covered a third (cusps 11, 3) or two thirds (12, 2) of its semi-diurnal arc on the circle meeting the equator at ARMC + 30°, 60°, 120°, 150°; the pole height depends on the answer's declination, so it iterates to a hundredth of an arcsecond, at most a hundred times; undefined inside the polar circle, and close to it the iteration may not settle | Placidus de Titis |
| `MERIDIAN` | X | the ecliptic points with right ascensions ARMC + 30 n along hour circles (cusp 1 the equatorial ascendant) | Zariel; axial rotation |
| `MORINUS` | M | the equator points ARMC + 30 n carried into ecliptic longitude along circles through the ecliptic poles | Morin de Villefranche |
| `CARTER` | F | hour circles at the eastern ascendant's right ascension plus 30 n | Carter's poli-equatorial |
| `HORIZON` | H | Campanus's geometry applied to the horizon by swapping the pole for the zenith (the latitude's complement, the meridian turned), cusp 1 the east point's vertical | azimuthal houses |
| `KRUSINSKI` | U | the great circle through the ascendant and the zenith cut into twelve, each point carried to the ecliptic along its hour circle | Krusinski, Pisa and Goelzer |
| `APC` | Y | the ascendant parallel circle's twelve sectors, each cusp its own great circle; the houses are not opposed in pairs, and the midheaven is the meridian's | Koppejan |
| `PULLEN_SD` | L | sinusoidal delta: house widths in a quadrant of size Q are 30° + d, 30° + 3d, … with d = (Q − 90°) / 4; a quadrant under 30° gives two coincident cusps | Pullen (Astrolog) |
| `PULLEN_SR` | Q | sinusoidal ratio: widths x, xr, xr³, xr⁴ in geometric progression, the ratio from the closed form of its quartic | Pullen (Astrolog) |
| `SUNSHINE` | I | the Sun's own diurnal and nocturnal arcs trisected (Treindl's construction of Makransky's system): each house point on the Sun's path, the great circle through it and the north and south points, its crossing of the equator and its pole height, then the construction above; the only system that needs the date; cusps not opposed in pairs; undefined where the Sun neither rises nor sets or the triangle collapses | Makransky; Treindl |

**The polar policy.** Placidus, Koch, Sunshine and, when its iteration
fails, Placidus near the circle are undefined at the place; the
catalogue marks the first three `POLAR_UNDEFINED`. `ERROR` refuses with
the system, the latitude and the reason; `FALLBACK_PORPHYRY` and
`FALLBACK_WHOLE_SIGN` return the substitute's cusps with
`Outcome::Substituted { asked }` (Porphyry is what the engines
substitute, so their cusps and the SDK's agree there); `CLAMP` computes
the system at the nearest latitude where it is defined, a millionth of a
degree inside the polar circle, with `Outcome::Clamped`. The circle
systems (Regiomontanus, Campanus, Topocentric, APC) are defined inside
the polar circle; when the midheaven is below the horizon they turn
their quadrant cusps and their reported midheaven with the ascendant,
while the equal and quadrant-trisecting systems turn the ascendant alone
and keep the meridian's midheaven, which is what the engines report.

**The auxiliary points.** The vertex is the horizon's construction at
ARMC − 90° with the latitude's complement as pole height, held in the
western hemisphere in the tropics; the equatorial ascendant the hour
circle at ARMC + 90°; Koch's co-ascendant the horizon's construction at
ARMC − 90° turned by 180°; Munkasey's co-ascendant the construction at
ARMC + 90° with the complement; the polar ascendant the construction at
ARMC − 90° with the latitude.

## 5. The API

Rust: `houses::houses(system, &Input, PolarPolicy) -> Result<Houses>`;
`houses::houses_at(system, ut1, tt, &Place, &ChartFrame, PolarPolicy)`,
which takes the meridian from the SDK's sidereal time and the obliquity
from its record; `houses::is_polar(latitude, obliquity)`;
`houses::circle_point(ra, pole, &Obliquity)` for a caller building its
own circle. C ABI and bindings arrive with the chart layer:
`ts_houses(system, jd_ut, place, frame, policy)` returning the cusps, the
angles and the outcome, and batch forms over instants.

## 6. Errors and degenerate states

| situation | outcome |
|---|---|
| a system this build has no construction for | `UNSUPPORTED`, field `houses.placement_system` |
| a non-finite meridian, latitude, obliquity or offset | `INVALID_ARG`, naming the field |
| Sunshine without the Sun's declination, or with one beyond ±24° | `INVALID_ARG`, field `sun_declination_deg` |
| a system undefined at the latitude under `ERROR` | `UNSUPPORTED`, naming the system, the latitude and the reason, with the three other policies as the hint, field `houses.polar_policy` |
| the poles | the latitude steps 1e-10° inside them; every system computes |
| a Pullen quadrant under 30° | two coincident cusps, as the definition says |
| the Sun's declination and the latitude with |tan δ tan φ| ≥ 1 (Sunshine) | the arcs are clamped as the engines do; a collapsed triangle is the undefined case above |

## 7. Performance budget

| operation | budget | measured (`cargo bench -p teistro-astro`, Apple M-series, 2026-09-05) |
|---|---:|---:|
| Placidus (four cusps iterated to convergence) with the angles | 5 µs | 2.2 µs |
| Regiomontanus with the angles | 2 µs | 0.33 µs |
| whole sign with the angles | 1 µs | 0.26 µs |

No allocation; the obliquity's trigonometry is computed once per call.

## 8. Tests

- Unit: the textbook angles (ARMC 0 at the equator: midheaven 0°,
  ascendant 90°; ARMC 90°: midheaven 90°, ascendant 180° at every
  latitude); every system computes at Kathmandu with cusp 1 the
  ascendant and cusp 10 the midheaven where the system has them and
  opposite cusps opposed where its houses oppose; the four polar policies
  at Tromsø (69.65°: refusal naming the system and latitude, Porphyry and
  whole-sign substitution, clamping to the circle); Sunshine's refusals;
  the sign-based systems in a sidereal zodiac.
- **Against the baseline's 55 charts** (`tests/baseline_houses.rs`: all
  twenty-two systems, 14 520 cusps, the fixture's ayanamsha turning its
  sidereal cusps tropical): within 0.00021° (three quarters of an
  arcsecond) for the charts between 1800 and 2200, the two engines'
  sidereal times; within 0.0033° for the two charts beyond 2200, where
  the engine behind the baseline switches to a long-term sidereal-time
  model; Sunshine within 0.05° at Fairbanks in June, where the Sun barely
  rises and the baseline's Sun is topocentric; the baseline's Placidus
  and Koch at Tromsø are Porphyry's cusps, as the SDK's substitution
  gives.
- **Against Teimeris** (`tests/teimeris_houses.rs` over
  `fixtures/teimeris/houses.json`, written by the adapter's
  `houses-table` binary: twenty-one systems at ten latitudes from −66° to
  80°, two longitudes, three instants, 25 194 cusps and angles): within
  4.8e-6° at worst (APC's and Koch's sixth cusps at 64.8°, where the
  constructions are steep) and 2.2e-6° on the vertex in the tropics; the
  twenty-four polar rows substituted by both sides alike; bound 1e-5°.
  The engine's Munkasey co-ascendant for the horizon system at the
  equator is skipped: its transform of the latitude leaves a value a hair
  below zero and the degenerate southern branch.

## 9. Localisation

None: systems are catalogue keys; the cusps and points are numbers.

## 10. Open questions

1. **House position of a body.** For the systems that divide semi-arcs
   (Placidus, Koch, Gauquelin), the prime vertical (Campanus) or the
   equator (Regiomontanus, Meridian), a body's house depends on its
   latitude as well as its longitude; the engines carry a construction per
   system. Phase 4's `houses` crate decides whether to carry them or to
   place by longitude with the difference stated.
2. **Cusp speeds** (degrees per day) for the systems with a closed form
   and by differencing for the others, with the chart layer's need for
   them.
3. **The Gauquelin sectors** (36) and the Sunshine alternative
   construction (Makransky's own, letter i) are not catalogued; they
   arrive with a consumer who asks.
4. **A provider's houses.** The port's `HOUSES` override is declared by
   no provider yet; when Teimeris's adapter declares it, the kit gains a
   row holding it to the SDK within the bound above.
5. **Sidereal houses by projection.** The engines offer, beside the
   traditional subtraction of the ayanamsha, house circles rebuilt
   against the ecliptic of the ayanamsha's epoch or the solar system's
   plane for the four frame-type ayanamshas; the SDK subtracts, and the
   projections wait for the `equinox` completion step
   (`astro-timescales-and-frames.md`).
