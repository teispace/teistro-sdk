# Synastry, measured

Status: `generated` by `cargo xtask synastry` from the corpus's recorded
births, 2026-10-03. Do not edit: `check-synastry` regenerates this page
and fails on any difference.

Each of the 1485 pairs of the corpus's 55 births is read against each
other twice (`western-synastry.md`, C241): in the tropical zodiac, the
default, and in each chart's own. The births are founded under
`conformance-baseline`, whose zodiac is sidereal, with the outer
planets; every point of one is read against every point of the other,
the lagna included, under Leo's nine aspects and his orbs.

| proposed rule | verdict | measured |
|---|---|---|
| every row stands inside the orb its model allowed | **holds** | 0 of 174147 disagree |
| Leo's orbs never put one pair of points at two aspects | **holds** | 0 of 174147 disagree |
| the tropical zodiac and each chart's own find the same contacts | falsified | 16233 of 95190 disagree; the two charts' ayanamshas stand at most 30207″ apart |
| the share of contacts the two readings part on grows with each band of years between the births | **holds** | 0 of 4 disagree |

## By the years between the births

Read in each chart's own zodiac, every separation across the two moves
by the difference of their ayanamshas, the precession between the
births. A contact parts the two readings only when that shift carries it
across its orb's edge.

| years apart | pairs | widest drift, ″ | contacts either finds | found by one only | share |
|---|---|---|---|---|---|
| under 10 | 271 | 527 | 15 916 | 191 | 1.2% |
| 10 to 25 | 317 | 1276 | 19 089 | 855 | 4.5% |
| 25 to 50 | 322 | 2534 | 19 658 | 1692 | 8.6% |
| 50 to 100 | 218 | 5051 | 13 934 | 2172 | 15.6% |
| 100 or more | 357 | 30207 | 26 593 | 11 323 | 42.6% |
