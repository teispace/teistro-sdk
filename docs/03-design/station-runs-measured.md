# Station runs, measured

Status: `generated` by `cargo xtask stations`. Do not edit:
`check-stations` regenerates this page and fails on any difference.

The design this measures is `astro-events-and-crossings.md` §4, "a slow
body's scan". A crossing search samples a slow body every *stride* steps
of its fine grid and reads the grid only where a line or a station lies
between two samples. That is sound only while no two stations fall
inside one stride, so the stride is bounded by three quarters of the
body's shortest run between stations, which `shortest_run_days` states
as a table. This page holds the table to the built-in ephemeris over its
whole coverage, JD 2378497.0 to 2597641.0, sampled daily in the
canonical frame. It then compares every crossing the strided scan finds
with the fine scan's, over signs, nakshatras and a single point
together: from 1900 to 2100 in the canonical frame, and from 2000 to
2020 seen from Delhi and from Tromsø, where a slow body near its
station wobbles by its daily parallax.

## 1. What the sky does

Runs are in days, from one station to the next, each station placed by
the speed's straight line between the daily samples either side.
**instants** is what the strided scan asked the ephemeris for, as a
share of what the fine scan asked; a body with no table keeps the fine
scan, and its stride is 1.

| body | stations | shortest retrograde | shortest direct | table | stride | crossings | instants |
|---|---:|---:|---:|---:|---:|---:|---:|
| Sun | 0 | — | — | — | 12 | 8000 | 44.9% |
| Moon | 0 | — | — | — | 2 | 106 947 | 98.3% |
| Mercury | 3783 | 19.76 | 87.83 | 19.50 | 5 | 9869 | 61.4% |
| Venus | 750 | 40.87 | 537.96 | 40.50 | 10 | 8452 | 49.0% |
| Mars | 562 | 59.78 | 684.14 | 59.50 | 14 | 4593 | 32.5% |
| Jupiter | 1099 | 117.45 | 272.41 | 117.00 | 44 | 1059 | 15.5% |
| Saturn | 1159 | 133.62 | 235.41 | 133.50 | 88 | 553 | 13.1% |
| Uranus | 1185 | 149.00 | 214.98 | 148.50 | 111 | 274 | 11.4% |
| Neptune | 1193 | 156.32 | 207.13 | 156.00 | 117 | 182 | 10.5% |
| Pluto | 1195 | 156.18 | 200.81 | 156.00 | 117 | 148 | 10.3% |
| MeanNode | 0 | — | — | — | 111 | 430 | 3.7% |
| TrueNode | 28 079 | 0.08 | 0.00 | none | 1 | 0 | — |

## 2. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| each planet's table entry lies between 95% and all of the shortest run it made between two stations | **holds** | 0 of 8 disagree |
| the Sun, the Moon and the mean node never turn | **holds** | 0 of 3 disagree |
| the true node turns within a day, so it keeps the fine scan | **holds** | its shortest run is 0.00 days |
| every body but the true node has a table entry, and the true node has none | **holds** | 0 of 12 disagree |
| the strided scan finds every crossing the fine scan finds, to the bit, geocentric | **holds** | 0 of 140507 disagree; 53.6% of the fine scan's instants |
| the same holds topocentric, seen from Delhi and Tromsø | **holds** | 0 of 28064 disagree; 56.4% of the fine scan's instants |

None of the six claims is falsified. The comparison is bit for bit, not
to a tolerance: the strided scan refines the same cells of the same
anchored grid from the same two samples, so any difference would be a
crossing it looked past.
