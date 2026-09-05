# Engine findings

Status: `active`, opened 2026-09-05. The register of discrepancies the
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

| # | finding | measured | upstream | the SDK meanwhile |
|---|---|---|---|---|
| F1 | Sidereal time steps where the long-term branch takes over from the IERS 2010 expression: the fixed joining offsets no longer meet under the default Delta T and precession models | −1.909″ at JD 2469807.5 (2050-01-01) and +0.098″ at JD 2396758.5 (1850-01-01), the bounds themselves taken by the long-term branch; beyond them the branch departs from the IERS 2010 expression by −0.50″ at 1700, +0.36″ at 1800, −1.79″ at 2100, −0.68″ at 2200 and +2.46″ at 2300, so it does not track the expression either; 0.127 s in the engine's equation of time from 2050 | [teimeris#1](https://github.com/teispace/teimeris/issues/1) | the SDK's GAST (IAU 2006 with the 2000B nutation, `gst06b`) is continuous and agrees with the engine strictly inside the window within 0.0012″ (`tests/teimeris_sidereal.rs`, which reports the branch rows without holding them); `tests/teimeris_phenomena.rs` holds the equation of time to 1 ms through 2030 and 0.2 s after; every sidereal-time-dependent comparison (houses, rise and set) is at instants inside the window |
| F2 | The phenomena's Moon: the apparent diameter from the light-time-corrected distance, the horizontal parallax from the geometric one, 40 km apart | ±40 km (1e-4); 0.16″ of disc, 0.32″ of parallax | [teimeris#2](https://github.com/teispace/teimeris/issues/2) | the SDK reads both from the apparent distance; the parallax is held to 0.5″ for the Moon, the disc to the rounding for every body but the Sun |
| F3 | A body without a magnitude (the nodes, the apogees) is reported as magnitude 0.0; the binding gives `Some(0.0)` | exact | [teimeris#3](https://github.com/teispace/teimeris/issues/3) | the SDK's `Phenomena.magnitude` is `None` for a point; the fixture test accepts the engine's zero for a point |
| F4 | The Horizon house system's Munkasey co-ascendant at latitude 0 takes the southern branch (0°) where every other system takes the northern (180°): the system's latitude transform restores a value a hair below zero | 0.000° against 180.000° at φ = 0 exactly | [teimeris#4](https://github.com/teispace/teimeris/issues/4) | `tests/teimeris_houses.rs` skips the six equatorial Horizon rows' Munkasey co-ascendant |
| F6 | The IAU 2000B nutation returns the luni-solar series alone: the model's fixed offsets in lieu of the planetary terms (−0.135 mas in longitude, +0.388 mas in obliquity; McCarthy and Luzum 2003) are not added, as in its upstream | +0.135 mas and −0.388 mas exactly at J2000.0 against the SDK's `nut00b`, where the two readings' arguments agree; 0.12 mas of sidereal time | [teimeris#6](https://github.com/teispace/teimeris/issues/6) | the SDK's `nut00b` carries the offsets as ERFA does; `tests/teimeris_sidereal.rs` holds the nutation within 0.01″ from 1700 to 2300 (the arguments' reading is C43) |
| F5 | Star catalogue rows: Sadalbari on λ Pegasi (the IAU name is μ Pegasi's), Algedi on α¹ Capricorni (α²'s), Rigil Kentaurus with a proper motion 224 mas/yr from Hipparcos's, Sgr A*'s east proper motion with cos δ applied twice (0.4 mas/yr short), the built-in IAU 1958 galactic pole as the B1950 definition where the file's row is the ICRS transform | 4702″, 381″, 22″ a century, 0.01″ by 2026 and 1″ by 700 BCE, 0.18″ | [teimeris#5](https://github.com/teispace/teimeris/issues/5) | the SDK's table is SIMBAD's with each value's bibcode (`astro-star-table.md`); `tests/teimeris_stars.rs` reports the three rows and compares the rest; the anchored ayanamshas' bounds carry the data differences (cruxes C40 to C42) |

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
