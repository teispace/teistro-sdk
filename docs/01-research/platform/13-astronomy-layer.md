# The SDK-native astronomy layer

Status: `research`, 2026-09-04. Follows from Q8 (everything in the SDK from
v1) and Q15 (Teimeris may be updated as needed). Feeds
`02-architecture/02-ephemeris-port.md` and the `astro` crate in the module
catalogue. Sources: Teimeris's headers and `DEFERRED.md` (which lists the
models Swiss Ephemeris offers: 11 precession, 5 nutation, 5 Delta T, 4
sidereal-time and 3 frame-bias models, all verified bit-identical or to
1e-14 arcsec against upstream), the Swiss Ephemeris documentation on
sidereal modes, the IAU SOFA conventions, Meeus's *Astronomical
Algorithms*, and Astronomy Engine's design.

## Why the SDK owns this

A provider can only be trusted for what it declares, and every provider
declares something different: Swiss and Teimeris give everything, a JPL
kernel gives geometric barycentric vectors, a truncated analytic series
gives heliocentric ecliptic coordinates of date. If the SDK computed only
astrology and left the rest to providers, a chart's houses, sidereal
longitudes and sunrise would change meaning with the provider. Owning the
layer above raw positions is what makes "same behaviour everywhere" true,
and it is what a market-level product has to own.

## Contents of the `astro` crate

| area | what | models and references | conformance target vs Teimeris |
|---|---|---|---|
| time scales | JD and MJD, split-JD representation for sub-millisecond precision, TT, UT1, UTC with a leap-second table, TDB approximation | IAU SOFA conventions | exact |
| Delta T | Espenak and Meeus 2006 polynomials, Stephenson, Morrison and Hohenkerk 2016, IERS tables for the modern span, the other models Swiss offers as selectable variants | five models as in Swiss and Teimeris | bit-identical per model (Teimeris achieved this) |
| sidereal time | GMST and GAST, IAU 2006/2000A; the older IAU 1982 and the other models as variants | four models | 1e-9 arcsec |
| precession | IAU 2006 (Capitaine), Vondrák 2011 long-term (the Swiss default), and the nine other models Swiss selects (Lieske 1977, Williams 1994, Simon 1994, Laskar 1986, Bretagnon 2003, Owen 1990, and so on) with matching obliquity expressions | eleven models | bit-identical per model |
| nutation | IAU 2000A (1365 terms) and 2000B (77 terms), IAU 1980; optional interpolation | five models | 1e-9 arcsec |
| frame bias and reference frames | ICRS, J2000 mean, ecliptic and equator of date | three models | bit-identical |
| position completion | from whatever the provider returns (geometric heliocentric or barycentric or geocentric, J2000 or of date) to apparent geocentric ecliptic of date: light-time iteration, annual aberration, gravitational deflection, nutation; and the inverse selections (true, no-nutation, no-aberration) as flags | as Swiss's flag set | 1e-6 arcsec when fed identical geometric vectors |
| topocentric correction | observer geodetic to geocentric (WGS84), parallax applied to positions, especially the Moon | | 1e-6 arcsec |
| coordinate transforms | ecliptic, equatorial, horizontal; refraction models (Bennett, Sæmundsson, Swiss's) | | exact to rounding |
| ayanamsha catalogue | all 47 Swiss sidereal modes in three families: epoch plus rate (Fagan/Bradley, Lahiri, De Luce, Raman, Ushashashi, Krishnamurti, Djwhal Khul, Yukteshwar, JN Bhasin, the Babylonian variants, Aldebaran 15 Taurus, Hipparchos, Sassanian, Galactic Centre 0 Sagittarius, J2000, J1900, B1950, Suryasiddhanta and Aryabhata variants, Lahiri 1940, Lahiri VP285, Krishnamurti VP291, Lahiri ICRC, Britton, Dhruva, Skydram); star-anchored (True Citra with Spica at 180°, True Revati with ζ Piscium at 359°50′, True Pushya with δ Cancri at 106°, True Mula, Galactic Centre and Galactic Equator variants using the star table with proper motion); and the siddhantic ones (Suryasiddhanta with true or mean Sun, Aryabhata 522); plus custom epoch, value and rate; mean or nutated output | the Swiss documentation and source semantics, re-derived, never copied | epoch-based bit-identical; star-anchored to 0.001 arcsec (depends on the star table and proper-motion model, to be measured) |
| star table | anchor stars for the ayanamshas and the 27 nakshatra yogataras with Hipparcos positions and proper motions, precessed to date; full catalogue (Solar Fire's 290) in v1.x | Hipparcos/Gaia data, open | 0.01 arcsec |
| house systems | all systems Swiss and Teimeris offer (Placidus, Koch, Porphyry, Regiomontanus, Campanus, equal from the ascendant, whole sign, meridian, Morinus, horizontal, Polich-Page topocentric, Alcabitius, Gauquelin sectors, Krusinski-Pisa-Goelzer, APC, Vehlow, axial rotation, Carter poli-equatorial, Sunshine and Sunshine Treindl, equal from MC, equal from 0° Aries, Pullen SD, Pullen SR, Sripati, and the Vedic Bhava-Chalit variants) with the polar-latitude behaviour of each stated and gated; cusp speeds | implemented from the published definitions and papers (Swiss's source is AGPL and is not reused) | 1e-6 degrees against Teimeris at 10 latitudes, including the polar failure modes |
| rise, set, transit | iterative altitude solver with disc convention (centre, upper limb), refraction on or off, custom horizon altitude, elevation correction; polar day and night as reported states | Meeus with iteration to convergence | 1 second against Teimeris `tm_event_search` |
| crossings and stations | bracketing scan with a step bounded by the fastest body's speed and the feature size, then Brent's method to a stated time tolerance; single body, composite angle `a·lon(A)+b·lon(B)` (tithi, yoga, karana, nakshatra lattices, returns, lunations, ingresses), speed-zero for stations; the two step-size hazards Teimeris recorded (a 40-day step swallowing Mercury's retrograde arc; a Pluto pair 24 days apart) become tests | | 1 second against Teimeris |
| eclipses and occultations | Besselian elements from Sun and Moon positions; global and local circumstances; lunar eclipses from shadow geometry | Meeus and the Explanatory Supplement | v1.x, 1 second of contact times |
| equation of time, planetary hours, twilight definitions | | | exact |

## Precision policy (Q13: the best we can)

`f64` throughout, with the numerical hygiene that makes `f64` deliver its
15 significant digits: split Julian days for sub-millisecond instants,
angle normalisation through one function, compensated summation where a
long series would lose bits (measured before adopting, as Teimeris did),
no fast-math, iteration to convergence with caps, and every tolerance
stated in the results schema. Where a computation is bit-identical to
Teimeris by construction it is gated as such; where it is not (different
algorithm for the same definition), the agreement is measured and
published.

## The oracle

Teimeris is the oracle for every row above: its conformance suite already
holds these models to upstream, so the SDK's agreement with Teimeris is
transitively agreement with Swiss Ephemeris. The SDK's test suite drives
Teimeris through the adapter and compares; the accuracy document is
generated from that run.

## What this changes elsewhere

- The ephemeris port's required surface is positions only; all higher
  operations are optional overrides.
- The module catalogue gains `astro` (this layer) between the ports and the
  domain modules.
- Houses, points, panchanga, gochar and muhurta depend on `astro`, not on
  provider capabilities.
- The roadmap gains a dedicated phase for the astronomy layer before the
  chart core, because everything above it needs it.
