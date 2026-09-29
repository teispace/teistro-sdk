# Sade Sati, measured

Status: `generated` by `cargo xtask sade-sati`. Do not edit:
`check-sade-sati` regenerates this page and fails on any difference.

The design this measures is `sade-sati.md`. **Nothing records a Sade
Sati** — no classical text names it (C147) and the corpus has no
transit search — so this page holds the periods to what the sky says.
All 55 recorded births, founded under the default profile (with a polar
day reckoned from civil midnight, since Tromsø has no June sunrise to
start one), are asked for every period of 1950 to 2050 in **one batch**
for each reckoning (`sdk.chart().sade_sati_many`), from the natal Moon,
with the 4th and the 8th as the smaller spells (C149). The profile is
geocentric, so every chart reads one scan of Saturn.

## 1. What a century holds

Every period reaching into the century, whole, per reckoning. A phase is
**re-entered** when Saturn turned retrograde across its edge and came
back, so its spell is more than one visit; a Sade Sati's length runs
from its first entry into the 12th to its last exit from the 2nd.

| reckoning | Sade Satis | phases re-entered | most visits to one house | 4th | 8th | shortest | longest |
|---|---:|---:|---:|---:|---:|---:|---:|
| Sign | 205 | 518 | 3 | 193 | 186 | 6.36 years | 8.89 years |
| Degree | 202 | 524 | 3 | 192 | 183 | 7.02 years | 8.94 years |

The two reckonings move a Sade Sati's start by up to 663 days either
way: the degree reading's first line is the Moon less 45°, the sign
reading's the start of the sign before the Moon's, and the two differ by
as much as the Moon's distance from the middle of its sign.

## 2. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| every bound of every visit is where the gochar a second either side moves Saturn into or out of the visit's house | **holds** | 0 of 16428 disagree |
| every Sade Sati is the rising, peak and setting phases, in that order | **holds** | 0 of 407 disagree |
| the degree reckoning begins earlier than the sign reckoning exactly when the natal Moon stands in the first half of its sign | **holds** | 0 of 202 disagree |
| a Sade Sati asked about at one instant inside it is, to the bit, the one a century's search finds | **holds** | 0 of 24 disagree |
| a batch of charts answers each as that chart alone | **holds** | 0 of 6 disagree |
| joining visits to one house under 270 days apart, the baseline engine's rule, groups them as the circuit does | **holds** | the longest pause inside a period is 237 days, and the nearest two periods of one house come 9537 days apart |

None of the six claims is falsified. The read-back reads the gochar, a
second consumer of the same longitudes, rather than the search that
found the bounds, and the batch and window claims hold the answer to the
bit rather than to a tolerance: the search samples an anchored grid and
unwraps each step from its own sample (`astro-events-and-crossings.md`
§4), so a window's start cannot move a bound.
