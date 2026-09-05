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
| F1 | Sidereal time steps where the long-term branch takes over from the IERS 2010 expression: the fixed joining offsets no longer meet under the default Delta T and precession models | −1.909″ at JD 2469807.5 (2050-01-01), +0.097″ at JD 2396758.5 (1850-01-01); +0.088″ drift by 1850 inside the window; 0.127 s in the engine's equation of time from 2050 | [teimeris#1](https://github.com/teispace/teimeris/issues/1) | the SDK's GAST (IAU 2000 with the 2000B equation of the equinoxes) is continuous; `tests/teimeris_phenomena.rs` holds the equation of time to 1 ms through 2030 and 0.2 s after; every sidereal-time-dependent comparison (houses, rise and set) is at instants inside the window |
| F2 | The phenomena's Moon: the apparent diameter from the light-time-corrected distance, the horizontal parallax from the geometric one, 40 km apart | ±40 km (1e-4); 0.16″ of disc, 0.32″ of parallax | [teimeris#2](https://github.com/teispace/teimeris/issues/2) | the SDK reads both from the apparent distance; the parallax is held to 0.5″ for the Moon, the disc to the rounding for every body but the Sun |
| F3 | A body without a magnitude (the nodes, the apogees) is reported as magnitude 0.0; the binding gives `Some(0.0)` | exact | [teimeris#3](https://github.com/teispace/teimeris/issues/3) | the SDK's `Phenomena.magnitude` is `None` for a point; the fixture test accepts the engine's zero for a point |
| F4 | The Horizon house system's Munkasey co-ascendant at latitude 0 takes the southern branch (0°) where every other system takes the northern (180°): the system's latitude transform restores a value a hair below zero | 0.000° against 180.000° at φ = 0 exactly | [teimeris#4](https://github.com/teispace/teimeris/issues/4) | `tests/teimeris_houses.rs` skips the six equatorial Horizon rows' Munkasey co-ascendant |
| F5 | Star catalogue rows: Sadalbari on λ Pegasi (the IAU name is μ Pegasi's), Algedi on α¹ Capricorni (α²'s), Rigil Kentaurus with a proper motion 224 mas/yr from Hipparcos's, Sgr A*'s east proper motion with cos δ applied twice (0.4 mas/yr short), the built-in IAU 1958 galactic pole as the B1950 definition where the file's row is the ICRS transform | 4702″, 381″, 22″ a century, 0.01″ by 2026 and 1″ by 700 BCE, 0.18″ | [teimeris#5](https://github.com/teispace/teimeris/issues/5) | the SDK's table is SIMBAD's with each value's bibcode (`astro-star-table.md`); `tests/teimeris_stars.rs` reports the three rows and compares the rest; the anchored ayanamshas' bounds carry the data differences (cruxes C40 to C42) |

## Observations not yet filed

- Inside the IERS window the engine's sidereal time drifts from the
  SDK's IAU 2000 GAST to +0.088″ by 1850 (F1's table): the two
  expressions differ (the engine's IERS 2010 form with the 33 periodic
  terms against the IAU 2000 GMST polynomial with the IAU 2000B equation
  of the equinoxes), and which is closer to the truth at 1850 is not
  settled here. The SDK's ports of `gmst06` and the IAU 2006 form are at
  hand; a comparison against ERFA's `gst06a` at 1850 decides whether the
  SDK moves to the 2006 expression before this is filed.
