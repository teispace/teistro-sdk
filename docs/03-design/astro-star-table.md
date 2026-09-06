# Astronomy: the star table

Status: `draft`, written 2026-09-05 when the star table was built: the
catalogue kind `star`, the `stars` module of `crates/astro` and the
twelve star-anchored ayanamshas it unlocks. Derives from
`01-research/platform/13-astronomy-layer.md` (the star table row and
its 0.01″ target), `astro-ayanamsha-catalogue.md` (the anchored
definitions), ADR-0021 (the ERFA ports) and `core-types-and-catalogue.md`
(a kind per enumerable thing). Teimeris's fixed-star search is the rank-2
oracle; SIMBAD's astrometry (Hipparcos, the new reduction; Gaia DR3) the
rank-1 data.

## 1. Purpose and scope

A fixed star has a place too: the sidereal zodiacs anchored to Spica, ζ
Piscium, δ Cancri, λ Scorpii, the galactic centre and the galactic poles
need it, the nakshatras' yogataras (their junction stars) are named
after it, and the fixed-star tradition of Western astrology reads
conjunctions with it. This page settles the catalogue of stars the SDK
carries with their astrometry and provenance, the computation that
carries a catalogue direction to the equator and ecliptic of a date
(proper motion, parallax, deflection, aberration, the frame bias,
precession and nutation), the Earth ephemeris that computation stands
on so that a star's place is the SDK's own over every provider, the
anchored ayanamshas' reading of it, and the measurements against
Teimeris. A topocentric star (the observer's own rotation in the
aberration) and the full catalogue of a few hundred named stars are
v1.x.

## 2. Inputs, settings and ports

A catalogued member (`Star`) or an `Astrometry` a caller supplies (a
direction at an epoch, proper motions, parallax, radial velocity), a TT
instant, the precession model (`PrecessionModel`, Vondrák 2011 by
default) and the corrections wanted (`Corrections`: nutation,
aberration, deflection, parallax). No settings knob is read directly:
the star-anchored ayanamshas read the table through `frame.ayanamsha`,
and a chart's star positions will read the frame knobs when the chart
layer exists. No port: the Earth's position and velocity are the SDK's
own (§4), so no provider takes part.

## 3. The data model

```rust
// the catalogue (catalogue/star.yaml, kind 56; catalogue/star_class.yaml, kind 57)
pub enum Star { Sheratan, Bharani, Alcyone, Aldebaran, /* … 128 members … */ SgrAStar, GalacticPole, GalacticPoleIau1958 }
pub struct StarAttributes { designation: &'static str, class: StarClass, hip: Option<i32>, magnitude: Option<f64>, astrometry: Astrometry, yogatara_of: Option<Nakshatra> }
pub struct Astrometry { ra_deg, dec_deg, pm_ra_mas_yr, pm_dec_mas_yr, parallax_mas, radial_velocity_km_s }   // ICRS, epoch J2000.0
pub enum StarClass { Star, RadioSource, Direction }

// the astronomy layer (crates/astro::stars)
pub struct Astrometry { ra_deg, dec_deg, pm_ra_mas_yr, pm_dec_mas_yr, parallax_mas, radial_velocity_km_s, epoch: JulianDay<Tt> }
//   Astrometry::of(Star), Astrometry::icrs(ra, dec).with_proper_motion(..).with_parallax(..).with_radial_velocity(..).at_epoch(..)
pub struct Corrections { nutation, aberration, deflection, parallax: bool }        // APPARENT, MEAN, GEOMETRIC
pub struct Options { precession: PrecessionModel, corrections: Corrections }       // APPARENT, MEAN, GEOMETRIC, with_precession()
pub struct Place { lon_deg, lat_deg, ra_deg, dec_deg }                             // the equator and ecliptic of date
pub fn place(&Astrometry, JulianDay<Tt>, &Options) -> Result<Place, Error>
pub fn place_of(Star, JulianDay<Tt>, &Options) -> Result<Place, Error>
pub fn yogatara(Nakshatra) -> Option<Star>

// the Earth (crates/astro::iau::epv00)
pub struct EarthState { heliocentric: PositionVelocity, barycentric: PositionVelocity, inside_span: bool }
pub fn epv00(date1, date2) -> EarthState

// the anchored ayanamshas (crates/astro::ayanamsha)
Definition::Object { anchor: Star, held_deg: [f64; 2], reading: Reading /* Longitude | RightAscension | GeometricLongitude */ }
```

