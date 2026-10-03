# The outer planets, measured

Status: `generated` by `cargo xtask outer-planets` from the corpus's
recorded births, 2026-10-03. Do not edit: `check-outer-planets`
regenerates this page and fails on any difference.

A chart asked `with_outer_planets` places Uranus, Neptune and Pluto
beside the nine (`western-outer-planets.md`). The corpus records the
three under `positions.outer`, from the Earth's centre even under its
topocentric settings. This page founds each of its 55 births under
`conformance-baseline` twice: geocentrically, to hold the ephemeris to
the recording, and as the profile founds it, topocentrically, to measure
what C239's choice of centre moves. Each longitude bound is what the
built-in ephemeris can promise for the body: the theory's own floor seen
from the Earth, and the `standard` tier's arcsecond of truncation.

| proposed rule | verdict | measured |
|---|---|---|
| URANUS's longitude, placed from the Earth's centre, is the recording's within 6.00″ | **holds** | worst 5.11″ over 55 |
| NEPTUNE's longitude, placed from the Earth's centre, is the recording's within 8.00″ | **holds** | worst 6.14″ over 55 |
| PLUTO's longitude, placed from the Earth's centre, is the recording's within 1.00″ | **holds** | worst 0.893″ over 55 |
| C239: placing URANUS from the observer moves it under 1.00″ | **holds** | worst 0.559″ |
| C239: placing NEPTUNE from the observer moves it under 1.00″ | **holds** | worst 0.436″ |
| C239: placing PLUTO from the observer moves it under 1.00″ | **holds** | worst 0.407″ |

## Against the recording, from the Earth's centre

The worst and the median difference over the births, in arcseconds; the
speed's in arcseconds a day.

| body | longitude | latitude | speed | from the observer |
|---|---|---|---|---|
| URANUS | 5.11 / 0.400 | 0.061 / 0.038 | 0.014 / 0.0002 | 0.559 / 0.240 |
| NEPTUNE | 6.14 / 0.718 | 0.177 / 0.044 | 0.0082 / 0.0005 | 0.436 / 0.246 |
| PLUTO | 0.893 / 0.280 | 0.121 / 0.019 | 0.014 / 0.0013 | 0.407 / 0.184 |

## What it means

The first three rows hold the ephemeris: what the chart places from the
Earth's centre is what the recording engine placed, inside the bound the
built-in ephemeris can promise for each body. The next three are C239's
size. The SDK places the outer planets from the chart's own centre, as
it places the nine, and the last column is how far that moves each one
from the recording's geocentric reading.
