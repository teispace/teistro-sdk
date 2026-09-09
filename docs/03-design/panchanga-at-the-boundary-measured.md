# The panchanga at the boundary, measured

Status: `generated` by `cargo xtask almanac`. Do not edit:
`check-almanac` regenerates this page and fails on any difference. The
design it is written for is [`panchanga-day.md`](panchanga-day.md) §14
at the boundary, beside
[`chart-at-the-boundary.md`](chart-at-the-boundary.md).

A blob has to state where one day's rows end and the next day's begin.
For a chart that was free: the grahas are the kind's, the bhavas are
twelve, and one stride serves the batch
([`chart-at-the-boundary.md`](chart-at-the-boundary.md) §4). A
panchanga's lists are not like that, and this pass measures how unlike.

The sample is **450 days at 3 places** — 408 ordinary and 42 polar —
founded over the analytic provider: a year at Kathmandu, and both
solstices at Reykjavík (64.15°N, where the Sun still rises) and at
Tromsø (69.65°N, where for weeks it does not). 0 range(s) refused.

## 1. A division is fixed; a crossing is not

The fifteen lists split cleanly in two, and the split is not the one a
reader would guess from the names.

| list | ordinary | polar | fixed everywhere? |
|---|---|---|---|
| `choghadiya` | 16 to 16 | 16 to 16 | **yes** |
| `horas` | 24 to 24 | 24 to 24 | **yes** |
| `kaalas` | 3 to 3 | 3 to 3 | **yes** |
| `muhurtas.daylight` | 15 to 15 | 15 to 15 | **yes** |
| `muhurtas.night` | 15 to 15 | 15 to 15 | **yes** |
| `limbs.karana` | 2 to 4 | 3 to 122 | no |
| `limbs.nakshatra` | 1 to 3 | 2 to 60 | no |
| `limbs.tithi` | 1 to 3 | 2 to 62 | no |
| `limbs.yoga` | 1 to 3 | 2 to 65 | no |
| `moon.rises` | 0 to 1 | 0 to 40 | no |
| `moon.sets` | 0 to 1 | 0 to 41 | no |
| `moon.signs` | 1 to 2 | 1 to 28 | no |
| `omens.panchaka` | 0 to 3 | 0 to 10 | no |
| `omens.yogas` | 0 to 2 | 0 to 1 | no |
| `sun.signs` | 1 to 2 | 1 to 3 | no |

Every list that is fixed is a **division of an arc**: twenty-four horas,
fifteen muhurtas of the daylight and fifteen of the night, sixteen
choghadiya, three inauspicious kaalas. Its count is a convention, so it
does not move when the arc does — a polar day whose synthesised arc
runs for a fortnight still has twenty-four horas, each of them fourteen
hours long.

Every list that is not fixed is a **crossing of a quantity inside the
window**: a tithi boundary, a moonrise, the Moon entering a sign. Its
count is set by how long the window is and how fast the quantity moves,
and neither is a convention.

This was not proposed and then checked. The two groups fell out of the
counts, and the rule is what they have in common.

## 2. Rectangular or ragged

A batch can lay a per-day list out two ways. **Rectangular**: one stride
for the whole batch, the widest day's count, with a count column saying
how many rows of each day are real. **Ragged**: the rows concatenated,
with an offset column saying where each day's begin.

Rectangular is the chart blob's shape, and it is tempting here because
it indexes without arithmetic. What it costs is the difference between
the widest day and every other day:

| batch | ragged rows | rectangular rows | wasted |
|---|---|---|---|
| the 408 ordinary days | 35 477 | 39 576 | 10.4% |
| all 450 days | 49 777 | 227 700 | **78.1%** |

Ten per cent is arguable. Seventy-eight is not, and the second
row is what a real caller gets: **one** polar day in a batch sets
the stride for every other day in it. The list that costs the
most is `limbs.karana`: a stride of 122 against a median day's handful, and
50 484 empty rows across the batch.

So the layout is **ragged**, and the same rule for every list rather
than rectangular for the five divisions and ragged for the ten
crossings: two layouts in one blob is two things for a reader to learn,
and the divisions cost nothing under the ragged one — their offsets
are a multiplication their reader never has to know about.

## 3. What is absent rather than empty

Three of a day's values are an `Option`, and a blob has no such thing:
every column has a value in every row. How often each is present decides
whether a sentinel would be read as a value.

| value | present on |
|---|---|
| `muhurtas.abhijit` | 450 of 450 days |
| `muhurtas.brahma` | 450 of 450 days |
| `sun.sankranti` | 39 of 450 days |

A sankranti is rare and the other two are not, which is the shape of the
problem rather than a surprise. None of the three can use a sentinel: an
absent Abhijit and an Abhijit at Julian day zero are both nought, and a
reader cannot tell them apart. So each crosses as a **presence flag
beside its value**, which is the rule the day's tagged enums already
follow ([`chart-at-the-boundary.md`](chart-at-the-boundary.md) §8).

## 4. What refused

Nothing. Every range the sample asks for is founded, at every latitude,
including the twenty-one days at each solstice inside the Arctic Circle.

That is worth stating because it was not true when this pass was first
run. Both Tromsø ranges refused with `NOT_CONVERGED` — *"the rise of
MOON … was not found: no crossing bracketed in 400 steps"* — at
**both** solstices. The cause was not the horizon but a step budget: the
horizon scan's cap was a constant 400, described in its own comment as
"a day of ten-minute steps" though 400 of them is two and three-quarter
days, and a polar day's synthesised arc is longer than that. A Moon that
does not rise at 69.65°N is an answer; a step budget is not. The cap is
now sized from the span the scan is asked to search, with 400 as a
floor, which changes only calls that previously failed.

Until that was fixed the polar column of §1 could not be measured at
all, and the fixed lists looked fixed because nothing had asked them a
hard question.

## 5. What this decides

| proposed rule | verdict | measured |
|---|---|---|
| every per-day list has the same length on every day | falsified | 10 of 15 disagree; so a single stride cannot serve the blob |
| the lists that are fixed are exactly the divisions of an arc | **holds** | 5 of 15 lists fixed, and all 5 are divisions |
| a polar day changes only the crossings, not the divisions | **holds** | 24 horas and 15+15 muhurtas at 69.65°N, as at 27.7°N |
| a crossing list stays small enough for a rectangular layout | falsified | `limbs.karana` reaches 122 on a polar day against 4 on an ordinary one |
| the default profile can found a day at any latitude | falsified | 1 of 1 disagree; a polar day under `day.polar_day_policy = UNDEFINED` is refused by name, which is `panchanga-day.md` §15's first row met in practice |

**The blob's per-day lists are ragged, with an offset column each.** One
rule for all fifteen, because two would be two things to learn and the
divisions lose nothing by it.

**A day's `Option` crosses as a presence flag beside its value**, never
as a sentinel.

**The polar day is a first-class case, not an edge one.** It is the case
that decides the layout, and until the scan's cap was sized from its
span the SDK could not produce one at 69.65°N at all.

What this pass does **not** decide, and the design page has to: whether
a synthesised polar arc a fortnight long should carry sixty-two tithis
at all, or whether an almanac that long is a different question from the
one `Almanac::day` answers.