**The catalogue.** 128 members: the 27 yogataras (with Vega, the
yogatara of Abhijit), the anchors of the ayanamshas (Spica, Revati
(ζ Piscium A), Asellus Australis (δ Cancri), Shaula (λ Scorpii),
Sagittarius A*, the north galactic pole of Liu, Zhu and Zhang (2011) and
the IAU 1958 pole referred to the ICRS), and the bright and traditional
stars of the fixed-star literature down to about the third magnitude
(Sirius to Errai), 125 stars in all. Keys are the IAU-approved proper
names (`SPICA`, `ASELLUS_AUSTRALIS`, `KAUS_MEDIA`); the designation
attribute is the Bayer or Flamsteed name in full (`alpha Virginis`,
`41 Arietis`). Every row's astrometry is SIMBAD's (CDS Strasbourg, a
TAP query of 2026-09-05 over the `basic` and `ident` tables): ICRS
coordinates at epoch J2000.0, proper motion (the right ascension rate
with cos δ applied), parallax, radial velocity and V magnitude, and
every row cites the bibcode of each value's source, so a Hipparcos row
(van Leeuwen 2007, `2007A&A...474..653V`) is told from a Gaia DR3 row
(`2020yCat.1350....0G`) by reading it. Sagittarius A* carries SIMBAD's
VLBI position (Petrov et al. 2011) and Reid and Brunthaler's (2020)
apparent proper motion; the poles carry no motion. The yogatara
identifications are Burgess's (1860), chapter VIII: Vishakha's is α
Librae (the text's own is ι Librae) and the 28-nakshatra Abhijit is a
doc note on Vega, since the nakshatra kind has twenty-seven members.

**A place** is on the equator and ecliptic of date: mean when
`nutation` is off, true when on, with the obliquity the precession
model is consistent with. `Corrections::APPARENT` is what an observer
at the Earth's centre sees; `MEAN` the same on the mean equator (what a
mean ayanamsha reads); `GEOMETRIC` the direction itself, proper motion
and precession only.

## 4. Algorithms

**The Earth** (`iau::epv00`). ERFA's `epv00`, ported with its 1951
coefficient rows: the Earth's heliocentric and barycentric position and
velocity in the BCRS from a simplified VSOP2000 (Moisson and
Bretagnon, 2001) oriented to DE405, within 13.4 km and 4.9 mm/s of
DE405 over 1900 to 2100, the errors doubling by 1800 and 2200 and
growing tenfold by 1500 and 2500. The aberration it feeds is therefore
within a few microarcseconds of the ephemeris's over the modern
centuries and within a milliarcsecond over the classical ones, which is
why a star's place can be the SDK's own: every provider gives the same
sidereal longitudes under a star-anchored zodiac, as the ayanamsha page
requires of every zodiac.

