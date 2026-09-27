# The transit hit list, measured

Status: `generated` by `cargo xtask hits`. Do not edit: `check-hits`
regenerates this page and fails on any difference.

The design this measures is `transit-hit-list.md`. **Nothing
records a hit list** — the corpus has no transit search and no
aspect at all — so this page holds the list to what the sky and a
founded chart say, over 2026. The first recorded birth is asked for
everything: every graha, every kind of event, and the conjunction
and the opposition to each natal graha and the lagna (C145), with a
3° orb (C146). All 55 births, founded under `conformance-baseline`, are
then asked for their aspects in **one batch**
(`sdk.chart().hits_many`), the Moon's excepted: it crosses each
line twenty-seven times a year, which would be most of the batch's
price, and the first chart's Moon shows everything the rest would.
The profile is topocentric, so every read-back chart is founded at
the birth's own place, under the profile with a polar day reckoned
from civil midnight (Tromsø has no January sunrise to start one).

## 1. What a year holds

The sky's events, the same for every chart, which the batch refines once
and hands to each:

| graha | sign ingresses | nakshatra ingresses | stations |
|---|---:|---:|---:|
| Sun | 12 | 27 | 0 |
| Moon | 161 | 361 | 0 |
| Mars | 8 | 18 | 0 |
| Mercury | 14 | 32 | 6 |
| Jupiter | 2 | 3 | 2 |
| Venus | 12 | 25 | 2 |
| Saturn | 0 | 3 | 2 |
| Rahu | 1 | 1 | 0 |
| Ketu | 1 | 2 | 0 |

Against the births' own points: 4792 exact aspects, 4769 windows entered
and 4787 left, of which 79 closed windows held no exact hit at all — a
graha turning inside the orb and going back the way it came.

## 2. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| every sign and nakshatra ingress stands between the division it left and the one it entered in charts founded a second either side | **holds** | 0 of 683 disagree |
| every exact aspect stands at its angle, to a tenth of an arcsecond, in a chart founded at its instant | **holds** | 0 of 4792 disagree |
| successive crossings of one line run opposite ways exactly when the graha stood still an odd number of times between them | **holds** | 0 of 1154 disagree |
| every orb's window opens, holds its exact hits and closes, in that order | **holds** | 0 of 4619 disagree |
| Ketu's every ingress is Rahu's at the same instant, six signs on | **holds** | 0 of 1 disagree |

None of the five claims is falsified. The worst exact aspect stood
0.0000″ from its angle. The parity reads every line the list crosses
— the sign and nakshatra boundaries and each birth's conjunction and
opposition lines — so a station missing from the list, or one reported
where the graha did not turn, breaks it at the next crossing.
