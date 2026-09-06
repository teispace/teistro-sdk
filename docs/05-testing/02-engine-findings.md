# Engine findings

Status: `active`, opened 2026-09-05; every finding closed 2026-09-06. The register of discrepancies the
SDK's measurements traced to the reference engine, Teimeris
(`github.com/teispace/teimeris`), rather than to the SDK. Each finding
is measured, filed upstream as an issue with its reproduction and
assigned to the engine's maintainer, and recorded here with the SDK's
handling, so that a fixture bound that looks loose has its reason on
this page and a fixed engine can tighten it.

## The rule

A comparison against the engine that fails is not resolved by widening
the bound. It is traced: to the SDK (fixed), to a documented convention
either side chose (registered in the cruxes register,
`01-research/feature-universe/19-verification-cruxes.md`, and the
deliberate-difference registry), or to the engine (measured, filed
upstream with a reproduction and a suggested fix, assigned, and entered
below with the bound the SDK holds it at meanwhile). When the upstream
issue closes, the fixture is regenerated and the bound tightened, and
the row here says so. Upstream data inherited from its own upstream
(the Swiss Ephemeris) is filed all the same; the engine is where the
SDK's users meet it.

## Findings

All six are fixed upstream in `eba52e6` (2026-09-06): four behind the
engine's `MAX` profile, and two unconditionally (F3, an additive API
change that moves no number, and F4, where the engine's own upstream
disagreed with itself). The SDK's adapter takes the profile from
`$TEIMERIS_PROFILE` and the recorded tables say which they were taken
under (`fixtures/teimeris/*.json`, `"profile": "max"`); the fixtures were
re-recorded and the bounds tightened the same day.

| # | finding | measured | upstream | how the SDK holds it now |
|---|---|---|---|---|
| F1 | Sidereal time steps where the long-term branch takes over from the IERS 2010 expression: the fixed joining offsets no longer meet under the default Delta T and precession models | −1.909″ at JD 2469807.5 (2050-01-01) and +0.098″ at JD 2396758.5 (1850-01-01), the bounds themselves taken by the long-term branch; beyond them the branch departs from the IERS 2010 expression by −0.50″ at 1700, +0.36″ at 1800, −1.79″ at 2100, −0.68″ at 2200 and +2.46″ at 2300, so it does not track the expression either; 0.127 s in the engine's equation of time from 2050 | [teimeris#1](https://github.com/teispace/teimeris/issues/1) | the branch now meets the expression at the window's bounds within 0.0014″, held by `tests/teimeris_sidereal.rs` as the regression test for the fix; beyond the window it remains a different model (−0.59″ at 1700, +4.36″ at 2300), reported and not held. The equation of time went from 0.127 s at 2050 to 7.5 ms at 2100, the model's own departure. **Closed.** The engine's answer records that the step is inherited from its upstream verbatim rather than caused by its defaults |
| F2 | The phenomena's Moon: the apparent diameter from the light-time-corrected distance, the horizontal parallax from the geometric one, 40 km apart | ±40 km (1e-4); 0.16″ of disc, 0.32″ of parallax | [teimeris#2](https://github.com/teispace/teimeris/issues/2) | both readings come from the apparent distance; the Moon's parallax agrees to 0.0002″ of its 3400″, and the bound went from 0.5″ to 0.001″, two thousand times tighter. **Closed.** |
| F3 | A body without a magnitude (the nodes, the apogees) is reported as magnitude 0.0; the binding gives `Some(0.0)` | exact | [teimeris#3](https://github.com/teispace/teimeris/issues/3) | the engine now reports a third state (`TM_MAGNITUDE_NONE`) rather than zero, unconditionally; the SDK's `Phenomena.magnitude` is `None` for a point and the fixture agrees exactly. **Closed.** |
| F4 | The Horizon house system's Munkasey co-ascendant at latitude 0 takes the southern branch (0°) where every other system takes the northern (180°): the system's latitude transform restores a value a hair below zero | 0.000° against 180.000° at φ = 0 exactly | [teimeris#4](https://github.com/teispace/teimeris/issues/4) | the six equatorial rows are compared like every other: 25,200 values, worst 0.002″, no exception. **Closed.** The engine's answer records that all 25 other systems returned 180° and only Horizon returned 0° |
| F6 | The IAU 2000B nutation returns the luni-solar series alone: the model's fixed offsets in lieu of the planetary terms (−0.135 mas in longitude, +0.388 mas in obliquity; McCarthy and Luzum 2003) are not added, as in its upstream | +0.135 mas and −0.388 mas exactly at J2000.0 against the SDK's `nut00b`, where the two readings' arguments agree; 0.12 mas of sidereal time | [teimeris#6](https://github.com/teispace/teimeris/issues/6) | the engine adds the offsets under `MAX`; what remains is the two readings of the model's arguments (C43), 0.0058″ from 1700 to 2300. **Closed.** The engine's answer corrects the absolute values quoted in the issue's reproduction; the relative claim was right |
| F5 | Star catalogue rows: Sadalbari on λ Pegasi (the IAU name is μ Pegasi's), Algedi on α¹ Capricorni (α²'s), Rigil Kentaurus with a proper motion 224 mas/yr from Hipparcos's, Sgr A*'s east proper motion with cos δ applied twice (0.4 mas/yr short), the built-in IAU 1958 galactic pole as the B1950 definition where the file's row is the ICRS transform | 4702″, 381″, 22″ a century, 0.01″ by 2026 and 1″ by 700 BCE, 0.18″ | [teimeris#5](https://github.com/teispace/teimeris/issues/5) | every row of the engine's table now compares, where five were left out: `tests/teimeris_stars.rs` asserts that none is. What remains is Gaia against Hipparcos, worst Rigil Kentaurus 6.3″ at 2100. The galactic-centre ayanamshas' bound went from 0.68″ at 700 CE to 0.05″ flat, and the IAU 1958 pole's from 0.3″ into the general 0.005″. **Closed.** The engine's answer measures Sadalbari's separation at 4695″ rather than 4702″ |

## What the round trip taught

The register's rule ends "when the upstream issue closes, the fixture is
regenerated and the bound tightened, and the row here says so". Doing it
once, on all six at once, showed three things worth keeping:

- **A profile is part of a fixture's provenance.** The engine ships
  `COMPATIBLE` (its own upstream, bit for bit) and `MAX` (the corrected
  astronomy). Four of the six fixes live only in `MAX`, so a table that
  does not say which profile it was taken under cannot be read. Every
  recorded table now carries `"profile"`.
- **A bound that a finding widened should say so, so it can be found
  again.** Each of the tightened bounds names the finding it came from
  and the number it held before, which is how they were located when the
  issues closed.
- **An exception is better counted than skipped.** The star comparison
  now asserts that no row is left out, rather than silently comparing
  what is left; the same test would have said nothing if the engine's
  table had regressed.

## Observations resolved

- The +0.088″ once read "inside the window" at 1850 (F1's first table)
  was the boundary instant JD 2396758.5 itself, which the engine's
  inclusive comparison gives to the long-term branch. Measured on
  2026-09-05 at 41 instants strictly inside the window, the engine's
  default sidereal time is the IERS 2010 expression and agrees with the
  SDK's IAU 2006 form (`gst06b`) within 0.0012″ at 1850 and 0.0004″
  from 1875, the remainder the two readings of the 2000B nutation (C43,
  F6); the SDK's IAU 2000 expression (`gst00b`) was the one 0.0096″
  away at 1850, and the SDK moved to the IAU 2006 expression the same
  day (`03-design/astro-events-and-crossings.md` §4).