**The place** (`stars::place`). ERFA's `atciq` chain, ported piece by
piece (`iau::apparent`): `pmpx` carries the catalogue direction to the
date by its space motion (the perspective and radial-velocity terms
included) and takes off the annual parallax from the observer's
barycentric position; `ldsun` bends the direction toward the Sun by the
Schwarzschild term, limited for a source behind it; `ab` applies the
relativistic aberration for the observer's barycentric velocity in
units of c; the frame bias (`bp06`'s bias matrix) takes the ICRS
direction to the J2000.0 mean equator; the precession model's matrix
(`precession::to_date`) takes it to the mean equator of date; `numat`
with the IAU 2000B nutation takes it to the true equator when asked;
the equatorial vector is read as right ascension and declination, then
rotated by the obliquity (mean, or mean plus the nutation in obliquity)
and read as longitude and latitude. Every routine reproduces ERFA's
reference values (`t_erfa_c.c`) to the tolerances the reference program
uses.

**The anchored ayanamshas** (`ayanamsha::anchored_value_deg`). The
anchor's place on the mean equator of date under the model in use, read
as its longitude (`Reading::Longitude`: aberration and deflection
applied, as the definitions' author's software computes them, so the
mean value of a star-anchored member carries the star's annual
aberration, ±20″ over the year, and its rate swings by ±130″ a year
around the general precession), as its right ascension projected to the
ecliptic along its meridian (`RightAscension`, Wilhelm's galactic
centre: the midheaven of that right ascension, eight arcminutes from the
centre's longitude reading), or as its geometric longitude
(`GeometricLongitude`, the three galactic-equator members: a pole is a
construction of planes, and bending its light would move the
intersection); less the sidereal longitude the anchor is held at, the
two terms of Gil Brand's `210 + 90 × 0.3819660113` and of `150 + 6°40′`
subtracted in turn so the rounding matches every published table. The
nutation in longitude is added by `Basis::True` afterwards, as for every
member; the position request's own flags do not reach the ayanamsha,
which is a property of the zodiac, not of the chart's centre.

## 5. The API

`Star::ALL`, `Star::from_key`, `star.attributes()` (the designation,
class, Hipparcos number, magnitude, astrometry and the nakshatra it is
the yogatara of), `star.sources()`; `stars::Astrometry::of(star)` or
`Astrometry::icrs(ra, dec)` with its builders and `at_epoch` for a
Gaia row stated at J2016.0; `stars::place(&astrometry, tt, &options)`
and `stars::place_of(star, tt, &options)` with `Options::APPARENT`,
`MEAN`, `GEOMETRIC` or a custom `Corrections` and `with_precession`;
`stars::yogatara(nakshatra)`; `iau::epv00::epv00` for the Earth. The
anchored ayanamshas need no new call: `ayanamsha::mean_deg` and
`value_deg` answer for them, `is_computable` is true for every defined
member, and the completion's sidereal zodiac follows. C ABI: the star
kind crosses as a key like any catalogue member; a place as four
degrees; the corrections as four flags; with `chart`.

## 6. Errors and degenerate states

A non-finite astrometric parameter: `INVALID_ARG` naming the field. A
declination at a pole: `INVALID_ARG` naming `dec_deg` (a star there has
no right ascension rate). A negative parallax: `INVALID_ARG` naming
`parallax_mas`, with the hint to give zero for a star without one; zero
places the star at infinity and idles its radial velocity. An instant
outside the Earth ephemeris's 1900 to 2100 span is answered, less well,
and `EarthState::inside_span` says so; a caller who needs the flag reads
`epv00` directly. A star behind the Sun is deflected by the limited
formula, never infinitely. A nakshatra without a yogatara does not
exist: `yogatara` returns `Some` for all twenty-seven.

## 7. Performance budget

| operation | budget | measured (release, Apple Silicon) |
|---|---:|---:|
| the Earth's barycentric state (`epv00`, 1951 terms) | 20 µs | 12.6 µs |
| a star's apparent place | 25 µs | 13.9 µs, the Earth's state and under two microseconds of corrections |
| an anchored ayanamsha, mean | 25 µs | 13.7 µs (an epoch-defined member is 0.59 µs) |

The budgets are held by `crates/astro/benches/astro.rs`. A request for
a hundred instants under a star-anchored zodiac spends about 1.4 ms on
its ayanamshas; a cache of the Earth's state by day is the first
optimisation should a consumer need one (§10).

## 8. Tests

- Every ERFA port against the reference values of `t_erfa_c.c` at the
  tolerances the reference program uses: `epv00` (both vectors), `pmpx`,
  `ab`, `ld`, `ldsun`, `numat`.
- The catalogue: `cargo xtask check-catalogue` holds every row to its
  schema, keys and sources; `stars::yogatara` answers for all twenty-seven
  nakshatras.
- The place: at J2000.0 the geometric place is the catalogue direction
  turned by the frame bias alone; the aberration is 1″ to 21″, the
  nutation 1″ to 18″, the deflection under an arcsecond; a century of
  precession carries Spica 50.25″ a year (the general precession less its
  own drift), Arcturus's proper motion 228″ a century, α Centauri's
  parallax a shift of 0.3″ to 0.75″; bad astrometry is refused by name.
- The anchored ayanamshas: Spica held at 180° gives 23°51′ at J2000.0,
  the galactic centre at 240° gives 26°51′, Cochrane's and Gil Brand's
  members differ from it by their offsets exactly, Wilhelm's
  right-ascension reading differs by eight arcminutes, the two
  galactic-pole members by three arcminutes, and a geometric anchor's
  rate is the general precession.
- Against Teimeris (`fixtures/teimeris/stars.json`, the adapter's
  `stars-table` binary; `crates/astro/tests/teimeris_stars.rs`): for the
  126 ICRS rows the engine knows by name, the SDK's pipeline over the
  engine's own astrometry reproduces its places at four instants from
  1900 to 2100, 1512 places: the mean places (aberration and deflection,
  no nutation) bit for bit, the apparent places within 0.00044″ (the
  engine's nutation against IAU 2000B), the true positions within the
  same when the parallax is kept as the engine keeps it; bound 0.002″.
  The engine's mean obliquity is the SDK's Vondrák series to the printed
  digit at every instant and its nutation within 0.0004″. The SDK's own
  astrometry against the engine's for the same stars (492 places): the
  Hipparcos rows agree to the milliarcsecond; the Gaia rows differ from
  the engine's older rows by up to 5.0″ over a century (Errai, fifty
  milliarcseconds a year apart in proper motion) or by two or three
  arcseconds in the position itself (Heze, Aljanah, Alnasl, Algieba);
  bound 10″, a guard against a wrong unit or a wrong star, since the
  difference is the catalogues' and not the SDK's. Every row compares,
  and the test asserts that none is left out: five were reported rather
  than compared until the engine's table was corrected
  (`05-testing/02-engine-findings.md`, F5, closed 2026-09-06) — its
  Rigil Kentaurus carried a proper motion 224 mas/yr from Hipparcos's,
  its Algedi was α¹ Capricorni where the IAU name is α²'s, its Sadalbari
  λ Pegasi where the IAU name is μ's, its Sgr A* an east rate with cos δ
  applied twice, and its built-in IAU 1958 pole the B1950 definition. The
  worst is now Rigil Kentaurus at 6.3″ in 2100, Gaia against Hipparcos
  for the nearest star.
- The anchored members against Teimeris's recorded values over −700 to
  2500 (`fixtures/teimeris/ayanamsha.json`, `teimeris_ayanamsha.rs`),
  arcseconds, the SDK less the engine:

  | member | J2000.0 | 2025 | 700 CE | 2500 | why |
  |---|---:|---:|---:|---:|---|
  | `TRUE_CHITRA`, `TRUE_MULA` | 0.0000 | 0.0000 | −0.0026, +0.0001 | −0.0004, 0.0000 | the same Hipparcos rows |
  | `GALEQU_TRUE`, `GALEQU_MULA` | −0.0002 | −0.0002 | −0.0002 | −0.0002 | the same pole, geometric |
  | `GALEQU_IAU1958` | −0.0002 | −0.0002 | −0.0002 | −0.0002 | the same pole since the engine's built-in row was corrected (C42, F5); it was +0.176″ at 700 CE against the B1950 definition |
  | the four `GALCENT_*` | +0.020 | +0.010 | +0.025 | −0.007 | the engine's FK5 record (C40); the growth to +0.54″ at 700 CE was its cos δ on the proper motion, corrected (F5) |
  | `TRUE_REVATI` | −0.016 | −0.045 | +1.44 | −0.59 | Gaia DR3 against Hipparcos, 1.1 mas/yr (C41) |
  | `TRUE_PUSHYA`, `TRUE_SHEORAN` | −0.002 | −0.030 | +1.46 | −0.56 | the same (C41) |

  The bounds are 0.005″ for the same rows, 0.3″ for the 1958 pole and the
  data's drift times the years for the rest.

## 9. Localisation

The star names are the catalogue keys' names in the locale packs, as
every member's; the designations are Latin genitives and stay so.

## 10. Open questions

1. **The full catalogue.** The fixed-star literature reads a few hundred
   stars (Solar Fire's 290); the same SIMBAD query extends the kind
   without changing a line of code, once the names and identifications
   are curated.
2. **A topocentric star.** The observer's own velocity in the aberration
   (465 m/s at the equator, 0.3″) and the diurnal parallax (nil for a
   star) come with the completion's centre step in Phase 3.
3. **A cache of the Earth's state** by instant, should a consumer place
   many stars or many instants under a star-anchored zodiac.
4. **The anchors' data against the engine's.** Two anchors are Gaia DR3
   rows where the engine's are Hipparcos, and the galactic centre's
   proper motion follows Reid and Brunthaler's convention where the
   engine's carries an extra cos δ; the resulting differences in the
   anchored ayanamshas are registered as deliberate (§8, the cruxes
   register).
